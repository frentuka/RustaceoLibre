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
        ErrorAscenderRolUsuario,
    };
    use crate::structs::producto::{
        Producto,
        CategoriaProducto,
        ErrorRegistrarProducto,
    };
    use crate::structs::publicacion::{
        Publicacion,
        ErrorRealizarPublicacion,
        ErrorModificarCantidadOfertada,
        ErrorVerPublicacionesVendedor
    };
    use crate::structs::compra::{
        Compra, ErrorCancelarCompra, ErrorCompraDespachada, ErrorCompraRecibida, ErrorComprarProducto, ErrorReclamarFondos, ErrorVerCompras, ErrorVerVentas, EstadoCompra
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
        /// <ID, Producto>
        pub productos: BTreeMap<u128, Producto>,
        /// <ID, Publicacion>
        pub publicaciones: BTreeMap<u128, Publicacion>,
        /// Lleva un recuento de la próxima ID disponible para las compras.
        compras_siguiente_id: u128,
        /// Lleva un recuento de la próxima ID disponible para los productos.
        productos_siguiente_id: u128,
        /// Lleva un recuento de la próxima ID disponible para las publicaciones.
        publicaciones_siguiente_id: u128,
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
                productos: Default::default(),
                publicaciones: Default::default(),
                compras_siguiente_id: 0,
                productos_siguiente_id: 0,
                publicaciones_siguiente_id: 0,
                owner: Self::env().caller(),
            }
        }

        //
        // /structs/usuario.rs    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        //

        /// Registra un usuario en el Mapping de usuarios.
        /// 
        /// Devuelve error si el usuario ya existe.
        #[ink(message)]
        pub fn registrar_usuario(&mut self, rol: Rol) -> Result<(), ErrorRegistrarUsuario>  {
            self._registrar_usuario(self.env().caller(), rol)
        }

        /// Recibe un rol y lo modifica para ese usuario si ya está registrado.
        /// 
        /// Devuelve error si el usuario no existe o ya posee ese rol.
        #[ink(message)]
        pub fn ascender_rol_usuario(&mut self) -> Result<(), ErrorAscenderRolUsuario> {
            self._ascender_rol_usuario(self.env().caller())
        }

        //
        // /structs/publicacion.rs    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        //

        /// Realiza una publicación con producto, precio y cantidad.
        /// 
        /// Devuelve Error si el precio o la cantidad son 0, o si `caller` no existe o no es vendedor.
        #[ink(message)]
        pub fn realizar_publicacion(&mut self, id_producto: u128, cantidad_ofertada: u32, precio: Balance) -> Result<u128, ErrorRealizarPublicacion> {
            self._realizar_publicacion(self.env().caller(), id_producto, cantidad_ofertada, precio)
        }

        /// Modifica la cantidad ofertada en una publicación,
        /// modificando también el stock del vendedor.
        /// 
        /// Devuelve Error si el usuario no está registrado, la venta no existe,
        /// el usuario no es el vendedor o la operación es imposible por falta de stock/cantidad ofertada.
        #[ink(message)]
        pub fn modificar_cantidad_ofertada(&mut self, id_publicacion: u128, nueva_cantidad_ofertada: u32) -> Result<(), ErrorModificarCantidadOfertada> {
            self._modificar_cantidad_ofertada(self.env().caller(), id_publicacion, nueva_cantidad_ofertada)
        }

        /// Dada una ID, devuelve la publicación
        /// 
        /// Devolverá None si la publicación no existe o el usuario no está registrado
        #[ink(message)]
        pub fn ver_publicacion(&self, id_publicacion: u128) -> Option<Publicacion> {
            self._ver_publicacion(self.env().caller(), id_publicacion)
        }

        /// Devuelve todos los productos publicados por el usuario que lo ejecute
        /// 
        /// Dará error si el usuario no está registrado como vendedor o si no tiene publicaciones.
        #[ink(message)]
        pub fn ver_publicaciones_vendedor(&self) -> Result<Vec<Publicacion>, ErrorVerPublicacionesVendedor> {
            self._ver_publicaciones_vendedor(self.env().caller())
        }

        //
        // structs/producto.rs    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        //

        /// Registra un producto en la lista de productos
        /// para su posterior uso en publicaciones
        /// 
        /// Devuelve error si el usuario no está registrado o no es vendedor.
        #[ink(message)]
        pub fn registrar_producto(&mut self, nombre: String, descripcion: String, categoria: CategoriaProducto, stock_inicial: u32) -> Result<u128, ErrorRegistrarProducto> {
            self._registrar_producto(self.env().caller(), nombre, descripcion, categoria, stock_inicial)
        }
        
        /// Dada una ID, devuelve la publicación del producto
        /// 
        /// Devuelve None si el producto no existe
        #[ink(message)]
        pub fn ver_producto(&self, id_producto: u128) -> Option<Producto> {
            self._ver_producto(self.env().caller(), id_producto)
        }
        
        //
        // compras.rs: administrar compras    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        //

        /// Compra una cantidad de un producto
        /// 
        /// Puede dar error si el usuario no existe, no es comprador, la publicación no existe,
        /// el stock es insuficiente o el vendedor de la misma no existe.
        #[ink(message, payable)]
        pub fn comprar_producto(&mut self, id_publicacion: u128, cantidad: u32) -> Result<u128, ErrorComprarProducto> {
            let operacion = self._comprar_producto(self.env().block_timestamp(), self.env().caller(), id_publicacion, cantidad, self.env().transferred_value());
            
            // si la operación es erronea por cualquier motivo, devolver fondos
            if operacion.is_err() {
                let _ = self.env().transfer(self.env().caller(), self.env().transferred_value());
            }
            
            operacion
        }

        /// Política de reclamo:
        /// 
        /// Si el vendedor despachó la compra y el comprador no la marcó como recibida después de 30 días,
        /// el comprador puede reclamar los fondos de la compra y la misma se marcará automáticamente como recibida.
        /// 
        /// Puede dar error si el usuario no está registrado, la compra no existe,
        /// el usuario no es el vendedor de la compra o su reclamo no condice con la política de reclamo
        #[ink(message)]
        pub fn reclamar_fondos(&mut self, id_compra: u128) -> Result<u128, ErrorReclamarFondos> {
            let operacion = self._reclamar_fondos(self.env().block_timestamp(), self.env().caller(), id_compra);
            
            let Ok(valor) = operacion
            else { return operacion };

            let _ = self.env().transfer(self.env().caller(), valor);

            operacion
        }

        /// Si la compra indicada está pendiente y el usuario es el vendedor, se establece como recibida.
        /// 
        /// Puede dar error si el usuario no está registrado, la compra no existe,
        /// la compra no está pendiente, ya fue recibida, es el cliente quien intenta despacharla
        /// o ya fue cancelada.
        #[ink(message)]
        pub fn compra_despachada(&mut self, compra_id: u128) -> Result<(), ErrorCompraDespachada> {
            self._compra_despachada(self.env().block_timestamp(), self.env().caller(), compra_id)
        }
        
        /// Si la compra indicada fue despachada y el usuario es el comprador, se establece como recibida.
        /// 
        /// Puede dar error si el usuario no está registrado, la compra no existe,
        /// la compra no fue despachada, ya fue recibida, es el vendedor quien intenta recibirla
        /// o ya fue cancelada.
        #[ink(message)]
        pub fn compra_recibida(&mut self, id_compra: u128) -> Result<(), ErrorCompraRecibida> {
            let operacion = self._compra_recibida(self.env().block_timestamp(), self.env().caller(), id_compra);

            let Ok((vendedor, valor)) = operacion
            else { return Err(operacion.unwrap_err()) };

            let _ = self.env().transfer(vendedor, valor);

            Ok(())
        }
        
        /// Cancela la compra si ambos participantes de la misma ejecutan esta misma función
        /// y si ésta no fue recibida ni ya cancelada.
        /// 
        /// Devuelve error si el usuario o la compra no existen, si el usuario no participa en la compra,
        /// si la compra ya fue cancelada o recibida y si quien solicita la cancelación ya la solicitó antes.
        #[ink(message)]
        pub fn cancelar_compra(&mut self, id_compra: u128) -> Result<bool, ErrorCancelarCompra> {
            let operacion = self._cancelar_compra(self.env().block_timestamp(), self.env().caller(), id_compra);

            let Ok(operacion) = operacion
            else { return Err(operacion.unwrap_err()) };

            let Some((comprador, valor)) = operacion
            else { return Ok(false) };

            let _ = self.env().transfer(comprador, valor);

            Ok(true)
        }

        //
        // compra.rs: visualizar compras    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        //

        /// Devuelve las compras del usuario que lo ejecuta
        /// 
        /// Dará error si el usuario no está registrado como comprador o no tiene compras
        #[ink(message)]
        pub fn ver_compras(&self) -> Result<Vec<Compra>, ErrorVerCompras> {
            self._ver_compras(self.env().caller())
        }

        /// Devuelve las compras del usuario que lo ejecuta que estén en el estado especificado
        /// 
        /// Dará error si el usuario no está registrado como comprador o no tiene compras en ese estado
        #[ink(message)]
        pub fn ver_compras_estado(&self, estado: EstadoCompra) -> Result<Vec<Compra>, ErrorVerCompras> {
            self._ver_compras_estado(self.env().caller(), estado)
        }

        /// Devuelve las compras del usuario que lo ejecuta que estén en el estado especificado
        /// 
        /// Dará error si el usuario no está registrado como comprador o no tiene compras en ese estado
        #[ink(message)]
        pub fn ver_compras_categoria(&self, categoria: CategoriaProducto) -> Result<Vec<Compra>, ErrorVerCompras> {
            self._ver_compras_categoria(self.env().caller(), categoria)
        }

        //
        // compras.rs: visualizar ventas    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        //
        
        /// Devuelve las ventas del usuario que lo ejecuta
        /// 
        /// Dará error si el usuario no está registrado como vendedor o no tiene ventas
        #[ink(message)]
        pub fn ver_ventas(&self) -> Result<Vec<Compra>, ErrorVerVentas> {
            self._ver_ventas(self.env().caller())
        }

        /// Devuelve las ventas del usuario que lo ejecuta que estén en el estado especificado
        /// 
        /// Dará error si el usuario no está registrado como vendedor o no tiene ventas en ese estado
        #[ink(message)]
        pub fn ver_ventas_estado(&self, estado: EstadoCompra) -> Result<Vec<Compra>, ErrorVerVentas> {
            self._ver_ventas_estado(self.env().caller(), estado)
        }

        /// Devuelve las ventas del usuario que lo ejecuta que estén en el estado especificado
        /// 
        /// Dará error si el usuario no está registrado como vendedor o no tiene ventas en ese estado
        #[ink(message)]
        pub fn ver_ventas_categoria(&self, categoria: CategoriaProducto) -> Result<Vec<Compra>, ErrorVerVentas> {
            self._ver_ventas_categoria(self.env().caller(), categoria)
        }

        /// Ver la calificación histórica promedio del usuario como comprador.
        /// 
        /// Devolverá None si no es comprador o no tiene calificaciones.
        #[ink(message)]
        pub fn ver_calificacion_comprador(&self) -> Option<u8> {
            self._ver_calificacion_comprador(self.env().caller())
        }

        /// Ver la calificación histórica promedio del usuario como vendedor.
        /// 
        /// Devolverá None si no es vendedor o no tiene calificaciones.
        #[ink(message)]
        pub fn ver_calificacion_vendedor(&self) -> Option<u8> {
            self._ver_calificacion_vendedor(self.env().caller())
        }

////////////////////////////////////////////////////////////////////////////////

        /// Devuelve la siguiente ID disponible para compras
        /// 
        /// Si la próxima ID causaría Overflow, devuelve 0 y reinicia la cuenta.
        pub fn next_id_compras(&mut self) -> u128 {
            let id = self.compras_siguiente_id; // obtener actual
            let add_res = self.compras_siguiente_id.checked_add(1); // sumarle 1 al actual para que apunte a un id desocupado
            
            let Some(add_res) = add_res
            else {
                self.compras_siguiente_id = 1;
                return 0;
            };

            self.compras_siguiente_id = add_res;
            id // devolver
        }

        /// Devuelve la siguiente ID disponible para compras
        /// 
        /// Si la próxima ID causaría Overflow, devuelve 0 y reinicia la cuenta.
        pub fn next_id_productos(&mut self) -> u128 {
            let id = self.productos_siguiente_id; // obtener actual
            let add_res = self.productos_siguiente_id.checked_add(1); // sumarle 1 al actual para que apunte a un id desocupado
            
            let Some(add_res) = add_res
            else {
                self.productos_siguiente_id = 1;
                return 0;
            };

            self.productos_siguiente_id = add_res;
            id // devolver
        }

        /// Devuelve la siguiente ID disponible para publicaciones
        /// 
        /// Si la próxima ID causaría Overflow, devuelve 0 y reinicia la cuenta.
        pub fn next_id_publicaciones(&mut self) -> u128 {
            let id = self.publicaciones_siguiente_id; // obtener actual
            let add_res = self.publicaciones_siguiente_id.checked_add(1); // sumarle 1 al actual para que apunte a un id desocupado
            
            let Some(add_res) = add_res
            else {
                self.publicaciones_siguiente_id = 1;
                return 0;
            };

            self.publicaciones_siguiente_id = add_res;
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
}