#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod structs;

#[allow(non_local_definitions)] // error molesto
#[ink::contract]
mod rustaceo_libre {
    use ink::storage::Mapping;
    use ink::prelude::{vec::Vec, string::String, collections::BTreeMap};

    // structs propias
    use crate::structs::usuario::{
        Usuario,
        Rol,
        ErrorRegistrarUsuario,
        ErrorModificarRolUsuario,
    };
    use crate::structs::producto::{
        Producto,
        CategoriaProducto,
        ErrorRealizarPublicacion,
        ErrorVerProductosVendedor,
    };
    use crate::structs::compra::{
        Compra,
        EstadoCompra,
        ErrorComprarProducto,
        ErrorCompraDespachada,
        ErrorCompraRecibida,
        ErrorCancelarCompra,
        ErrorVerCompras,
        ErrorVerVentas,
    };

    //
    // RustaceoLibre: main struct
    //

    /// Definición de la estructura del contrato
    #[ink(storage)]
    pub struct RustaceoLibre {
        /// <ID del usuario, Usuario>
        pub usuarios: Mapping<AccountId, Usuario>,
        /// <ID, Compra>
        pub compras: BTreeMap<u128, Compra>,
        /// <ID, Publicacion>
        pub publicaciones: BTreeMap<u128, Producto>,
        /// Lleva un recuento de la próxima ID disponible para los productos.
        pub publicaciones_siguiente_id: u128,
        /// Lleva un recuento de la próxima ID disponible para las compras.
        pub compras_siguiente_id: u128,
        /// ID del dueño del contrato
        pub owner: AccountId,
    }

    #[ink(impl)]
    impl RustaceoLibre {
        /// Construye un nuevo contrato con sus valores por defecto
        #[ink(constructor)]
        pub fn new() -> Self {
            Self::_new()
        }

        /// Constructor por defecto (Ídem new())
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::_new()
        }

        /// Crea una nueva instancia de RustaceoLibre
        fn _new() -> Self {
            Self {
                usuarios: Default::default(),
                compras: Default::default(),
                publicaciones: Default::default(),
                publicaciones_siguiente_id: 0,
                compras_siguiente_id: 0,
                owner: Self::env().caller(),
            }
        }

        //
        // /structs/usuario.rs
        //

        /// Registra un usuario en el Mapping de usuarios.
        /// 
        /// Devuelve error si el usuario ya existe.
        #[ink(message)]
        pub fn registrar_usuario(&mut self, rol: Rol) -> Result<(), ErrorRegistrarUsuario>  {
            self._registrar_usuario(self.env().caller(), rol)
        }

        ///////////////

        /// Recibe un rol y lo modifica para ese usuario si ya está registrado.
        /// 
        /// Devuelve error si el usuario no existe o ya posee ese rol.
        #[ink(message)]
        pub fn modificar_rol_usuario(&mut self, rol: Rol) -> Result<(), ErrorModificarRolUsuario> {
            self._modificar_rol_usuario(self.env().caller(), rol)
        }

        //
        // /structs/vendedor.rs
        //

        /// Realiza una publicación con producto, precio y cantidad.
        /// 
        /// Devuelve Error si el precio o la cantidad son 0, o si `caller` no existe o no es vendedor.
        #[ink(message)]
        pub fn realizar_publicacion(&mut self, nombre: String, descripcion: String, categoria: CategoriaProducto, precio: Balance, stock: u32) -> Result<u128, ErrorRealizarPublicacion> {
            self._realizar_publicacion(self.env().caller(), nombre, descripcion, categoria, precio, stock)
        }

        ///////////////
        
        /// Dada una ID, devuelve la publicación del producto si es posible
        #[ink(message)]
        pub fn ver_producto(&self, id_producto: u128) -> Option<Producto> {
            self._ver_producto(id_producto).cloned()
        }

        ///////////////
        
        /// Devuelve todos los productos publicados por el usuario que lo ejecute
        /// 
        /// Dará error si el usuario no está registrado como vendedor o si no tiene publicaciones.
        #[ink(message)]
        pub fn ver_productos_vendedor(&self) -> Result<Vec<Producto>, ErrorVerProductosVendedor> {
            self._ver_publicaciones_vendedor(self.env().caller())
        }
        
        //
        // compras.rs: administrar compras
        //

        /// Compra una cantidad de un producto
        /// 
        /// Puede dar error si el usuario no existe, no es comprador, la publicación no existe,
        /// el stock es insuficiente o el vendedor de la misma no existe.
        #[ink(message)]
        pub fn comprar_producto(&mut self, id_publicacion: u128, cantidad: u32) -> Result<u128, ErrorComprarProducto> {
            self._comprar_producto(self.env().caller(), id_publicacion, cantidad)
        }

        ///////////////

        /// Si la compra indicada está pendiente y el usuario es el vendedor, se establece como recibida.
        /// 
        /// Puede dar error si el usuario no está registrado, la compra no existe,
        /// la compra no está pendiente, ya fue recibida, es el cliente quien intenta despacharla
        /// o ya fue cancelada.
        #[ink(message)]
        pub fn compra_despachada(&mut self, compra_id: u128) -> Result<(), ErrorCompraDespachada> {
            self._compra_despachada(self.env().caller(), compra_id)
        }

        ///////////////
        
        /// Si la compra indicada fue despachada y el usuario es el comprador, se establece como recibida.
        /// 
        /// Puede dar error si el usuario no está registrado, la compra no existe,
        /// la compra no fue despachada, ya fue recibida, es el vendedor quien intenta recibirla
        /// o ya fue cancelada.
        #[ink(message)]
        pub fn compra_recibida(&mut self, id_compra: u128) -> Result<(), ErrorCompraRecibida> {
            self._compra_recibida(self.env().caller(), id_compra)
        }

        ///////////////
        
        /// Cancela la compra si ambos participantes de la misma ejecutan esta misma función
        /// y si ésta no fue recibida ni ya cancelada.
        /// 
        /// Devuelve error si el usuario o la compra no existen, si el usuario no participa en la compra,
        /// si la compra ya fue cancelada o recibida y si quien solicita la cancelación ya la solicitó antes.
        #[ink(message)]
        pub fn cancelar_compra(&mut self, id_compra: u128) -> Result<bool, ErrorCancelarCompra> {
            self._cancelar_compra(self.env().caller(), id_compra)
        }

        //
        // compra.rs: visualizar compras
        //

        /// Devuelve las compras del usuario que lo ejecuta
        /// 
        /// Dará error si el usuario no está registrado como comprador o no tiene compras
        #[ink(message)]
        pub fn ver_compras(&self) -> Result<Vec<Compra>, ErrorVerCompras> {
            self._ver_compras(self.env().caller())
        }

        ///////////////

        /// Devuelve las compras del usuario que lo ejecuta que estén en el estado especificado
        /// 
        /// Dará error si el usuario no está registrado como comprador o no tiene compras en ese estado
        #[ink(message)]
        pub fn ver_compras_estado(&self, estado: EstadoCompra) -> Result<Vec<Compra>, ErrorVerCompras> {
            self._ver_compras_estado(self.env().caller(), estado)
        }

        ///////////////

        /// Devuelve las compras del usuario que lo ejecuta que estén en el estado especificado
        /// 
        /// Dará error si el usuario no está registrado como comprador o no tiene compras en ese estado
        #[ink(message)]
        pub fn ver_compras_categoria(&self, categoria: CategoriaProducto) -> Result<Vec<Compra>, ErrorVerCompras> {
            self._ver_compras_categoria(self.env().caller(), categoria)
        }

        //
        // compras.rs: visualizar ventas
        //
        
        /// Devuelve las ventas del usuario que lo ejecuta
        /// 
        /// Dará error si el usuario no está registrado como vendedor o no tiene ventas
        #[ink(message)]
        pub fn ver_ventas(&self) -> Result<Vec<Compra>, ErrorVerVentas> {
            self._ver_ventas(self.env().caller())
        }

        ///////////////

        /// Devuelve las ventas del usuario que lo ejecuta que estén en el estado especificado
        /// 
        /// Dará error si el usuario no está registrado como vendedor o no tiene ventas en ese estado
        #[ink(message)]
        pub fn ver_ventas_estado(&self, estado: EstadoCompra) -> Result<Vec<Compra>, ErrorVerVentas> {
            self._ver_ventas_estado(self.env().caller(), estado)
        }

        ///////////////

        /// Devuelve las ventas del usuario que lo ejecuta que estén en el estado especificado
        /// 
        /// Dará error si el usuario no está registrado como vendedor o no tiene ventas en ese estado
        #[ink(message)]
        pub fn ver_ventas_categoria(&self, categoria: CategoriaProducto) -> Result<Vec<Compra>, ErrorVerVentas> {
            self._ver_ventas_categoria(self.env().caller(), categoria)
        }

        /// Devuelve la siguiente ID disponible para compras
        /// 
        /// Si la próxima ID causaría Overflow, devuelve 0 y reinicia la cuenta.
        pub fn next_id_compras(&mut self) -> u128 {
            let id = self.compras_siguiente_id; // obtener actual
            let add_res = self.compras_siguiente_id.checked_add(1); // sumarle 1 al actual para que apunte a un id desocupado
            if add_res.is_none() {
                self.compras_siguiente_id = 1;
                return 0;
            }
            id // devolver
        }

        /// Devuelve la siguiente ID disponible para publicaciones
        /// 
        /// Si la próxima ID causaría Overflow, devuelve 0 y reinicia la cuenta.
        pub fn next_id_publicaciones(&mut self) -> u128 {
            let id = self.publicaciones_siguiente_id; // obtener actual
            let add_res = self.publicaciones_siguiente_id.checked_add(1); // sumarle 1 al actual para que apunte a un id desocupado
            if add_res.is_none() {
                self.publicaciones_siguiente_id = 1;
                return 0;
            }
            id // devolver
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let rustaceo_libre = RustaceoLibre::default();
            assert_eq!(rustaceo_libre.compras_siguiente_id, 0);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut rustaceo_libre = RustaceoLibre::new();
            assert_eq!(rustaceo_libre.next_id_compras(), 0);
            assert_eq!(rustaceo_libre.next_id_compras(), 1);
            assert_eq!(rustaceo_libre.next_id_publicaciones(), 0);
            assert_eq!(rustaceo_libre.next_id_publicaciones(), 1);
        }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::ContractsBackend;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut c: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = RustaceoLibreRef::default();

            // // When
            // let contract = c
            //     .instantiate("RustaceoLibre", &ink_e2e::alice(), &mut constructor)
            //     .submit()
            //     .await
            //     .expect("instantiate failed");
            // let mut call_builder = contract.call_builder::<RustaceoLibre>();
            //
            // // Then
            // let get = call_builder.next_id_compras();
            // let get_result = c.call(&ink_e2e::alice(), &get).dry_run().await?;
            // assert!(matches!(get_result.return_value(), Some(0)));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = RustaceoLibreRef::new();
            // let contract = client
            //     .instantiate("RustaceoLibre", &ink_e2e::bob(), &mut constructor)
            //     .submit()
            //     .await
            //     .expect("instantiate failed");
            // let mut call_builder = contract.call_builder::<RustaceoLibre>();
            //
            // let get = call_builder.next_id_compras();
            // let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            // assert!(matches!(get_result.return_value(), Some(0)));
            //
            // // When
            // let next_id_compras = call_builder.next_id_compras();
            // let _flip_result = client
            //     .call(&ink_e2e::bob(), &next_id_compras)
            //     .submit()
            //     .await
            //     .expect("flip failed");
            //
            // // Then
            // let get = call_builder.next_id_compras();
            // let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            // assert!(matches!(get_result.return_value(), Some(1)));

            Ok(())
        }
    }
}