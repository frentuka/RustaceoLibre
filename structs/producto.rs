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
    Otros, Cat1, Cat2
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