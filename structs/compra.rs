
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
    Cancelado,
    SolicitadaCancelacionPorComprador
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
/*
Crear orden de compra (solo Compradores).
Al comprar: se crea la orden y se descuenta stock.
Estados de orden: pendiente, enviado, recibido, cancelada.
Solo el Vendedor puede marcar como enviado.
Solo el Comprador puede marcar como recibido o cancelada si aún está pendiente.
Cancelación requiere consentimiento mutuo. */
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
    UsuarioInexistente,
    PublicacionInexistente,
    CompradorNoEsComprador,
    StockInsuficiente
}

impl RustaceoLibre {

    pub fn _registrar_compra(&mut self, publicacion_id: u128, comprador: AccountId) -> Result<u128, ErrorRegistrarCompra> {


        //validar usuario 
        let Some(comprador) = self.usuarios.get(comprador)
        else { return Err(ErrorRegistrarCompra::UsuarioInexistente); };
        
        //validar rol
        if !comprador.es_comprador() {
            return Err(ErrorRegistrarCompra::CompradorNoEsComprador);
        }

        //validar publicacion
        let Some(publicacion) = self.publicaciones.get(&publicacion_id).cloned()
        else { return Err(ErrorRegistrarCompra::PublicacionInexistente); };


        if publicacion.stock == 0 {
            return Err(ErrorRegistrarCompra::StockInsuficiente);
        }

        //validar vendedor
        let vendedor_id = publicacion.vendedor;
        let Some(vendedor) = self.usuarios.get(vendedor_id)
        else{ return Err(ErrorRegistrarCompra::UsuarioInexistente);};

        // crear compra
        let compra_id = self.next_id_compras();
        let compra = Compra::new(compra_id,comprador.id,publicacion.vendedor);

        // añadir compra al mapping de compras
        self.compras.insert(compra_id, compra);

        // actualizar compraventas comprador
        let mut comprador = comprador;
        comprador.compraventas.push(compra_id);

        self.usuarios.insert(comprador.id,&comprador);

        //actualizar compraventas vendedor
        let mut vendedor = vendedor;
        vendedor.compraventas.push(compra_id);

        self.usuarios.insert(vendedor.id,&vendedor);

        // quitar stock

        let mut publicacion_data = publicacion;

        let Some(resultado) = publicacion_data.stock.checked_sub(1)
        else {return Err(ErrorRegistrarCompra::StockInsuficiente);};

        publicacion_data.stock = resultado;
        
        self.publicaciones.insert(publicacion_id,publicacion_data);

        Ok(compra_id)
    }



    // marca como despachado el estado de la compra, solo si es vendedor y no esta en estado pendiente
    pub fn marcar_como_despachado(&mut self, compra_id: u128, caller: AccountId) -> Result<(), &'static str> {
    let compra = self.compras.get_mut(&compra_id).ok_or("Compra no encontrada")?;

    if compra.vendedor != caller {
        return Err("Solo el vendedor puede marcar como despachado");
    }

    if compra.estado != EstadoCompra::Pendiente {
        return Err("La compra no está en estado pendiente");
    }

    compra.estado = EstadoCompra::Despachado;
    Ok(())
    }

    // marca como entregado el estado de la compra, solo si es comprador y no esta en estado despachado
    pub fn marcar_como_entregado(&mut self, compra_id: u128, caller: AccountId) -> Result<(), &'static str> {
    let compra = self.compras.get_mut(&compra_id).ok_or("Compra no encontrada")?;

    if compra.comprador != caller {
        return Err("Solo el comprador puede marcar como entregado");
    }

    if compra.estado != EstadoCompra::Despachado {
        return Err("La compra no está en estado despachado");
    }

    compra.estado = EstadoCompra::Entregado;
    Ok(())
    }

    // marca como SolicitadaCancelacionPorComprador el estado de la compra, solo si es comprador
    // y esta en estado pendiente
    pub fn solicitar_cancelacion(&mut self, compra_id: u128, caller: AccountId) -> Result<(), &'static str> {
    let compra = self.compras.get_mut(&compra_id).ok_or("Compra no encontrada")?;

    if compra.comprador != caller {
        return Err("Solo el comprador puede solicitar la cancelación");
    }

    if compra.estado != EstadoCompra::Pendiente {
        return Err("Solo se puede cancelar una compra pendiente");
    }

    compra.estado = EstadoCompra::SolicitadaCancelacionPorComprador;
    Ok(())
    }

    //marca como cancelado el estado de la compra, solo si es vendedor y fue solicitada previamente 
    //por el comprador(verificando que no haya sido procesada)
    
    pub fn confirmar_cancelacion(&mut self, compra_id: u128, caller: AccountId) -> Result<(), &'static str> {
    let compra = self.compras.get_mut(&compra_id).ok_or("Compra no encontrada")?;

    if compra.vendedor != caller {
        return Err("Solo el vendedor puede confirmar la cancelación");
    }

    if compra.estado != EstadoCompra::SolicitadaCancelacionPorComprador {
        return Err("La cancelación no fue solicitada o ya fue procesada");
    }

    compra.estado = EstadoCompra::Cancelado;
    Ok(())
    }

}