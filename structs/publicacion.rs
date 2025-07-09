use crate::structs::producto::{self, Producto};
use ink::{prelude::vec::Vec, xcm::v2::NetworkId::Polkadot}; // esto me permite usar el vector de Ink.


#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum Categoria {
    #[default]
    Otros, Cat1, Cat2
}


#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub struct Publicacion {
    pub producto: Vec<Producto>,
    pub categoria: Categoria,
    pub stock: u32,
    pub precio: u128,
}

pub enum ErrorCrearNuevaPublicacion{
    StockCero,
    PrecioCero,
    VectorVacio,
}

impl Publicacion{

    fn new (producto: Vec<Producto>, categoria: Categoria, stock:u32, precio:u128)-> Result<Publicacion, ErrorCrearNuevaPublicacion>{
        
        //manejo de errores
        if producto.is_empty() {
            return Err(ErrorCrearNuevaPublicacion::VectorVacio);
        }
        if stock == 0 {
            return Err(ErrorCrearNuevaPublicacion::StockCero);
        }
        if precio == 0{
            return Err(ErrorCrearNuevaPublicacion::PrecioCero);
        }
        //manejo de errores

        return Ok(
            Publicacion{
                producto,
                categoria,
                stock,
                precio
            }
        );
    }

}
