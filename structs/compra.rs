
//
// estado compra
//

use ink::primitives::AccountId;

use crate::rustaceo_libre::RustaceoLibre;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum EstadoCompra {
    #[default]
    Pendiente,
    Despachado,
    Entregado,
    Cancelado
}

//
// compra
//

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub struct Compra {
    pub publicacion: u128,
    pub estado: EstadoCompra,
    pub comprador: AccountId,
    pub vendedor: AccountId,
}

//
// impl Compra
//

impl Compra {
    pub fn new(publicacion: u128, comprador: AccountId, vendedor: AccountId) -> Self {
        Self {
            publicacion,
            estado: Default::default(),
            comprador,
            vendedor
        }
    }
}

//
// impl Compra -> RustaceoLibre
//

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorRegistrarCompra {
    CompradorInexistente,
    PublicacionInexistente,
    CompradorNoEsComprador,
    StockInsuficiente
}

impl RustaceoLibre {

    pub fn _registrar_compra(&mut self, publicacion: u128, comprador: AccountId) -> Result<(), ErrorRegistrarCompra> {
        let Some(comprador) = self.usuarios.get(comprador)
        else { return Err(ErrorRegistrarCompra::CompradorInexistente); };

        if !comprador.es_comprador() {
            return Err(ErrorRegistrarCompra::CompradorNoEsComprador);
        }

        let Some(publicacion) = self.publicaciones.get(&publicacion)
        else { return Err(ErrorRegistrarCompra::PublicacionInexistente); };

        if publicacion.stock == 0 {
            return Err(ErrorRegistrarCompra::StockInsuficiente);
        }

        // crear compra
        // añadirla a self.compras
        // añadirla a vendedor y comprador
        // quitar stock

        todo!();
    }

}