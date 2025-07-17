#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod structs;

#[allow(non_local_definitions)] // error molesto
#[ink::contract]
mod rustaceo_libre {
    use ink::{
        prelude::vec::Vec,
        prelude::string::String,
        prelude::collections::BTreeMap,
        storage::Mapping,
    };

    //
    // imports propios
    //

    use crate::structs::usuario::{
        Usuario,
        StockProductos,
        RolDeSeleccion,
        ErrorAscenderRolUsuario,
        ErrorRegistrarUsuario,
    };

    use crate::structs::producto::{
        CategoriaProducto, ErrorIngresarStockProducto, ErrorRegistrarProducto, ErrorRetirarStockProducto, ErrorVerStockPropio, Producto
    };

    use crate::structs::publicacion::{
        Publicacion,
        ErrorModificarCantidadOfertada,
        ErrorVerPublicacionesVendedor,
        ErrorRealizarPublicacion,
    };

    use crate::structs::pedido::{
        Pedido,
        EstadoPedido,
        ErrorProductoDespachado,
        ErrorProductoRecibido,
        ErrorCalificarPedido,
        ErrorComprarProducto,
        ErrorCancelarPedido,
        ErrorReclamarFondos,
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
        pub pedidos: BTreeMap<u128, Pedido>,
        /// <ID, Producto>
        pub productos: BTreeMap<u128, Producto>,
        /// <ID, Publicacion>
        pub publicaciones: BTreeMap<u128, Publicacion>,
        /// Lleva un recuento de la próxima ID disponible para las compras.
        pedidos_siguiente_id: u128,
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
                pedidos: Default::default(),
                productos: Default::default(),
                publicaciones: Default::default(),
                pedidos_siguiente_id: 0,
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
        pub fn registrar_usuario(&mut self, rol: RolDeSeleccion) -> Result<(), ErrorRegistrarUsuario>  {
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

        /// Dada la ID de un producto y un stock, incrementa la posesión en stock de ese producto del vendedor.
        /// 
        /// Devolverá la nueva cantidad de stock disponible de ese producto para el vendedor.
        /// Devolverá error si la cantidad ingresada es cero, el usuario no está registrado,
        /// no es vendedor o el producto no existe.
        #[ink(message)]
        pub fn ingresar_stock_producto(&mut self, id_producto: u128, cantidad_ingresada: u32) -> Result<u32, ErrorIngresarStockProducto> {
            self._ingresar_stock_producto(self.env().caller(), id_producto, cantidad_ingresada)
        }

        /// Dada la ID de un producto y un stock, decrementa la posesión en stock de ese producto del vendedor.
        /// 
        /// Devolverá la nueva cantidad de stock disponible de ese producto para el vendedor.
        /// Devolverá error si la cantidad ingresada es cero, el usuario no está registrado,
        /// no es vendedor o el producto no existe.
        #[ink(message)]
        pub fn retirar_stock_producto(&mut self, id_producto: u128, cantidad_retirada: u32) -> Result<u32, ErrorRetirarStockProducto> {
            self._retirar_stock_producto(self.env().caller(), id_producto, cantidad_retirada)
        }
        
        /// Dada una ID, devuelve la publicación del producto
        /// 
        /// Devuelve None si el producto no existe
        #[ink(message)]
        pub fn ver_producto(&self, id_producto: u128) -> Option<Producto> {
            self._ver_producto(self.env().caller(), id_producto)
        }

        /// Devuelve el listado de stock del vendedor que llame la función
        /// 
        /// Dará error si el usuario no está registrado, no es vendedor o no posee stock de ningún producto
        #[ink(message)]
        pub fn ver_stock_propio(&self) -> Result<StockProductos, ErrorVerStockPropio> {
            self._ver_stock_propio(self.env().caller())
        }
        
        //
        // pedido.rs: administrar compras    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        //

        /// Compra una cantidad de un producto
        /// 
        /// Puede dar error si el usuario no existe, no es comprador, la publicación no existe,
        /// el stock es insuficiente o el vendedor de la misma no existe.
        #[ink(message, payable)]
        pub fn comprar_producto(&mut self, id_publicacion: u128, cantidad: u32) -> Result<u128, ErrorComprarProducto> {
            let operacion = self._comprar_producto(self.env().block_timestamp(), self.env().caller(), id_publicacion, cantidad, self.env().transferred_value());

            if let Ok(operacion) = operacion {
                // devolver fondos sobrantes. el checkeo tal vez es innecesario pero por si acaso
                if operacion.monto_transferido_sobrante > 0 {
                    let _ = self.env().transfer(self.env().caller(), operacion.monto_transferido_sobrante);
                }

                Ok(operacion.id_nueva_transaccion)
            } else {
                // fallo: devolver totalidad de los fondos transferidos
                let _ = self.env().transfer(self.env().caller(), self.env().transferred_value());
                Err(operacion.unwrap_err())
            }
        }

        /// Política de reclamo:
        /// 
        /// Si el vendedor despachó el producto y el comprador no lo marcó como recibido después de 60 días,
        /// el vendedor puede reclamar los fondos del pedido y la misma se marcará automáticamente como recibida,
        /// sin necesidad de consentimiento ni voluntad del comprador.
        /// 
        /// Puede dar error si el usuario no está registrado, la transacción no existe,
        /// el usuario no es el vendedor de la publicación o el tiempo pasado no condice con la política de reclamo
        #[ink(message)]
        pub fn reclamar_fondos(&mut self, id_compra: u128) -> Result<u128, ErrorReclamarFondos> {
            let operacion = self._reclamar_fondos(self.env().block_timestamp(), self.env().caller(), id_compra);
            
            let Ok(valor) = operacion
            else { return operacion };

            let _ = self.env().transfer(self.env().caller(), valor);

            operacion
        }

        /// Si el pedido indicada está pendiente y el usuario es el vendedor, se establece como recibida.
        /// 
        /// Puede dar error si el usuario no está registrado, el pedido no existe,
        /// no está pendiente, ya fue recibido, no es el vendedor quien intenta despacharlo
        /// o ya fue cancelado.
        #[ink(message)]
        pub fn pedido_despachado(&mut self, compra_id: u128) -> Result<(), ErrorProductoDespachado> {
            self._pedido_despachado(self.env().block_timestamp(), self.env().caller(), compra_id)
        }
        
        /// Si el pedido indicado fue despachado y el usuario es el comprador, se establece como recibido.
        /// 
        /// Puede dar error si el usuario no está registrado, la compra no existe,
        /// la compra no fue despachada, ya fue recibida, no es el comprador quien intenta recibirlo
        /// o ya fue cancelado.
        #[ink(message)]
        pub fn pedido_recibido(&mut self, id_compra: u128) -> Result<(), ErrorProductoRecibido> {
            let operacion = self._pedido_recibido(self.env().block_timestamp(), self.env().caller(), id_compra);

            let Ok((vendedor, valor)) = operacion
            else { return Err(operacion.unwrap_err()) };

            let _ = self.env().transfer(vendedor, valor);

            Ok(())
        }

        /// Dada una ID de pedido y una calificación (1..=5), se califica el mismo.
        /// Sólo se puede calificar una vez y sólo pueden calificar el comprador y vendedor de un pedido.
        /// 
        /// Devolverá error si el usuario no está registrado, la calificación no es válida (1..=5),
        /// la transacción no existe, no fue recibida o el usuario ya calificó esta transacción,
        #[ink(message)]
        pub fn calificar_pedido(&mut self, id_compra: u128, calificacion: u8) -> Result<(), ErrorCalificarPedido> {
            self._calificar_pedido(self.env().caller(), id_compra, calificacion)
        }
        
        /// Cancela el pedido si ambos participantes del mismo ejecutan esta misma función
        /// y si éste no fue recibida ni ya cancelada.
        /// Entrega automáticamente los fondos de la compra al comprador y el stock al vendedor.
        /// 
        /// Devuelve error si el usuario o pedido no existen, si el usuario no participa en el pedido,
        /// si el pedido ya fue cancelado o recibido y si quien solicita la cancelación ya la solicitó antes.
        #[ink(message)]
        pub fn cancelar_pedido(&mut self, id_compra: u128) -> Result<bool, ErrorCancelarPedido> {
            let operacion = self._cancelar_pedido(self.env().block_timestamp(), self.env().caller(), id_compra);

            let Ok(operacion) = operacion
            else { return Err(operacion.unwrap_err()) };

            let Some((comprador, valor)) = operacion
            else { return Ok(false) };

            let _ = self.env().transfer(comprador, valor);

            Ok(true)
        }

        //
        // pedido.rs: visualizar compras    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        //

        /// Devuelve las compras del usuario que lo ejecuta
        /// 
        /// Dará error si el usuario no está registrado como comprador o no tiene compras
        #[ink(message)]
        pub fn ver_compras(&self) -> Result<Vec<Pedido>, ErrorVerCompras> {
            self._ver_compras(self.env().caller())
        }

        /// Devuelve las compras del usuario que lo ejecuta que estén en el estado especificado
        /// 
        /// Dará error si el usuario no está registrado como comprador o no tiene compras en ese estado
        #[ink(message)]
        pub fn ver_compras_estado(&self, estado: EstadoPedido) -> Result<Vec<Pedido>, ErrorVerCompras> {
            self._ver_compras_estado(self.env().caller(), estado)
        }

        /// Devuelve las compras del usuario que lo ejecuta que estén en el estado especificado
        /// 
        /// Dará error si el usuario no está registrado como comprador o no tiene compras en ese estado
        #[ink(message)]
        pub fn ver_compras_categoria(&self, categoria: CategoriaProducto) -> Result<Vec<Pedido>, ErrorVerCompras> {
            self._ver_compras_categoria(self.env().caller(), categoria)
        }

        //
        // pedido.rs: visualizar ventas    ///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        //
        
        /// Devuelve las ventas del usuario que lo ejecuta
        /// 
        /// Dará error si el usuario no está registrado como vendedor o no tiene ventas
        #[ink(message)]
        pub fn ver_ventas(&self) -> Result<Vec<Pedido>, ErrorVerVentas> {
            self._ver_ventas(self.env().caller())
        }

        /// Devuelve las ventas del usuario que lo ejecuta que estén en el estado especificado
        /// 
        /// Dará error si el usuario no está registrado como vendedor o no tiene ventas en ese estado
        #[ink(message)]
        pub fn ver_ventas_estado(&self, estado: EstadoPedido) -> Result<Vec<Pedido>, ErrorVerVentas> {
            self._ver_ventas_estado(self.env().caller(), estado)
        }

        /// Devuelve las ventas del usuario que lo ejecuta que estén en el estado especificado
        /// 
        /// Dará error si el usuario no está registrado como vendedor o no tiene ventas en ese estado
        #[ink(message)]
        pub fn ver_ventas_categoria(&self, categoria: CategoriaProducto) -> Result<Vec<Pedido>, ErrorVerVentas> {
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

        /// Devuelve la siguiente ID disponible para pedidos
        /// 
        /// Si la próxima ID causaría Overflow, devuelve 0 y reinicia la cuenta.
        pub fn next_id_pedidos(&mut self) -> u128 {
            let id = self.pedidos_siguiente_id; // obtener actual
            let add_res = self.pedidos_siguiente_id.checked_add(1); // sumarle 1 al actual para que apunte a un id desocupado
            
            let Some(add_res) = add_res
            else {
                self.pedidos_siguiente_id = 1;
                return 0;
            };

            self.pedidos_siguiente_id = add_res;
            id // devolver
        }

        /// Devuelve la siguiente ID disponible para productos
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

        #[ink::test]
        fn new_works() {
            let rustaceo_libre = RustaceoLibre::new();
            assert_eq!(rustaceo_libre.pedidos_siguiente_id, 0);
        }

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let rustaceo_libre = RustaceoLibre::default();
            assert_eq!(rustaceo_libre.pedidos_siguiente_id, 0);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn next_id_works() {
            let mut rustaceo_libre = RustaceoLibre::new();
            assert_eq!(rustaceo_libre.next_id_pedidos(), 0);
            assert_eq!(rustaceo_libre.next_id_pedidos(), 1);
            assert_eq!(rustaceo_libre.next_id_productos(), 0);
            assert_eq!(rustaceo_libre.next_id_productos(), 1);
            assert_eq!(rustaceo_libre.next_id_publicaciones(), 0);
            assert_eq!(rustaceo_libre.next_id_publicaciones(), 1);

            rustaceo_libre.pedidos_siguiente_id = u128::MAX;
            rustaceo_libre.productos_siguiente_id = u128::MAX;
            rustaceo_libre.publicaciones_siguiente_id = u128::MAX;

            assert_eq!(rustaceo_libre.next_id_pedidos(), 0);
            assert_eq!(rustaceo_libre.next_id_productos(), 0);
            assert_eq!(rustaceo_libre.next_id_publicaciones(), 0);
        }
    }
}