
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
    Recibido, // por el comprador; este campo sólo le corresponde a él.
    Cancelado,
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
    pub id: u128,
    pub publicacion: u128,
    pub estado: EstadoCompra,
    pub comprador: AccountId,
    pub vendedor: AccountId,
    primer_solicitud_cancelacion: Option<AccountId>, // Almacena la AccountId de quien solicitó la cancelación para verificar mutualidad.
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
    pub fn new(id: u128, publicacion: u128, comprador: AccountId, vendedor: AccountId) -> Self {
        Self {
            id,
            publicacion,
            estado: Default::default(),
            comprador,
            vendedor,
            primer_solicitud_cancelacion: None
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
    UsuarioNoEsComprador,
    PublicacionInexistente,
    VendedorInexistente,
    StockInsuficiente
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorCompraDespachada {
    UsuarioNoRegistrado,
    CompraInexistente,
    SoloVendedorPuede,
    CompraCancelada,
    EstadoNoPendiente,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorRecibirCompra {
    UsuarioNoRegistrado, // de la compra
    CompraInexistente,
    SoloCompradorPuede,
    CompraYaRecibida,
    CompraNoDespachada,
    CompraCancelada,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorCancelarCompra {
    UsuarioNoRegistrado,
    CompraInexistente,
    UsuarioNoParticipa, // de la compra
    CompraYaRecibida,
    CompraYaCancelada,
    EsperandoConfirmacionMutua, // sólo si quien ya solicitó la cancelación es quien hace el llamado a cancelar
}

impl RustaceoLibre {

    pub fn _comprar_producto(&mut self, comprador: AccountId, id_publicacion: u128, cantidad: u32) -> Result<u128, ErrorRegistrarCompra> {
        //validar usuario 
        let Some(comprador) = self.usuarios.get(comprador)
        else { return Err(ErrorRegistrarCompra::UsuarioInexistente); };
        
        //validar rol
        if !comprador.es_comprador() {
            return Err(ErrorRegistrarCompra::UsuarioNoEsComprador);
        }

        //validar publicacion
        let Some(publicacion) = self.publicaciones.get(&id_publicacion).cloned()
        else { return Err(ErrorRegistrarCompra::PublicacionInexistente); };

        // validar stock
        if publicacion.stock < cantidad {
            return Err(ErrorRegistrarCompra::StockInsuficiente);
        }

        // validar vendedor
        let vendedor_id = publicacion.vendedor;
        let Some(vendedor) = self.usuarios.get(vendedor_id)
        else{ return Err(ErrorRegistrarCompra::VendedorInexistente);};

        //
        // todo bien
        // quitar stock
        //

        // último check
        let Some(resultado) = publicacion.stock.checked_sub(cantidad)
        else {return Err(ErrorRegistrarCompra::StockInsuficiente);};

        // primer modificación
        let mut publicacion = publicacion;
        publicacion.stock = resultado;
        self.publicaciones.insert(id_publicacion,publicacion);

        //
        // crear compra
        //

        let id_compra = self.next_id_compras();
        let compra = Compra::new(id_compra, id_publicacion, comprador.id, vendedor_id);

        // añadir compra al mapping de compras
        self.compras.insert(id_compra, compra);

        //
        // actualizar compraventas al comprador
        //

        let mut comprador = comprador;
        comprador.compraventas.push(id_compra);

        self.usuarios.insert(comprador.id,&comprador);

        //
        // actualizar compraventas al vendedor
        //

        let mut vendedor = vendedor;
        vendedor.compraventas.push(id_compra);

        self.usuarios.insert(vendedor.id,&vendedor);

        // fin
        Ok(id_compra)
    }


    // marca como despachado el estado de la compra, solo si es vendedor y no esta en estado pendiente
    pub fn _compra_despachada(&mut self, caller: AccountId, compra_id: u128) -> Result<(), ErrorCompraDespachada> {
        // validar usuario
        let Some(usuario) = self.usuarios.get(caller)
        else { return Err(ErrorCompraDespachada::UsuarioNoRegistrado); };

        // validar compra #1
        let Some(compra) = usuario.compraventas.iter().find_map(|&id| if id == compra_id { Some(id) } else { None })
        else { return Err(ErrorCompraDespachada::CompraInexistente) };

        // validar compra #2
        let Some(compra) = self.compras.get(&compra)
        else { return Err(ErrorCompraDespachada::CompraInexistente) };

        // validar compra no cancelada
        if compra.estado == EstadoCompra::Cancelado {
            return Err(ErrorCompraDespachada::CompraCancelada);
        }
        
        // validar caller == vendedor
        if compra.vendedor != caller {
            return Err(ErrorCompraDespachada::SoloVendedorPuede);
        }

        // validar estado == pendiente
        if compra.estado != EstadoCompra::Pendiente {
            return Err(ErrorCompraDespachada::EstadoNoPendiente);
        }

        // hacer cambios y guardar
        let mut compra = compra.clone();
        compra.estado = EstadoCompra::Despachado;
        self.compras.insert(compra.id, compra);

        // fin
        Ok(())
    }

    // marca como entregado el estado de la compra, solo si es comprador y no esta en estado despachado
    pub fn _compra_recibida(&mut self, caller: AccountId, id_compra: u128) -> Result<(), ErrorRecibirCompra> {
        let Some(usuario) = self.usuarios.get(caller)
        else { return Err(ErrorRecibirCompra::CompraInexistente); };

        let Some(compra) = usuario.compraventas.iter().find_map(|&id| if id == id_compra { Some(id) } else { None })
        else { return Err(ErrorRecibirCompra::CompraInexistente); };

        let Some(compra) = self.compras.get(&compra)
        else { return Err(ErrorRecibirCompra::CompraInexistente); };

        match compra.estado {
            EstadoCompra::Pendiente => return Err(ErrorRecibirCompra::CompraNoDespachada),
            EstadoCompra::Despachado => (),
            EstadoCompra::Recibido => return Err(ErrorRecibirCompra::CompraYaRecibida),
            EstadoCompra::Cancelado => return Err(ErrorRecibirCompra::CompraCancelada),
        }

        if compra.comprador != caller {
            return Err(ErrorRecibirCompra::SoloCompradorPuede);
        }

        let mut compra = compra.clone();
        compra.estado = EstadoCompra::Recibido;
        self.compras.insert(compra.id, compra);
        Ok(())
    }

    /// Cancela la compra si ambos participantes de la misma ejecutan esta misma función
    /// y si ésta no fue recibida ni ya cancelada.
    /// 
    /// Devuelve error si el usuario o la compra no existen, si el usuario no participa en la compra,
    /// si la compra ya fue cancelada o recibida y si quien solicita la cancelación ya la solicitó antes.
    pub fn _cancelar_compra(&mut self, id_compra: u128, caller: AccountId) -> Result<bool, ErrorCancelarCompra> {
        // validar usuario
        let Some(usuario) = self.usuarios.get(caller)
        else { return Err(ErrorCancelarCompra::UsuarioNoRegistrado); };

        // validar compra #1
        let Some(compra) = usuario.compraventas.iter().find_map(|&id| if id == id_compra { Some(id) } else { None })
        else { return Err(ErrorCancelarCompra::CompraInexistente); };

        // validar compra #2
        let Some(compra) = self.compras.get(&compra)
        else { return Err(ErrorCancelarCompra::CompraInexistente); };

        // validar comprador
        if compra.comprador != caller && compra.vendedor != caller {
            return Err(ErrorCancelarCompra::UsuarioNoParticipa);
        }

        // validar estado
        match compra.estado {
            EstadoCompra::Pendiente | EstadoCompra::Despachado => (),
            EstadoCompra::Recibido => return Err(ErrorCancelarCompra::CompraYaRecibida),
            EstadoCompra::Cancelado => return Err(ErrorCancelarCompra::CompraYaCancelada),
        }
        
        //
        // validar si ya existe una solicitud de cancelación
        //

        let mut compra= compra.clone();
        let Some(primer_solicitud_cancelacion) = compra.primer_solicitud_cancelacion 
        else {
            compra.primer_solicitud_cancelacion = Some(caller);
            self.compras.insert(compra.id, compra);
            return Ok(false);
        };

        //
        // asegurar que exista mutualidad
        //

        if primer_solicitud_cancelacion == caller {
            return Err(ErrorCancelarCompra::EsperandoConfirmacionMutua);
        }

        // insertar
        compra.estado = EstadoCompra::Cancelado;
        self.compras.insert(compra.id, compra);

        // fin
        Ok(true)
    }
}