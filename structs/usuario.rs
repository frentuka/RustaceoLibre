//
// Usuario
//

use ink::metadata::TypeInfo;
use ink::primitives::AccountId;
use ink::scale;
use ink::storage::traits::StorageLayout;

/// Estructura de un único usuario
#[derive(scale::Decode, scale::Encode)] // es necesario importar todo esto para que funcione dentro de Mapping
#[cfg_attr(feature = "std", derive(TypeInfo, StorageLayout))]
pub struct Usuario {
    id: AccountId,
    comprador: bool,
    vendedor: bool,
    /// ID de las compras realizadas, estén en proceso o no.
    compras: Option<Vec<u128>>,
    /// ID de las ventas realizadas, estén en proceso o no.
    ventas: Option<Vec<u128>>,
    /// ID de las publicaciones existentes, estén activas o no.
    publicaciones: Option<Vec<u128>>
}