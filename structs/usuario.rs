use ink::prelude::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum Rol {
    #[default]
    Comprador, Vendedor, Ambos
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub struct Usuario {
    rol: Rol,
    ventas: Option<Vec<u128>>,
    compras: Option<Vec<u128>>,
}

impl Usuario {
    pub fn es_comprador(&self) -> bool {
        self.rol == Rol::Comprador || self.rol == Rol::Ambos
    }

    pub fn es_vendedor(&self) -> bool {
        self.rol == Rol::Vendedor || self.rol == Rol::Ambos
    }
}