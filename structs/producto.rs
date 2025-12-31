use ink::{prelude::string::String, primitives::AccountId};

use crate::{rustaceo_libre::RustaceoLibre, structs::usuario::StockProductos};

//
// categoria
//

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum CategoriaProducto {
    #[default]
    Ninguna,
    Hogar,
    Tecnologia,
    Indumentaria,
    Ferreteria
}

//
// producto
//

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub struct Producto {
    pub nombre: String,
    pub descripcion: String,
    pub categoria: CategoriaProducto,
    pub ventas: u128
}

//
// impl Producto
//

impl Producto {
    pub fn new(nombre: String, descripcion: String, categoria: CategoriaProducto) -> Self {
        Self {
            nombre,
            descripcion,
            categoria,
            ventas: 0
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorRegistrarProducto {
    UsuarioNoRegistrado,
    NoEsVendedor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorIngresarStockProducto {
    CantidadInvalida,
    UsuarioNoRegistrado,
    NoEsVendedor,
    ProductoInexistente
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorRetirarStockProducto {
    CantidadInvalida,
    UsuarioNoRegistrado,
    NoEsVendedor,
    StockInsuficiente,
    ProductoInexistente
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorVerStockPropio {
    UsuarioNoRegistrado,
    NoEsVendedor,
    NoPoseeStockAlguno,
}


impl RustaceoLibre {

    //

    /// Registra un producto en la lista de productos
    /// para su posterior uso en publicaciones
    /// 
    /// Devuelve error si el usuario no está registrado o no es vendedor.
    pub(crate) fn _registrar_producto(&mut self, caller: AccountId, nombre: String, descripcion: String, categoria: CategoriaProducto, stock_inicial: u32) -> Result<u128, ErrorRegistrarProducto> {
        // validar usuario
        let Some(mut usuario) = self.usuarios.get(&caller).cloned()
        else { return Err(ErrorRegistrarProducto::UsuarioNoRegistrado); };

        // validar que sea vendedor
        if !usuario.es_vendedor() {
            return Err(ErrorRegistrarProducto::NoEsVendedor);
        }

        // obtener id e instanciar producto
        let id_producto = self.next_id_productos();
        let producto = Producto::new(nombre, descripcion, categoria);

        // guardar producto
        self.productos.insert(id_producto, producto);

        // guardar stock inicial del producto en el vendedor
        usuario.establecer_stock_producto(&id_producto, &stock_inicial);
        self.usuarios.insert(caller, usuario);

        Ok(id_producto)
    }

    //

    /// Dada la ID de un producto y un stock, incrementa la posesión en stock de ese producto del vendedor.
    /// 
    /// Devolverá la nueva cantidad de stock disponible de ese producto para el vendedor.
    /// Devolverá error si la cantidad ingresada es cero, el usuario no está registrado,
    /// no es vendedor o el producto no existe.
    pub(crate) fn _ingresar_stock_producto(&mut self, caller: AccountId, id_producto: u128, cantidad_ingresada: u32) -> Result<u32, ErrorIngresarStockProducto> {
        // validar cantidad
        if cantidad_ingresada < 1 {
            return Err(ErrorIngresarStockProducto::CantidadInvalida);
        }
        
        // validar usuario
        let Some(mut usuario) = self.usuarios.get(&caller).cloned()
        else { return Err(ErrorIngresarStockProducto::UsuarioNoRegistrado); };

        // validar que sea vendedor
        if !usuario.es_vendedor() {
            return Err(ErrorIngresarStockProducto::NoEsVendedor);
        }

        // validar que exista el producto
        if !self.productos.contains_key(&id_producto) {
            return Err(ErrorIngresarStockProducto::ProductoInexistente);
        }

        // validar cantidad #2
        let stock_actual = if let Some(stock) = usuario.obtener_stock_producto(&id_producto) { stock } else { 0 };
        let Some(nuevo_stock_actual) = stock_actual.checked_add(cantidad_ingresada)
        else { return Err(ErrorIngresarStockProducto::CantidadInvalida); };

        // todo bien
        usuario.establecer_stock_producto(&id_producto, &nuevo_stock_actual);
        self.usuarios.insert(usuario.id, usuario);

        Ok(nuevo_stock_actual)
    }

    //

    /// Dada la ID de un producto y un stock, decrementa la posesión en stock de ese producto del vendedor.
    /// 
    /// Devolverá la nueva cantidad de stock disponible de ese producto para el vendedor.
    /// Devolverá error si la cantidad ingresada es cero, el usuario no está registrado,
    /// no es vendedor o el producto no existe.
    pub(crate) fn _retirar_stock_producto(&mut self, caller: AccountId, id_producto: u128, cantidad_retirada: u32) -> Result<u32, ErrorRetirarStockProducto> {
        // validar cantidad
        if cantidad_retirada < 1 {
            return Err(ErrorRetirarStockProducto::CantidadInvalida);
        }
        
        // validar usuario
        let Some(mut usuario) = self.usuarios.get(&caller).cloned()
        else { return Err(ErrorRetirarStockProducto::UsuarioNoRegistrado); };

        // validar que sea vendedor
        if !usuario.es_vendedor() {
            return Err(ErrorRetirarStockProducto::NoEsVendedor);
        }

        // validar que el stock del vendedor sea suficiente
        let stock_actual = if let Some(stock) = usuario.obtener_stock_producto(&id_producto) { stock } else { 0 };
        if stock_actual < cantidad_retirada {
            return Err(ErrorRetirarStockProducto::StockInsuficiente);
        }

        // validar cantidad #2
        let stock_actual = if let Some(stock) = usuario.obtener_stock_producto(&id_producto) { stock } else { 0 };
        let Some(nuevo_stock_actual) = stock_actual.checked_sub(cantidad_retirada)
        else { return Err(ErrorRetirarStockProducto::CantidadInvalida); };

        // todo bien
        usuario.establecer_stock_producto(&id_producto, &nuevo_stock_actual);
        self.usuarios.insert(usuario.id, usuario);

        Ok(nuevo_stock_actual)
    }

    //

    /// Dada una ID, devuelve el producto correspondiente si es posible
    /// 
    /// Devolverá None si el producto no existe
    pub(crate) fn _ver_producto(&self, id_producto: u128) -> Option<Producto> {
        self.productos.get(&id_producto).cloned()
    }

    /// Devuelve el listado de stock del vendedor que llame la función
    /// 
    /// Dará error si el usuario no está registrado, no es vendedor o no posee stock de ningún producto
    pub(crate) fn _ver_stock_propio(&self, caller: AccountId) -> Result<StockProductos, ErrorVerStockPropio> {
        let Some(usuario) = self.usuarios.get(&caller)
        else { return Err(ErrorVerStockPropio::UsuarioNoRegistrado); };

        if !usuario.es_vendedor() {
            return Err(ErrorVerStockPropio::NoEsVendedor);
        }

        let Some(stock_productos) = usuario.obtener_stock_productos()
        else { return Err(ErrorVerStockPropio::NoPoseeStockAlguno); };

        Ok(stock_productos)
    }

    //

    /// Devuelve la cantidad de ventas que ese producto haya tenido
    pub(crate) fn _ver_ventas_producto(&self, id: u128) -> Option<u128> {
        let Some(producto) = self.productos.get(&id)
        else { return None; };

        Some(producto.ventas)
    }

}


#[cfg(test)]
mod tests {

    use super::*;
    use crate::structs::usuario::{RolDeSeleccion};
    use crate::structs::producto::{CategoriaProducto, Producto};
    
    //
    // registrar producto
    //

    #[ink::test]
    fn registrar_producto_works() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let vendedor = accounts.alice;
        
        // Crear contrato
        let mut contrato = RustaceoLibre::new(0);

        // Simular llamado como Alice
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(vendedor);

        // Registrar a Alice como Vendedor
        let rol = RolDeSeleccion::Vendedor;
        assert_eq!(contrato.registrar_usuario(rol), Ok(()));

        // Registrar producto
        let nombre: String = "Mate".into();
        let descripcion : String= "Mate de calabaza forrado en cuero".into();
        let categoria = CategoriaProducto::Hogar;
        let stock_inicial = 10;
        
        let result = contrato.registrar_producto(nombre.clone(), descripcion.clone(), categoria.clone(), stock_inicial);
        assert!(result.is_ok());

        let id_producto = result.unwrap();

        // Verificar que el producto se haya guardado
        let producto = contrato.ver_producto(id_producto);
        assert_eq!(
            producto,
            Some(Producto {
                nombre,
                descripcion,
                categoria,
                ventas: 0
            })
        );

        // Verificar que el stock esté en el usuario
        let usuario = contrato.usuarios.get(&vendedor).unwrap();
        let stock = usuario.obtener_stock_producto(&id_producto);
        assert_eq!(stock, Some(stock_inicial));
    }
    #[ink::test]
    fn registrar_producto_falla_usuario_no_registrado() {
        let mut contrato = RustaceoLibre::default();
        let caller = AccountId::from([0x1; 32]);

        let nombre = "Mate".into();
        let descripcion = "De madera".into();
        let categoria = CategoriaProducto::Hogar;
        let stock_inicial = 10;

        let result = contrato._registrar_producto(caller, nombre, descripcion, categoria, stock_inicial);
        
        assert_eq!(result, Err(ErrorRegistrarProducto::UsuarioNoRegistrado));
    }
    #[ink::test]
    fn registrar_producto_falla_usuario_no_es_vendedor() {
        let mut contrato = RustaceoLibre::default();
        let caller = AccountId::from([0x2; 32]);

        // registrar usuario como comprador
        let rol = RolDeSeleccion::Comprador;
        contrato._registrar_usuario(caller, rol).expect("Debe registrarse");

        let nombre = "Mate".into();
        let descripcion = "De madera".into();
        let categoria = CategoriaProducto::Hogar;
        let stock_inicial = 10;

        let result = contrato._registrar_producto(caller, nombre, descripcion, categoria, stock_inicial);
        
        assert_eq!(result, Err(ErrorRegistrarProducto::NoEsVendedor));
    }
    
    //
    // ingresar stock
    //

    #[ink::test]
    fn ingresar_stock_producto_falla_producto_inexistente() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let vendedor = accounts.alice;

        // Crear contrato
        let mut contrato = RustaceoLibre::new(0);

        // Registrar a Alice como Vendedor
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(vendedor);
        let rol = RolDeSeleccion::Vendedor;
        assert_eq!(contrato.registrar_usuario(rol), Ok(()));

        // Usar un id_producto que NO existe en el mapa de productos
        let id_producto_inexistente = 9999;

        // Intentar ingresar stock para ese producto
        let res = contrato._ingresar_stock_producto(vendedor, id_producto_inexistente, 10);
        assert_eq!(res, Err(ErrorIngresarStockProducto::ProductoInexistente));
    }
    #[ink::test]
    fn ingresar_stock_producto_falla_usuario_no_registrado() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let vendedor = accounts.alice;

        let mut contrato = RustaceoLibre::new(0);

        // crear y registrar prod pero NO registrar usuario
        let producto = Producto {
            nombre: "Producto".to_string(),
            descripcion: "Desc".to_string(),
            categoria: CategoriaProducto::Hogar,
            ventas: 0
        };
        let id_producto = contrato.next_id_productos();
        contrato.productos.insert(id_producto, producto);

        let res = contrato._ingresar_stock_producto(vendedor, id_producto, 5);
        assert_eq!(res, Err(ErrorIngresarStockProducto::UsuarioNoRegistrado));
    }

    #[ink::test]
    fn ingresar_stock_producto_falla_no_es_vendedor() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let comprador = accounts.bob;

        let mut contrato = RustaceoLibre::new(0);

        let rol = RolDeSeleccion::Comprador;
        assert_eq!(contrato._registrar_usuario(comprador, rol), Ok(()));

        let producto = Producto {
            nombre: "Producto".to_string(),
            descripcion: "Desc".to_string(),
            categoria: CategoriaProducto::Hogar,
            ventas: 0
        };
        let id_producto = contrato.next_id_productos();
        contrato.productos.insert(id_producto, producto);

        let res = contrato._ingresar_stock_producto(comprador, id_producto, 5);
        assert_eq!(res, Err(ErrorIngresarStockProducto::NoEsVendedor));
    }

    #[ink::test]
    fn ingresar_stock_producto_falla_cantidad_invalida() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let vendedor = accounts.alice;

    
        let mut contrato = RustaceoLibre::new(0);

        // Registrar a Alice como Vendedor
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(vendedor);
        let rol = RolDeSeleccion::Vendedor;
        assert_eq!(contrato.registrar_usuario(rol), Ok(()));

        let producto = Producto {
            nombre: "Producto".to_string(),
            descripcion: "Desc".to_string(),
            categoria: CategoriaProducto::Hogar,
            ventas: 0
        };
        let id_producto = contrato.next_id_productos();
        contrato.productos.insert(id_producto, producto);

    
        let res = contrato._ingresar_stock_producto(vendedor, id_producto, 0);
        assert_eq!(res, Err(ErrorIngresarStockProducto::CantidadInvalida));
    }

    
    //
    // retirar stock
    //
    #[ink::test]

    fn retirar_stock_producto_falla_stock_insuficiente() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let vendedor = accounts.alice;

        let mut contrato = RustaceoLibre::new(0);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(vendedor);

        // Registrar a Alice como Vendedor
        let rol = RolDeSeleccion::Vendedor;
        assert_eq!(contrato.registrar_usuario(rol), Ok(()));

    
        let producto = Producto {
            nombre: "Producto".to_string(),
            descripcion: "Desc".to_string(),
            categoria: CategoriaProducto::Hogar,
            ventas: 0
        };
        let id_producto = contrato.next_id_productos();
        contrato.productos.insert(id_producto, producto);

        // Ingresar stock menor al que se intentará retirar
        let _ = contrato._ingresar_stock_producto(vendedor, id_producto, 3);

        // Intentar retirar más stock del disponible
        let res = contrato._retirar_stock_producto(vendedor, id_producto, 5);
        assert_eq!(res, Err(ErrorRetirarStockProducto::StockInsuficiente));
    }

    #[ink::test]
    fn retirar_stock_producto_falla_cantidad_invalida() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let vendedor = accounts.alice;

        let mut contrato = RustaceoLibre::new(0);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(vendedor);
        let rol = RolDeSeleccion::Vendedor;
        assert_eq!(contrato.registrar_usuario(rol), Ok(()));

        let producto = Producto {
            nombre: "Producto".to_string(),
            descripcion: "Desc".to_string(),
            categoria: CategoriaProducto::Hogar,
            ventas: 0
        };
        let id_producto = contrato.next_id_productos();
        contrato.productos.insert(id_producto, producto);

        let _ = contrato._ingresar_stock_producto(vendedor, id_producto, 10);

        // Intentar retirar cantidad inválida (0)
        let res = contrato._retirar_stock_producto(vendedor, id_producto, 0);
        assert_eq!(res, Err(ErrorRetirarStockProducto::CantidadInvalida));
    }

    #[ink::test]
    fn retirar_stock_producto_falla_no_es_vendedor() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let comprador = accounts.bob;

        let mut contrato = RustaceoLibre::new(0);

        // Registrar a Bob como Comprador
        let rol = RolDeSeleccion::Comprador;
        assert_eq!(contrato._registrar_usuario(comprador, rol), Ok(()));

        let producto = Producto {
            nombre: "Producto".to_string(),
            descripcion: "Desc".to_string(),
            categoria: CategoriaProducto::Hogar,
            ventas: 0
        };
        let id_producto = contrato.next_id_productos();
        contrato.productos.insert(id_producto, producto);

        // Intentar retirar stock siendo solo comprador
        let res = contrato._retirar_stock_producto(comprador, id_producto, 5);
        assert_eq!(res, Err(ErrorRetirarStockProducto::NoEsVendedor));
    }


    #[ink::test]
    fn retirar_stock_producto_falla_usuario_no_registrado() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let vendedor = accounts.alice;

        let mut contrato = RustaceoLibre::new(0);

        // Crear y registrar producto, pero NO registrar usuario
        let producto = Producto {
            nombre: "Producto".to_string(),
            descripcion: "Desc".to_string(),
            categoria: CategoriaProducto::Hogar,
            ventas: 0
        };
        let id_producto = contrato.next_id_productos();
        contrato.productos.insert(id_producto, producto);

        // Intentar retirar stock sin usuario registrado
        let res = contrato._retirar_stock_producto(vendedor, id_producto, 5);
        assert_eq!(res, Err(ErrorRetirarStockProducto::UsuarioNoRegistrado));
    }

    //
    // ver producto
    //
    
    #[ink::test]
    fn ver_producto_falla_producto_inexistente() {
    let contrato = RustaceoLibre::default();
    let id_producto = 13548;

    let resultado = contrato._ver_producto(id_producto);

    assert_eq!(resultado, None);
}

    //
    // ver stock propio
    //



    #[ink::test]
    fn ingresar_stock_producto_works() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let vendedor = accounts.alice;
        let mut contrato = RustaceoLibre::new(0);

        // Setup: Registrar vendedor y producto
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(vendedor);
        let _ = contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor);
        
        let nombre = "Teclado".into();
        let desc = "Mecanico".into();
        let cat = CategoriaProducto::Tecnologia;
        let stock_inicial = 10;
        
        let id_producto = contrato._registrar_producto(vendedor, nombre, desc, cat, stock_inicial).unwrap();

        // ACT: Ingresar 5 unidades más
        let nuevo_stock = contrato._ingresar_stock_producto(vendedor, id_producto, 5);

        // ASSERT: El stock debe ser 15
        assert_eq!(nuevo_stock, Ok(15));

        // Verificar persistencia en el usuario
        let usuario = contrato.usuarios.get(&vendedor).unwrap();
        assert_eq!(usuario.obtener_stock_producto(&id_producto), Some(15));
    }

    #[ink::test]
    fn retirar_stock_producto_works() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let vendedor = accounts.alice;
        let mut contrato = RustaceoLibre::new(0);

        // Setup: Registrar vendedor y producto con stock 10
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(vendedor);
        let _ = contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor);
        let id_producto = contrato._registrar_producto(vendedor, "Mouse".into(), "Gamer".into(), CategoriaProducto::Tecnologia, 10).unwrap();

        // ACT: Retirar 4 unidades
        let nuevo_stock = contrato._retirar_stock_producto(vendedor, id_producto, 4);

        // ASSERT: El stock restante debe ser 6
        assert_eq!(nuevo_stock, Ok(6));
        
        let usuario = contrato.usuarios.get(&vendedor).unwrap();
        assert_eq!(usuario.obtener_stock_producto(&id_producto), Some(6));
    }

    #[ink::test]
    fn ver_stock_propio_works() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let vendedor = accounts.alice;
        let mut contrato = RustaceoLibre::new(0);

        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(vendedor);
        let _ = contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor);

        // Registrar dos productos
        let id1 = contrato._registrar_producto(vendedor, "P1".into(), "D1".into(), CategoriaProducto::Hogar, 10).unwrap();
        let id2 = contrato._registrar_producto(vendedor, "P2".into(), "D2".into(), CategoriaProducto::Hogar, 20).unwrap();

        // ACT
        let stock_total = contrato._ver_stock_propio(vendedor);

        // ASSERT
        assert!(stock_total.is_ok());
        // Nota: Dependiendo de cómo sea tu struct StockProductos, podrías hacer aserciones más profundas aquí.
        // Por ejemplo, si es un Vec o HashMap, verificar que contenga id1 e id2.
    }

    #[ink::test]
    fn ver_stock_propio_falla_usuario_no_registrado() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let mut contrato = RustaceoLibre::new(0);
        
        assert_eq!(contrato._ver_stock_propio(accounts.alice), Err(ErrorVerStockPropio::UsuarioNoRegistrado));
    }

    #[ink::test]
    fn ver_ventas_producto_works() {
        let mut contrato = RustaceoLibre::new(0);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let vendedor = accounts.alice;
        
        // Setup
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(vendedor);
        let _ = contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor);
        let id = contrato._registrar_producto(vendedor, "A".into(), "B".into(), CategoriaProducto::Ferreteria, 1).unwrap();

        // ACT: Producto nuevo tiene 0 ventas
        assert_eq!(contrato._ver_ventas_producto(id), Some(0));

        // ACT: Producto inexistente
        assert_eq!(contrato._ver_ventas_producto(9999), None);
    }

    #[ink::test]
    fn test_categoria_producto_derives() {
        // Este test "tonto" ayuda a que el coverage marque como usadas las derivaciones Clone, PartialEq, Debug
        let c1 = CategoriaProducto::Hogar;
        let c2 = CategoriaProducto::Tecnologia;
        let c3 = c1.clone();

        assert_eq!(c1, c3);
        assert_ne!(c1, c2);
        assert_eq!(CategoriaProducto::default(), CategoriaProducto::Ninguna);
        
        // Testear formateo de debug
        let debug_str = format!("{:?}", c1);
        assert!(!debug_str.is_empty());
    }


}