//
// Producto
//

use ink::metadata::TypeInfo;
use ink::primitives::{AccountId};
use ink::scale;
use ink::storage::traits::StorageLayout;

/// Estructura de un único producto
#[derive(scale::Decode, scale::Encode)] // es necesario importar todo esto para que funcione dentro de Mapping
#[cfg_attr(feature = "std", derive(TypeInfo, StorageLayout))]
pub struct Producto {
    id: u128,
    vendedor: AccountId,
    nombre: String,
    descripcion: String,
    price: u128, // según internet, (Balance instanceof u128) == true
    stock: u32,
    active: bool
}