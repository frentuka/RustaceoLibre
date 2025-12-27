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
        /// total de la tarifa: total_compra * tarifa_de_servicio / 1_000
        tarifa_de_servicio: u128,
        /// ID del dueño del contrato
        pub owner: AccountId,
    }

    #[ink(impl)]
    impl RustaceoLibre {
        /// Construye un nuevo contrato con sus valores por defecto
        #[ink(constructor)]
        pub fn new(tarifa_de_servicio: u128) -> Self {
            Self::_new(tarifa_de_servicio)
        }

        /// Constructor por defecto (Ídem new())
        /// La tarifa de servicio por defecto es 0
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::_new(0)
        }

        /// Crea una nueva instancia de RustaceoLibre
        fn _new(tarifa_de_servicio: u128) -> Self {
            Self {
                usuarios: Default::default(),
                pedidos: Default::default(),
                productos: Default::default(),
                publicaciones: Default::default(),
                pedidos_siguiente_id: 0,
                productos_siguiente_id: 0,
                publicaciones_siguiente_id: 0,
                tarifa_de_servicio,
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

        #[ink(message)]
        pub fn calcular_tarifa_de_servicio(&self, valor_compra: u128) -> u128 {
            self._calcular_tarifa_de_servicio(valor_compra)
        }

        fn _calcular_tarifa_de_servicio(&self, valor_compra: u128) -> u128 {
            let Some(tarifa_servicio) = valor_compra.checked_div(1000)
            else { return 0; };

            let Some(tarifa_servicio) = tarifa_servicio.checked_mul(self.tarifa_de_servicio)
            else { return 0; };

            tarifa_servicio
        }

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

            let tarifa_servicio = self._calcular_tarifa_de_servicio(valor);
            let Some(valor_final) = valor.checked_sub(tarifa_servicio)
            else { 
                let _ = self.env().transfer(self.env().caller(), valor);
                return operacion;
            };

            let _ = self.env().transfer(self.env().caller(), valor_final);

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
        /// El comprador recibirá los fondos descontando una tarifa por el servicio.
        /// 
        /// Puede dar error si el usuario no está registrado, la compra no existe,
        /// la compra no fue despachada, ya fue recibida, no es el comprador quien intenta recibirlo
        /// o ya fue cancelado.
        #[ink(message)]
        pub fn pedido_recibido(&mut self, id_compra: u128) -> Result<(), ErrorProductoRecibido> {
            let operacion = self._pedido_recibido(self.env().block_timestamp(), self.env().caller(), id_compra);

            let Ok((vendedor, valor)) = operacion
            else { return Err(operacion.unwrap_err()) };

            // descontar tarifa de servicio
            let tarifa_servicio = self._calcular_tarifa_de_servicio(valor);

            let Some(valor_final) = valor.checked_sub(tarifa_servicio)
            else {
                // desestimar tarifa de servicio por error al calcularla
                let _ = self.env().transfer(vendedor, valor);
                return Ok(());
            };

            let _ = self.env().transfer(vendedor, valor_final);

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
            let rustaceo_libre = RustaceoLibre::new(1000);
            assert_eq!(rustaceo_libre.pedidos_siguiente_id, 0);
            assert_eq!(rustaceo_libre._calcular_tarifa_de_servicio(1000), 1000);
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
            let mut rustaceo_libre = RustaceoLibre::new(0);

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

        use ink::env::test;
        use ink::env::DefaultEnvironment;

        fn set_caller(caller: AccountId) {
            test::set_caller::<DefaultEnvironment>(caller);
        }

        fn set_value_transferred(value: Balance) {
            test::set_value_transferred::<DefaultEnvironment>(value);
        }

        fn set_balance(who: AccountId, amount: Balance) {
            test::set_account_balance::<DefaultEnvironment>(who, amount);
        }

        fn balances_setup(accounts: &test::DefaultAccounts<DefaultEnvironment>) {
            // Dar balances grandes a todos, y al contrato también para que transfers no fallen por falta de fondos.
            set_balance(accounts.alice, 1_000_000_000_000);
            set_balance(accounts.bob, 1_000_000_000_000);
            set_balance(accounts.charlie, 1_000_000_000_000);
            set_balance(accounts.django, 1_000_000_000_000);

            let contract = test::callee::<DefaultEnvironment>();

            set_balance(contract, 1_000_000_000_000);
        }

        /// Escenario mínimo para cubrir wrappers de lib.rs:
        /// - alice: vendedor
        /// - bob: comprador
        /// - alice registra producto con stock y publica (precio y cantidad)
        /// Devuelve (contract, accounts, id_publicacion).
        fn setup_publicacion_basica() -> (RustaceoLibre, test::DefaultAccounts<DefaultEnvironment>, u128) {
            let accounts = test::default_accounts::<DefaultEnvironment>();
            balances_setup(&accounts);

            set_caller(accounts.alice);
            let mut c = RustaceoLibre::new(0);

            // registrar vendedor
            assert!(c.registrar_usuario(RolDeSeleccion::Vendedor).is_ok());

            // registrar producto con stock inicial
            let id_producto = c
                .registrar_producto(
                    "prod".into(),
                    "desc".into(),
                    CategoriaProducto::Hogar,
                    10,
                )
                .expect("registrar_producto debe funcionar");

            // realizar publicación: ofrece 5 unidades a precio 100
            let id_publicacion = c
                .realizar_publicacion(id_producto, 5, 100)
                .expect("realizar_publicacion debe funcionar");

            // registrar comprador (bob)
            set_caller(accounts.bob);
            assert!(c.registrar_usuario(RolDeSeleccion::Comprador).is_ok());

            (c, accounts, id_publicacion)
        }

        fn comprar_1(
            c: &mut RustaceoLibre,
            accounts: &test::DefaultAccounts<DefaultEnvironment>,
            id_publicacion: u128,
            valor_transferido: Balance,
        ) -> Result<u128, ErrorComprarProducto> {
            set_caller(accounts.bob);
            set_value_transferred(valor_transferido);
            c.comprar_producto(id_publicacion, 1)
        }

        #[ink::test]
        fn comprar_producto_err_cubre_rama_devolucion_total() {
            let (mut c, accounts, id_publicacion) = setup_publicacion_basica();

            // Forzamos error del wrapper: cantidad 0 => CantidadCero.
            // Además seteamos value_transferred para ejecutar la línea del transfer de devolución total.
            set_caller(accounts.bob);
            set_value_transferred(500);

            let res = c.comprar_producto(id_publicacion, 0);
            assert!(matches!(res, Err(ErrorComprarProducto::CantidadCero)));
        }

        #[ink::test]
        fn comprar_producto_ok_sin_sobrante_cubre_rama_ok() {
            let (mut c, accounts, id_publicacion) = setup_publicacion_basica();

            // precio unitario=100, cantidad=1 => transfer exacto: 100 => sobrante 0 (no entra al if)
            let id_pedido = comprar_1(&mut c, &accounts, id_publicacion, 100)
                .expect("compra ok");

            assert!(c.pedidos.contains_key(&id_pedido));
        }

        #[ink::test]
        fn comprar_producto_ok_con_sobrante_cubre_devolucion_sobrante() {
            let (mut c, accounts, id_publicacion) = setup_publicacion_basica();

            // total=100, transferimos 150 => sobrante 50 (cubre if monto_transferido_sobrante > 0)
            let id_pedido = comprar_1(&mut c, &accounts, id_publicacion, 150)
                .expect("compra ok con sobrante");

            assert!(c.pedidos.contains_key(&id_pedido));
        }

        #[ink::test]
        fn flujo_pendiente_despachado_recibido_via_messages() {
            let (mut c, accounts, id_publicacion) = setup_publicacion_basica();

            // comprar (crea pedido pendiente)
            let id_pedido = comprar_1(&mut c, &accounts, id_publicacion, 100)
                .expect("compra ok");

            // despachar por vendedor (message wrapper)
            set_caller(accounts.alice);
            assert!(c.pedido_despachado(id_pedido).is_ok());

            // recibir por comprador (message wrapper con transfer)
            set_caller(accounts.bob);
            assert!(c.pedido_recibido(id_pedido).is_ok());
        }

        #[ink::test]
        fn pedido_recibido_wrapper_err_solo_comprador_puede() {
            let (mut c, accounts, id_publicacion) = setup_publicacion_basica();

            let id_pedido = comprar_1(&mut c, &accounts, id_publicacion, 100)
                .expect("compra ok");

            set_caller(accounts.alice);
            assert!(c.pedido_despachado(id_pedido).is_ok());

            // Intentar recibir con el vendedor: debe fallar con SoloCompradorPuede (punto [14])
            set_caller(accounts.alice);
            let res = c.pedido_recibido(id_pedido);
            assert!(matches!(res, Err(ErrorProductoRecibido::SoloCompradorPuede)));
        }

        #[ink::test]
        fn cancelar_pedido_wrapper_false_y_true() {
            let (mut c, accounts, id_publicacion) = setup_publicacion_basica();

            let id_pedido = comprar_1(&mut c, &accounts, id_publicacion, 100)
                .expect("compra ok");

            // 1er llamada cancelar por VENDEDOR => no aplica política unilateral => Ok(false)
            set_caller(accounts.alice);
            let r1 = c.cancelar_pedido(id_pedido).expect("cancelar debe ok");
            assert_eq!(r1, false);

            // 2da llamada por COMPRADOR => aplica política unilateral => Ok(true)
            set_caller(accounts.bob);
            let r2 = c.cancelar_pedido(id_pedido).expect("cancelar debe ok");
            assert_eq!(r2, true);
        }


        #[ink::test]
        fn ver_compras_y_ventas_wrappers() {
            let (mut c, accounts, id_publicacion) = setup_publicacion_basica();

            let id_pedido = comprar_1(&mut c, &accounts, id_publicacion, 100)
                .expect("compra ok");

            // comprador ve compras
            set_caller(accounts.bob);
            let compras = c.ver_compras().expect("debe tener compras");
            assert!(compras.iter().any(|p| p.id == id_pedido));

            // vendedor ve ventas
            set_caller(accounts.alice);
            let ventas = c.ver_ventas().expect("debe tener ventas");
            assert!(ventas.iter().any(|p| p.id == id_pedido));
        }

    }
}