use ink::{prelude::vec::Vec, prelude::string::String, primitives::AccountId};

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
    pub vendedor: AccountId,
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
    pub fn new(vendedor: AccountId, nombre: String, descripcion: String, categoria: CategoriaProducto, precio: u128, stock: u32) -> Self {
        Self {
            vendedor,
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorVerProductosVendedor {
    VendedorInexistente,
    NoEsVendedor,
    NoTieneVentas,
}

impl RustaceoLibre {

    //

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

        // instanciar el producto y obtener id de publicacion
        let id_publicacion = self.next_id_publicaciones();
        let publicacion = Producto::new(
            caller,
                        nombre,
                        descripcion,
                        categoria,
                        precio,
                        stock
        );

        // agregar al map principal
        self.publicaciones.insert(id_publicacion, publicacion);

        // agregar al vendedor. primero: obtener publicaciones
        let mut publicaciones = if let Some(publicaciones) = vendedor.publicaciones { publicaciones }
        else { Default::default() };

        publicaciones.push(id_publicacion);
        // reemplazar publicaciones en vendedor
        vendedor.publicaciones = Some(publicaciones);
        // reemplazar vendedor en usuarios
        self.usuarios.insert(vendedor.id, &vendedor);

        // fin
        Ok(id_publicacion)
    }

    //

    // amaría usar &'a Producto pero ink! no soporta argumentos genéricos en general
    /// Dada una ID, devuelve la publicación del producto si es posible
    pub(crate) fn _ver_producto(&self, id_publicacion: u128) -> Option<&Producto> {
        self.publicaciones.get(&id_publicacion)
    }

    //

    /// Devuelve todos los productos que correspondan al vendedor que ejecute esta función.
    /// 
    /// Dará error si el usuario no existe, no está registrado como vendedor o no tiene publicaciones.
    pub(crate) fn _ver_publicaciones_vendedor(&self, caller: AccountId) -> Result<Vec<Producto>, ErrorVerProductosVendedor> {
        let Some(caller) = self.usuarios.get(caller)
        else { return Err(ErrorVerProductosVendedor::VendedorInexistente); };

        if !caller.es_vendedor() {
            return Err(ErrorVerProductosVendedor::NoEsVendedor);
        }

        let Some(publicaciones) = caller.publicaciones
        else { return Err(ErrorVerProductosVendedor::NoTieneVentas); };

        if publicaciones.is_empty() {
            return Err(ErrorVerProductosVendedor::NoTieneVentas);
        }

        // todo bien
        // mapear las publicaciones: Vec<u128> -> Vec<Producto> y devolver

        let vec_publicaciones: Vec<Producto> = publicaciones.iter().filter_map(| p | {
            self.publicaciones.get(p)
        }).cloned().collect();

        // si no está vacío, devolver
        if vec_publicaciones.is_empty() {
            Err(ErrorVerProductosVendedor::NoTieneVentas)
        } else {
            Ok(vec_publicaciones)
        }
    }
}