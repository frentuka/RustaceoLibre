use crate::{rustaceo_libre::RustaceoLibre, structs::producto::Producto};
use ink::{prelude::vec::Vec, primitives::AccountId};

//
// publicacion
//

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub struct Publicacion {
    pub vendedor: AccountId,
    pub productos: Vec<Producto>,
    pub stock: u32,
    pub precio: u128,
}

//
// errores
//

pub enum ErrorNuevaPublicacion {
    StockCero,
    PrecioCero,
    VectorVacio,
}

//
// impl publicacion
//

impl Publicacion{
    fn new(vendedor: AccountId, productos: Vec<Producto>, stock:u32, precio:u128)-> Result<Publicacion, ErrorNuevaPublicacion>{
        // errores
        if productos.is_empty() {
            return Err(ErrorNuevaPublicacion::VectorVacio);
        }
        if stock == 0 {
            return Err(ErrorNuevaPublicacion::StockCero);
        }
        if precio == 0{
            return Err(ErrorNuevaPublicacion::PrecioCero);
        }

        return Ok(
            Publicacion{
                vendedor,
                productos,
                stock,
                precio
            }
        );
    }
}

//
// impl Publicacion -> RustaceoLibre
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
}

impl RustaceoLibre {
    /// Realiza una publicación con producto, precio y cantidad.
    /// Devuelve Error si el precio o la cantidad son 0, o si `caller` no existe o no es vendedor.
    pub(crate) fn _realizar_publicacion(&mut self, caller: AccountId, productos: Vec<Producto>, precio: u128) -> Result<(), ErrorRealizarPublicacion> {
        let Some(usuario) = self.usuarios.get(caller) else {
            return Err(ErrorRealizarPublicacion::UsuarioInexistente);
        };

        if !usuario.es_vendedor() {
            return Err(ErrorRealizarPublicacion::NoEsVendedor);
        }

        todo!();
    }
}