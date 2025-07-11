
//
// estado compra
//

use ink::primitives::AccountId;
use ink::prelude::vec::Vec;

use crate::{rustaceo_libre::RustaceoLibre, structs::producto::CategoriaProducto};

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
pub enum ErrorComprarProducto {
    CantidadCero,
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

// compra recibida

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorCompraRecibida {
    UsuarioNoRegistrado, // de la compra
    CompraInexistente,
    SoloCompradorPuede,
    CompraYaRecibida,
    CompraNoDespachada,
    CompraCancelada,
}

// cancelar compra

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

// ver compras

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorVerCompras {
    UsuarioNoRegistrado,
    NoEsComprador,
    SinCompraVenta,
}

// ver ventas

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorVerVentas {
    UsuarioNoRegistrado,
    NoEsVendedor,
    SinCompraVenta,
}

impl RustaceoLibre {

    /// Compra una cantidad de un producto
    /// 
    /// Puede dar error si el usuario no existe, no es comprador, la publicación no existe,
    /// el stock es insuficiente o el vendedor de la misma no existe.
    pub fn _comprar_producto(&mut self, caller: AccountId, id_publicacion: u128, cantidad: u32) -> Result<u128, ErrorComprarProducto> {
        if cantidad == 0 {
            return Err(ErrorComprarProducto::CantidadCero);
        }
        
        // validar usuario
        let Some(comprador) = self.usuarios.get(caller)
        else { return Err(ErrorComprarProducto::UsuarioInexistente); };
        
        // validar rol
        if !comprador.es_comprador() {
            return Err(ErrorComprarProducto::UsuarioNoEsComprador);
        }

        // validar publicacion
        let Some(publicacion) = self.publicaciones.get(&id_publicacion).cloned()
        else { return Err(ErrorComprarProducto::PublicacionInexistente); };

        // validar vendedor
        let id_vendedor = publicacion.vendedor;
        let Some(vendedor) = self.usuarios.get(id_vendedor)
        else{ return Err(ErrorComprarProducto::VendedorInexistente);};

        // validar que la cantidad ofertada en la publicación sea <= a la cantidad comprada
        let Some(nuevo_stock_publicacion) = publicacion.cantidad_ofertada.checked_sub(cantidad)
        else { return Err(ErrorComprarProducto::StockInsuficiente); };

        //
        // todo bien
        //

        // primer modificación
        let mut publicacion = publicacion;
        publicacion.cantidad_ofertada = nuevo_stock_publicacion;
        self.publicaciones.insert(id_publicacion,publicacion);

        //
        // crear compra
        //

        let id_compra = self.next_id_compras();
        let compra = Compra::new(id_compra, id_publicacion, comprador.id, id_vendedor);

        // añadir compra al mapping de compras
        self.compras.insert(id_compra, compra);

        //
        // actualizar compras al comprador
        //

        let mut comprador = comprador;
        comprador.agregar_compra(id_compra);

        self.usuarios.insert(comprador.id,&comprador);

        //
        // actualizar ventas al vendedor
        //

        let mut vendedor = vendedor;
        vendedor.agregar_venta(id_compra);

        self.usuarios.insert(vendedor.id,&vendedor);

        // fin
        Ok(id_compra)
    }

    //

    /// Si la compra indicada está pendiente y el usuario es el vendedor, se establece como recibida.
    /// 
    /// Puede dar error si el usuario no está registrado, la compra no existe,
    /// la compra no está pendiente, ya fue recibida, es el cliente quien intenta despacharla
    /// o ya fue cancelada.
    pub fn _compra_despachada(&mut self, caller: AccountId, id_compra: u128) -> Result<(), ErrorCompraDespachada> {
        // validar usuario
        let Some(usuario) = self.usuarios.get(caller)
        else { return Err(ErrorCompraDespachada::UsuarioNoRegistrado); };

        // validar venta #0
        let Some(ventas) = usuario.obtener_compras()
        else { return Err(ErrorCompraDespachada::CompraInexistente); };

        // validar venta #1
        let Some(venta) = ventas.iter().find_map(|&id| if id == id_compra { Some(id) } else { None })
        else { return Err(ErrorCompraDespachada::CompraInexistente) };

        // validar venta #2
        let Some(venta) = self.compras.get(&venta)
        else { return Err(ErrorCompraDespachada::CompraInexistente) };

        // validar caller == vendedor
        if venta.vendedor != caller {
            return Err(ErrorCompraDespachada::SoloVendedorPuede);
        }

        // validar compra no cancelada
        if venta.estado == EstadoCompra::Cancelado {
            return Err(ErrorCompraDespachada::CompraCancelada);
        }

        // validar estado == pendiente
        if venta.estado != EstadoCompra::Pendiente {
            return Err(ErrorCompraDespachada::EstadoNoPendiente);
        }

        // hacer cambios y guardar
        let mut venta = venta.clone();
        venta.estado = EstadoCompra::Despachado;
        self.compras.insert(venta.id, venta);

        // fin
        Ok(())
    }

    //

    /// Si la compra indicada fue despachada y el usuario es el comprador, se establece como recibida.
    /// 
    /// Puede dar error si el usuario no está registrado, la compra no existe,
    /// la compra no fue despachada, ya fue recibida, es el vendedor quien intenta recibirla
    /// o ya fue cancelada.
    pub fn _compra_recibida(&mut self, caller: AccountId, id_compra: u128) -> Result<(), ErrorCompraRecibida> {
        let Some(usuario) = self.usuarios.get(caller)
        else { return Err(ErrorCompraRecibida::UsuarioNoRegistrado); };

        let Some(compras) = usuario.obtener_compras()
        else { return Err(ErrorCompraRecibida::CompraInexistente); };

        let Some(compra) = compras.iter().find_map(|&id| if id == id_compra { Some(id) } else { None })
        else { return Err(ErrorCompraRecibida::CompraInexistente); };

        let Some(compra) = self.compras.get(&compra)
        else { return Err(ErrorCompraRecibida::CompraInexistente); };

        if compra.comprador != caller {
            return Err(ErrorCompraRecibida::SoloCompradorPuede);
        }

        match compra.estado {
            EstadoCompra::Pendiente => return Err(ErrorCompraRecibida::CompraNoDespachada),
            EstadoCompra::Despachado => (),
            EstadoCompra::Recibido => return Err(ErrorCompraRecibida::CompraYaRecibida),
            EstadoCompra::Cancelado => return Err(ErrorCompraRecibida::CompraCancelada),
        }

        let mut compra = compra.clone();
        compra.estado = EstadoCompra::Recibido;
        self.compras.insert(compra.id, compra);
        Ok(())
    }

    //

    /// Cancela la compra si ambos participantes de la misma ejecutan esta misma función
    /// y si ésta no fue recibida ni ya cancelada.
    /// 
    /// Devuelve error si el usuario o la compra no existen, si el usuario no participa en la compra,
    /// si la compra ya fue cancelada o recibida y si quien solicita la cancelación ya la solicitó antes.
    pub fn _cancelar_compra(&mut self, caller: AccountId, id_compra: u128) -> Result<bool, ErrorCancelarCompra> {
        // validar usuario
        let Some(usuario) = self.usuarios.get(caller)
        else { return Err(ErrorCancelarCompra::UsuarioNoRegistrado); };

        // validar compra #0
        let Some(compras) = usuario.obtener_compras()
        else { return Err(ErrorCancelarCompra::CompraInexistente); };

        // validar compra #1
        let Some(compra) = compras.iter().find_map(|&id| if id == id_compra { Some(id) } else { None })
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

    //

    /// Devuelve las compras del usuario que lo ejecuta
    /// 
    /// Dará error si el usuario no está registrado como comprador o no tiene compras
    pub fn _ver_compras(&self, caller: AccountId) -> Result<Vec<Compra>, ErrorVerCompras> {
        let Some(usuario) = self.usuarios.get(caller)
        else { return Err(ErrorVerCompras::UsuarioNoRegistrado) };

        if !usuario.es_comprador() {
            return Err(ErrorVerCompras::NoEsComprador);
        }

        let Some(compras) = usuario.obtener_compras()
        else { return Err(ErrorVerCompras::SinCompraVenta); };

        let compras: Vec<Compra> = compras.iter().filter_map(|id_compraventa| {
            let Some(compra) = self.compras.get(&id_compraventa)
            else { return None };
            Some(compra)
        }).cloned().collect();

        if compras.is_empty() {
            return Err(ErrorVerCompras::SinCompraVenta);
        }

        Ok(compras)
    }

    //

    /// Devuelve las compras del usuario que lo ejecuta que estén en el estado especificado
    /// 
    /// Dará error si el usuario no está registrado como comprador o no tiene compras en ese estado
    pub fn _ver_compras_estado(&self, caller: AccountId, estado: EstadoCompra) -> Result<Vec<Compra>, ErrorVerCompras> {
        let compras = self._ver_compras(caller)?;
        let compras = compras.iter().filter(|compra| compra.estado == estado).cloned().collect();
        Ok(compras)
    }

    //

    /// Devuelve las compras del usuario que lo ejecuta que estén en el estado especificado
    /// 
    /// Dará error si el usuario no está registrado como comprador o no tiene compras en ese estado
    pub fn _ver_compras_categoria(&self, caller: AccountId, categoria: CategoriaProducto) -> Result<Vec<Compra>, ErrorVerCompras> {
        let compras = self._ver_compras(caller)?;
        let compras = compras.iter().filter(|compra| {
            // obtener publicación desde id
            let Some(publicacion) = self.publicaciones.get(&compra.publicacion)
            else { return false; };

            // obtener producto desde id
            let Some(producto) = self.productos.get(&publicacion.producto)
            else { return false; };

            // fin
            producto.categoria == categoria
        }).cloned().collect();
        Ok(compras)
    }

    //

    /// Devuelve las compras del usuario que lo ejecuta
    /// 
    /// Dará error si el usuario no está registrado como comprador o no tiene compras
    pub fn _ver_ventas(&self, caller: AccountId) -> Result<Vec<Compra>, ErrorVerVentas> {
        let Some(usuario) = self.usuarios.get(caller)
        else { return Err(ErrorVerVentas::UsuarioNoRegistrado) };

        if !usuario.es_vendedor() {
            return Err(ErrorVerVentas::NoEsVendedor);
        }

        let Some(ventas) = usuario.obtener_ventas()
        else { return Err(ErrorVerVentas::SinCompraVenta); };
        
        let ventas: Vec<Compra> = ventas.iter().filter_map(|id_compraventa| {
            let Some(venta) = self.compras.get(&id_compraventa)
            else { return None };
            Some(venta)
        }).cloned().collect();

        if ventas.is_empty() {
            return Err(ErrorVerVentas::SinCompraVenta);
        }

        Ok(ventas)
    }

    //

    /// Devuelve las compras del usuario que lo ejecuta que estén en el estado especificado
    /// 
    /// Dará error si el usuario no está registrado como comprador o no tiene compras en ese estado
    pub fn _ver_ventas_estado(&self, caller: AccountId, estado: EstadoCompra) -> Result<Vec<Compra>, ErrorVerVentas> {
        let ventas = self._ver_ventas(caller)?;
        let ventas = ventas.iter().filter(|ventas| ventas.estado == estado).cloned().collect();
        Ok(ventas)
    }

    //

    /// Devuelve las compras del usuario que lo ejecuta que estén en el estado especificado
    /// 
    /// Dará error si el usuario no está registrado como comprador o no tiene compras en ese estado
    pub fn _ver_ventas_categoria(&self, caller: AccountId, categoria: CategoriaProducto) -> Result<Vec<Compra>, ErrorVerVentas> {
        let ventas = self._ver_ventas(caller)?;
        let ventas = ventas.iter().filter(|ventas| {
            // obtener publicacion desde id
            let Some(publicacion) = self.publicaciones.get(&ventas.publicacion)
            else { return false };

            // obtener producto desde id
            let Some(producto) = self.productos.get(&publicacion.producto)
            else { return false; };

            producto.categoria == categoria
        }).cloned().collect();
        Ok(ventas)
    }

}