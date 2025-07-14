use ink::{prelude::string::String, primitives::AccountId};


use crate::rustaceo_libre::RustaceoLibre;

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

    /// Dada una ID, devuelve el producto si es posible
    /// 
    /// Devolverá None si el producto no existe o el usuario no está registrado
    pub(crate) fn _ver_producto(&self, caller: AccountId, id_producto: u128) -> Option<Producto> {
        if !self.usuarios.contains(caller) {
            return None;
        }

        self.productos.get(&id_producto).cloned()
    }
}


#[cfg(test)]
mod tests {
        use super::*;
        use crate::structs::usuario::Rol;
        use crate::structs::producto::{CategoriaProducto, Producto};
    #[ink::test]
    fn registrar_producto_works() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let vendedor = accounts.alice;
        
        // Crear contrato
        let mut contrato = RustaceoLibre::new();

        // Simular llamado como Alice
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(vendedor);

        // Registrar a Alice como Vendedor
        let rol = Rol::Vendedor(Default::default());
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
        let rol = Rol::Comprador(Default::default());
        contrato._registrar_usuario(caller, rol).expect("Debe registrarse");

        let nombre = "Mate".into();
        let descripcion = "De madera".into();
        let categoria = CategoriaProducto::Hogar;
        let stock_inicial = 10;

        let result = contrato._registrar_producto(caller, nombre, descripcion, categoria, stock_inicial);
        
        assert_eq!(result, Err(ErrorRegistrarProducto::NoEsVendedor));
    }
    #[ink::test]
    fn ver_producto_falla_usuario_no_registrado() {
    let contrato = RustaceoLibre::default();
    let caller = AccountId::from([0x4; 32]);
    let id_producto = 1;

    let resultado = contrato._ver_producto(caller, id_producto);

    assert_eq!(resultado, None);
}

}