
//
// estado compra
//

use ink::primitives::AccountId;
use ink::prelude::vec::Vec;

use crate::{rustaceo_libre::RustaceoLibre, structs::producto::CategoriaProducto};

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum EstadoCompra {
    Pendiente(u64),
    Despachado(u64),
    Recibido(u64), // por el comprador; este campo sólo le corresponde a él.
    Cancelado(u64),
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
    pub timestamp: u64,
    pub publicacion: u128,
    pub valor: u128,
    pub fondos_transferidos: bool,
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
    pub fn new(id: u128, timestamp: u64, publicacion: u128, valor: u128, comprador: AccountId, vendedor: AccountId) -> Self {
        Self {
            id,
            timestamp,
            publicacion,
            valor,
            fondos_transferidos: false,
            estado: EstadoCompra::Pendiente(timestamp),
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
    StockInsuficiente,
    ValorTransferidoInsuficiente,
    Desconocido
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorReclamarFondos {
    UsuarioNoRegistrado,
    CompraNoExiste,
    NoEsElVendedor,
    NoConvalidaPoliticaDeReclamo,
    FondosYaTransferidos,
    EstadoNoEsDespachado,
}

impl RustaceoLibre {

    /// Compra una cantidad de un producto
    /// 
    /// Puede dar error si el usuario no existe, no es comprador, la publicación no existe,
    /// el stock es insuficiente o el vendedor de la misma no existe.
    pub fn _comprar_producto(&mut self, timestamp: u64, caller: AccountId, id_publicacion: u128, cantidad: u32, valor_transferido: u128) -> Result<u128, ErrorComprarProducto> {
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

        // validar que la cantidad de valor transferida sea suficiente para pagar
        let Some(valor_total_compra) = publicacion.precio_unitario.checked_mul(u128::from(cantidad)) // safe cast: u32 -> u128
        else { return Err(ErrorComprarProducto::Desconocido); }; 

        if valor_transferido < valor_total_compra {
            return Err(ErrorComprarProducto::ValorTransferidoInsuficiente);
        }

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
        let compra = Compra::new(id_compra, timestamp, id_publicacion, valor_total_compra, comprador.id, id_vendedor);

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

    /// Política de reclamo:
    /// 
    /// Si el vendedor despachó la compra y el comprador no la marcó como recibida después de 30 días,
    /// el comprador puede reclamar los fondos de la compra y la misma se marcará automáticamente como recibida.
    /// 
    /// Puede dar error si el usuario no está registrado, la compra no existe,
    /// el usuario no es el vendedor de la compra o su reclamo no condice con la política de reclamo
    pub fn _reclamar_fondos(&mut self, timestamp: u64, caller: AccountId, id_compra: u128) -> Result<u128, ErrorReclamarFondos> {
        // validar usuario
        if !self.usuarios.contains(caller) {
            return Err(ErrorReclamarFondos::UsuarioNoRegistrado);
        }

        // validar compra
        let Some(compra) = self.compras.get(&id_compra).cloned()
        else { return Err(ErrorReclamarFondos::CompraNoExiste); };

        // validar usuario es vendedor
        if caller != compra.vendedor {
            return Err(ErrorReclamarFondos::NoEsElVendedor);
        }

        // validar que los fondos no hayan sido ya transferidos
        if compra.fondos_transferidos {
            return Err(ErrorReclamarFondos::FondosYaTransferidos);
        }

        // validar politica de reclamo: estado de la compra
        let EstadoCompra::Despachado(timestamp_despacho) = compra.estado
        else { return Err(ErrorReclamarFondos::EstadoNoEsDespachado); };

        let Some(elapsed_time) = timestamp.checked_sub(timestamp_despacho)
        else { return Err(ErrorReclamarFondos::NoConvalidaPoliticaDeReclamo) };

        // 30 días: 1000ms*60s*60m*24h*30d
        let time_millis_30_days = 2_592_000_000u64;

        // validar politica de reclamo: 30 días desde despacho
        if elapsed_time < time_millis_30_days {
            return Err(ErrorReclamarFondos::NoConvalidaPoliticaDeReclamo);
        }

        let valor_compra = compra.valor;

        // guardar compra
        let mut compra = compra;
        compra.fondos_transferidos = true;
        compra.estado = EstadoCompra::Recibido(timestamp);
        self.compras.insert(id_compra, compra);

        // devolver Ok(valor) debería transferir los fondos de la compra en lib.rs
        Ok(valor_compra)
    }

    //

    /// Si la compra indicada está pendiente y el usuario es el vendedor, se establece como recibida.
    /// 
    /// Puede dar error si el usuario no está registrado, la compra no existe,
    /// la compra no está pendiente, ya fue recibida, es el cliente quien intenta despacharla
    /// o ya fue cancelada.
    pub fn _compra_despachada(&mut self, timestamp: u64, caller: AccountId, id_compra: u128) -> Result<(), ErrorCompraDespachada> {
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
        if matches!(venta.estado, EstadoCompra::Cancelado(_)) {
            return Err(ErrorCompraDespachada::CompraCancelada);
        }

        // validar estado == pendiente
        if !matches!(venta.estado, EstadoCompra::Pendiente(_)) {
            return Err(ErrorCompraDespachada::EstadoNoPendiente);
        }

        // hacer cambios y guardar
        let mut venta = venta.clone();
        venta.estado = EstadoCompra::Despachado(timestamp);
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
    pub fn _compra_recibida(&mut self, timestamp: u64, caller: AccountId, id_compra: u128) -> Result<(AccountId, u128), ErrorCompraRecibida> {
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
            EstadoCompra::Pendiente(_) => return Err(ErrorCompraRecibida::CompraNoDespachada),
            EstadoCompra::Despachado(_) => (),
            EstadoCompra::Recibido(_) => return Err(ErrorCompraRecibida::CompraYaRecibida),
            EstadoCompra::Cancelado(_) => return Err(ErrorCompraRecibida::CompraCancelada),
        }

        let vendedor = compra.vendedor;
        let valor_compra = compra.valor;

        let mut compra = compra.clone();
        compra.estado = EstadoCompra::Recibido(timestamp);
        compra.fondos_transferidos = true;
        self.compras.insert(compra.id, compra);


        Ok((vendedor, valor_compra))
    }

    //

    /// Cancela la compra si ambos participantes de la misma ejecutan esta misma función
    /// y si ésta no fue recibida ni ya cancelada.
    /// 
    /// Excepción de unilateralidad:
    ///  - El comprador puede cancelar la compra unilateralmente si pasaron 14 días sin ser despachada por el vendedor.
    /// 
    /// Fondos:
    ///  - Si el comprador cancela la compra unilateralmente por haber pasado 14 días sin ser despachada,
    ///     se le devolverán la totalidad de los fondos.
    ///  - Si el comprador es el primero en solicitar la cancelación bilateral de la compra luego de ser despachada,
    ///     en caso de que se cancele, éste recibirá el 90% del reembolso y el vendedor se quedará con el restante 10%.
    ///  - Si el vendedor es el primero en solicitar la cancelación bilateral, aún luego de ser despachada,
    ///     en caso de que se cancele, el comprador recibirá la totalidad de los fondos.
    /// 
    /// Devuelve error si el usuario o la compra no existen, si el usuario no participa en la compra,
    /// si la compra ya fue cancelada o recibida y si quien solicita la cancelación ya la solicitó antes.
    pub fn _cancelar_compra(&mut self, timestamp: u64, caller: AccountId, id_compra: u128) -> Result<bool, ErrorCancelarCompra> {
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
            EstadoCompra::Pendiente(_) | EstadoCompra::Despachado(_) => (),
            EstadoCompra::Recibido(_) => return Err(ErrorCancelarCompra::CompraYaRecibida),
            EstadoCompra::Cancelado(_) => return Err(ErrorCancelarCompra::CompraYaCancelada),
        }

        // Excepción de unilateralidad:
        //  - El comprador puede cancelar la compra unilateralmente si pasaron 14 días sin ser despachada por el vendedor.
        if compra.comprador == caller {
            if let EstadoCompra::Despachado(timestamp_despacho) = compra.estado {
                // 7 días: 1000ms*60s*60m*24h*14d
                let time_millis_7_days = 1_209_600_000u64;
                let tiempo_transcurrido = timestamp.checked_sub(timestamp_despacho);
                // usar Some(tiempo_transcurrido) me obliga a returnar desde acá
                if tiempo_transcurrido.is_some() {
                    let tiempo_transcurrido = tiempo_transcurrido.unwrap();

                    if tiempo_transcurrido > time_millis_7_days {
                        // devolverle 90% al comprador
                        // darle 10% al vendedor
                        // cancelar compra
                        // establecer fondos de la compra como transferidos

                        return Ok(true)
                    }
                }
            }
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
        compra.estado = EstadoCompra::Cancelado(timestamp);
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