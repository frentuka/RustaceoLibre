use ink::{prelude::vec::Vec, primitives::AccountId};

//
// rol
//

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

//
// usuario
//

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub struct Usuario {
    pub id: AccountId,
    pub rol: Rol,
    pub compraventas: Option<Vec<u128>>, // Guarda compras y ventas. Si vendedor == id, Usuario es el vendedor. Caso contrario, es comprador.
    pub publicaciones: Option<Vec<u128>>,
}

//
// impl usuario
//

impl Usuario {
    pub fn new(id: AccountId, rol: Rol) -> Self {
        Self {
            id,
            rol,
            compraventas: Default::default(),
            publicaciones: Default::default(),
        }
    }

    pub fn es_comprador(&self) -> bool {
        self.rol == Rol::Comprador || self.rol == Rol::Ambos
    }

    pub fn es_vendedor(&self) -> bool {
        self.rol == Rol::Vendedor || self.rol == Rol::Ambos
    }
}