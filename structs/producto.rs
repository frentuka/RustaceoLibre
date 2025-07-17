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
    Hogar,
    Tecnologia,
    Ropa,
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
        let Some(mut usuario) = self.usuarios.get(caller)
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
        self.usuarios.insert(caller, &usuario);

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
        let Some(mut usuario) = self.usuarios.get(caller)
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
        self.usuarios.insert(usuario.id, &usuario);

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
        let Some(mut usuario) = self.usuarios.get(caller)
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
        self.usuarios.insert(usuario.id, &usuario);

        Ok(nuevo_stock_actual)
    }

    //

    /// Dada una ID, devuelve el producto correspondiente si es posible
    /// 
    /// Devolverá None si el producto no existe o el usuario no está registrado
    pub(crate) fn _ver_producto(&self, caller: AccountId, id_producto: u128) -> Option<Producto> {
        if !self.usuarios.contains(caller) {
            return None;
        }

        self.productos.get(&id_producto).cloned()
    }

    /// Devuelve el listado de stock del vendedor que llame la función
    /// 
    /// Dará error si el usuario no está registrado, no es vendedor o no posee stock de ningún producto
    pub(crate) fn _ver_stock_propio(&self, caller: AccountId) -> Result<StockProductos, ErrorVerStockPropio> {
        let Some(usuario) = self.usuarios.get(caller)
        else { return Err(ErrorVerStockPropio::UsuarioNoRegistrado); };

        if !usuario.es_vendedor() {
            return Err(ErrorVerStockPropio::NoEsVendedor);
        }

        let Some(stock_productos) = usuario.obtener_stock_productos()
        else { return Err(ErrorVerStockPropio::NoPoseeStockAlguno); };

        Ok(stock_productos)
    }

}


#[cfg(test)]
mod tests {

        use super::*;
        use crate::structs::usuario::{Rol, RolDeSeleccion};
        use crate::structs::producto::{CategoriaProducto, Producto};
    
    //
    // registrar producto
    //

    #[ink::test]
    fn registrar_producto_works() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let vendedor = accounts.alice;
        
        // Crear contrato
        let mut contrato = RustaceoLibre::new();

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
            })
        );

        // Verificar que el stock esté en el usuario
        let usuario = contrato.usuarios.get(vendedor).unwrap();
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
        let mut contrato = RustaceoLibre::new();

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

        let mut contrato = RustaceoLibre::new();

        // crear y registrar prod pero NO registrar usuario
        let producto = Producto {
            nombre: "Producto".to_string(),
            descripcion: "Desc".to_string(),
            categoria: CategoriaProducto::Hogar,
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

        let mut contrato = RustaceoLibre::new();

        let rol = RolDeSeleccion::Comprador;
        assert_eq!(contrato._registrar_usuario(comprador, rol), Ok(()));

        let producto = Producto {
            nombre: "Producto".to_string(),
            descripcion: "Desc".to_string(),
            categoria: CategoriaProducto::Hogar,
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

    
        let mut contrato = RustaceoLibre::new();

        // Registrar a Alice como Vendedor
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(vendedor);
        let rol = RolDeSeleccion::Vendedor;
        assert_eq!(contrato.registrar_usuario(rol), Ok(()));

        let producto = Producto {
            nombre: "Producto".to_string(),
            descripcion: "Desc".to_string(),
            categoria: CategoriaProducto::Hogar,
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

        let mut contrato = RustaceoLibre::new();
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(vendedor);

        // Registrar a Alice como Vendedor
        let rol = RolDeSeleccion::Vendedor;
        assert_eq!(contrato.registrar_usuario(rol), Ok(()));

    
        let producto = Producto {
            nombre: "Producto".to_string(),
            descripcion: "Desc".to_string(),
            categoria: CategoriaProducto::Hogar,
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

        let mut contrato = RustaceoLibre::new();
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(vendedor);
        let rol = RolDeSeleccion::Vendedor;
        assert_eq!(contrato.registrar_usuario(rol), Ok(()));

        let producto = Producto {
            nombre: "Producto".to_string(),
            descripcion: "Desc".to_string(),
            categoria: CategoriaProducto::Hogar,
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

        let mut contrato = RustaceoLibre::new();

        // Registrar a Bob como Comprador
        let rol = RolDeSeleccion::Comprador;
        assert_eq!(contrato._registrar_usuario(comprador, rol), Ok(()));

        let producto = Producto {
            nombre: "Producto".to_string(),
            descripcion: "Desc".to_string(),
            categoria: CategoriaProducto::Hogar,
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

        let mut contrato = RustaceoLibre::new();

        // Crear y registrar producto, pero NO registrar usuario
        let producto = Producto {
            nombre: "Producto".to_string(),
            descripcion: "Desc".to_string(),
            categoria: CategoriaProducto::Hogar,
        };
        let id_producto = contrato.next_id_productos();
        contrato.productos.insert(id_producto, producto);

        // Intentar retirar stock sin usuario registrado
        let res = contrato._retirar_stock_producto(vendedor, id_producto, 5);
        assert_eq!(res, Err(ErrorRetirarStockProducto::UsuarioNoRegistrado));
    }

    #[ink::test]
    fn ingresar_retirar_stock_productos_works() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let vendedor = accounts.alice;
        
        let mut contrato = RustaceoLibre::new();

        let producto = Producto {
            nombre: "asd".to_string(),
            descripcion: "asd".to_string(),
            categoria: CategoriaProducto::Hogar,
        };

        let id_producto = contrato.next_id_productos();
        contrato.productos.insert(id_producto, producto);

        // Simular llamado como Alice
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(vendedor);

        // Registrar a Alice como Vendedor
        let rol = RolDeSeleccion::Vendedor;
        assert_eq!(contrato.registrar_usuario(rol), Ok(()));


        let res = contrato.ingresar_stock_producto(id_producto, 13548);
        let Ok(res) = res
        else { panic!("res debería ser Ok"); };

        assert_eq!(res, 13548);

        let stock = contrato.ver_stock_propio();
        let Ok(stock) = stock else { panic!("stock debería ser Ok"); };

        let Some(stock) = stock.get(&id_producto)
        else { panic!("stock debería ser Ok") };

        assert_eq!(stock, 13548);


        let res = contrato.retirar_stock_producto(id_producto, 10000);
        let Ok(res) = res
        else { panic!("res debería ser Ok"); };

        assert_eq!(res, 3548);

        let stock = contrato.ver_stock_propio();
        let Ok(stock) = stock else { panic!("stock debería ser Ok"); };

        let Some(stock) = stock.get(&id_producto)
        else { panic!("stock debería ser Ok") };

        assert_eq!(stock, 3548);
    }

    //
    // ver producto
    //
    
    #[ink::test]
    fn ver_producto_falla_usuario_no_registrado() {
    let contrato = RustaceoLibre::default();
    let caller = AccountId::from([0x4; 32]);
    let id_producto = 1;

    let resultado = contrato._ver_producto(caller, id_producto);

    assert_eq!(resultado, None);
}

    //
    // ver stock propio
    //

}