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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub struct Producto {
    pub nombre: String,
    pub descripcion: String,
    pub categoria: CategoriaProducto,
    pub precio: u128,
    pub stock: u32,
}

//
// impl Producto
//

impl Producto {
    pub fn new(nombre: String, descripcion: String, categoria: CategoriaProducto, precio: u128, stock: u32) -> Self {
        Self {
            nombre,
            descripcion,
            categoria,
            precio,
            stock
        }
    }
}

//
// impl Producto -> RustaceoLibre
//

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorRealizarPublicacion {
    UsuarioInexistente,
    NoEsVendedor,
    StockCero,
    PrecioCero,
}

impl RustaceoLibre {
    /// Realiza una publicación con producto, precio y cantidad.
    /// Devuelve Error si el precio o la cantidad son 0, o si `caller` no existe o no es vendedor.
    pub(crate) fn _realizar_publicacion(&mut self, caller: AccountId, nombre: String, descripcion: String, categoria: CategoriaProducto, precio: u128, stock: u32) -> Result<u128, ErrorRealizarPublicacion> {
        if precio == 0 {
            return Err(ErrorRealizarPublicacion::PrecioCero);
        }

        if stock == 0 {
            return Err(ErrorRealizarPublicacion::StockCero);
        }
        
        let Some(mut vendedor) = self.usuarios.get(caller) else {
            return Err(ErrorRealizarPublicacion::UsuarioInexistente);
        };

        if !vendedor.es_vendedor() {
            return Err(ErrorRealizarPublicacion::NoEsVendedor);
        }

        // proceder con la publicación
        let id_publicacion = self.next_id_publicaciones();

        let publicacion = Producto::new(
            nombre, descripcion, categoria, precio, stock
        );

        // agregar al map principal
        self.publicaciones.insert(id_publicacion, publicacion);

        // agregar al vendedor. primero: obtener publicaciones
        let mut publicaciones = if let Some(publicaciones) = vendedor.publicaciones { publicaciones }
        else { Default::default() };

        publicaciones.push(id_publicacion);
        // reemplazar en el vendedor
        vendedor.publicaciones = Some(publicaciones);
        // reemplazar vendedor en usuarios
        self.usuarios.insert(vendedor.id, &vendedor);

        Ok(id_publicacion)
    }
}