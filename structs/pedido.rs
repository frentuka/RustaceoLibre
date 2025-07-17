use ink::{primitives::AccountId, prelude::vec::Vec};

use crate::{rustaceo_libre::RustaceoLibre, structs::producto::CategoriaProducto};

//
// estado pedido
//

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum EstadoPedido { // (u64 -> timestamp)
    Pendiente(u64),
    Despachado(u64), // por el vendedor
    Recibido(u64),   // por el comprador
    Cancelado(u64),
}

//
// pedido
//

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub struct Pedido {
    pub id: u128,
    pub timestamp: u64,
    pub publicacion: u128,
    pub cantidad_comprada: u32,
    pub valor_total: u128, // cantidad de criptomoneda que el comprador transferirá al vendedor por esta operación
    pub fondos_fueron_transferidos: bool, // si los fondos (valor_total) fueron tranferidos al vendedor. sólo sucede cuando se marca como recibido
    pub estado: EstadoPedido,
    pub comprador: AccountId,
    pub vendedor: AccountId,
    pub calificacion_comprador: Option<u8>, // la calificación que el comprador dió al vendedor
    pub calificacion_vendedor: Option<u8>,  // viceversa
    primer_solicitud_cancelacion: Option<AccountId>, // almacena la id de quien solicitó la cancelación para verificar mutualidad
}

//
// impl pedido
//

impl Pedido {
    pub fn new(id: u128, timestamp: u64, publicacion: u128, cantidad_comprada: u32, valor: u128, comprador: AccountId, vendedor: AccountId) -> Self {
        Self {
            id,
            timestamp,
            publicacion,
            cantidad_comprada,
            valor_total: valor,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Pendiente(timestamp),
            comprador,
            vendedor,
            calificacion_comprador: None, // la calificación que dió el comprador
            calificacion_vendedor: None,  // ídem pero vendedor
            primer_solicitud_cancelacion: None
        }
    }
}

//
// impl pedido -> RustaceoLibre
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
    VendedorAutocomprandose,
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
pub enum ErrorProductoDespachado {
    UsuarioNoRegistrado,
    TransaccionInexistente,
    SoloVendedorPuede,
    PedidoYaDespachado,
    PedidoCancelado,
    EstadoNoPendiente,
}

// producto recibido

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorProductoRecibido {
    UsuarioNoRegistrado,
    PedidoInexistente,
    SoloCompradorPuede,
    PedidoYaRecibido,
    PedidoNoDespachado,
    PedidoCancelado,
}

// cancelar pedido

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorCancelarPedido {
    UsuarioNoRegistrado,
    PedidoInexistente,
    UsuarioNoParticipa, // del pedido
    PedidoYaRecibido,
    PedidoYaCancelado,
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
    NoTieneCompras,
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
    NoTieneVentas,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorReclamarFondos {
    UsuarioNoRegistrado,
    PedidoInexistente,
    SoloVendedorPuede,
    NoConvalidaPoliticaDeReclamo,
    FondosYaTransferidos,
    EstadoNoEsDespachado,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorCalificarPedido {
    CalificacionInvalida,
    UsuarioNoRegistrado,
    PedidoInexistente,
    PedidoNoRecibido,
    UsuarioNoParticipa,
    UsuarioYaCalifico,
    VendedorInexistente,
    CompradorInexistente,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Hash)]
pub struct ResultadoComprarProducto {
    pub id_nueva_transaccion: u128,
    pub monto_transferido_sobrante: u128
}

impl RustaceoLibre {

    //

    /// Compra una cantidad de un producto
    /// 
    /// Puede dar error si el usuario no existe, no es comprador, la publicación no existe,
    /// el stock es insuficiente o el vendedor de la misma no existe.
    pub fn _comprar_producto(&mut self, timestamp: u64, caller: AccountId, id_publicacion: u128, cantidad: u32, valor_transferido: u128) -> Result<ResultadoComprarProducto, ErrorComprarProducto> {
        // validar cantidad
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

        // validar que el vendedor no sea el comprador
        if caller == publicacion.vendedor {
            return Err(ErrorComprarProducto::VendedorAutocomprandose);
        }

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

        // asegurar que el valor sea válido
        if valor_transferido < valor_total_compra {
            return Err(ErrorComprarProducto::ValorTransferidoInsuficiente);
        }

        // asegurar que el valor sea válido #2
        let Some(monto_transferido_sobrante) = valor_transferido.checked_sub(valor_total_compra)
        else { return Err(ErrorComprarProducto::ValorTransferidoInsuficiente); };

        //
        // todo bien
        //

        // primer modificación
        let mut publicacion = publicacion;
        publicacion.cantidad_ofertada = nuevo_stock_publicacion;
        self.publicaciones.insert(id_publicacion,publicacion);

        //
        // crear transacción
        //

        let id_transaccion = self.next_id_pedidos();
        let transaccion = Pedido::new(id_transaccion, timestamp, id_publicacion, cantidad, valor_total_compra, comprador.id, id_vendedor);

        // añadir compra al mapping de compras
        self.pedidos.insert(id_transaccion, transaccion);

        //
        // actualizar compras al comprador
        //

        let mut comprador = comprador;
        comprador.agregar_compra(id_transaccion);

        self.usuarios.insert(comprador.id,&comprador);

        //
        // actualizar ventas al vendedor
        //

        let mut vendedor = vendedor;
        vendedor.agregar_venta(id_transaccion);

        self.usuarios.insert(vendedor.id,&vendedor);

        // fin
        Ok( ResultadoComprarProducto {
            id_nueva_transaccion: id_transaccion,
            monto_transferido_sobrante
        })
    }

    //

    /// Política de reclamo:
    /// 
    /// Si el vendedor despachó el pedido y el comprador no lo marcó como recibido después de 60 días,
    /// el vendedor puede reclamar los fondos del pedido y el mismo se marcará automáticamente como recibida,
    /// sin necesidad de consentimiento ni voluntad del comprador.
    /// 
    /// Puede dar error si el usuario no está registrado, la transacción no existe,
    /// el usuario no es el vendedor de la publicación o el tiempo pasado no condice con la política de reclamo
    pub fn _reclamar_fondos(&mut self, timestamp: u64, caller: AccountId, id_compra: u128) -> Result<u128, ErrorReclamarFondos> {
        // validar usuario
        if !self.usuarios.contains(caller) {
            return Err(ErrorReclamarFondos::UsuarioNoRegistrado);
        }

        // validar compra
        let Some(compra) = self.pedidos.get(&id_compra).cloned()
        else { return Err(ErrorReclamarFondos::PedidoInexistente); };

        // validar usuario es vendedor
        if caller != compra.vendedor {
            return Err(ErrorReclamarFondos::SoloVendedorPuede);
        }

        // validar que los fondos no hayan sido ya transferidos
        if compra.fondos_fueron_transferidos {
            return Err(ErrorReclamarFondos::FondosYaTransferidos);
        }

        // validar politica de reclamo: estado del pedido
        let EstadoPedido::Despachado(timestamp_despacho) = compra.estado
        else { return Err(ErrorReclamarFondos::EstadoNoEsDespachado); };

        // validar política de reclamo: tiempo desde despachado
        let Some(elapsed_time) = timestamp.checked_sub(timestamp_despacho)
        else { return Err(ErrorReclamarFondos::NoConvalidaPoliticaDeReclamo) };

        // 30 días: 1000ms*60s*60m*24h*60d
        let time_millis_60_days = 5_184_000_000u64;

        // validar politica de reclamo: 60 días desde despacho
        if elapsed_time < time_millis_60_days {
            return Err(ErrorReclamarFondos::NoConvalidaPoliticaDeReclamo);
        }

        let valor_compra = compra.valor_total;

        // guardar compra
        let mut compra = compra;
        compra.fondos_fueron_transferidos = true;
        compra.estado = EstadoPedido::Recibido(timestamp);
        self.pedidos.insert(id_compra, compra);

        // devolver Ok(valor) debería transferir los fondos de la compra en lib.rs
        Ok(valor_compra)
    }

    //

    /// Si el pedido indicada está pendiente y el usuario es el vendedor, se establece como recibida.
    /// 
    /// Puede dar error si el usuario no está registrado, el pedido no existe,
    /// no está pendiente, ya fue recibido, no es el vendedor quien intenta despacharlo
    /// o ya fue cancelada.
    pub fn _pedido_despachado(&mut self, timestamp: u64, caller: AccountId, id_venta: u128) -> Result<(), ErrorProductoDespachado> {
        // validar usuario
        let Some(usuario) = self.usuarios.get(caller)
        else { return Err(ErrorProductoDespachado::UsuarioNoRegistrado); };

        // validar venta #0
        let Some(ventas) = usuario.obtener_ventas()
        else { return Err(ErrorProductoDespachado::TransaccionInexistente); };

        // validar venta #1
        let Some(venta) = ventas.iter().find_map(|&id| if id == id_venta { Some(id) } else { None })
        else { return Err(ErrorProductoDespachado::TransaccionInexistente) };

        // validar venta #2
        let Some(venta) = self.pedidos.get(&venta)
        else { return Err(ErrorProductoDespachado::TransaccionInexistente) };

        // validar caller == vendedor
        if venta.vendedor != caller {
            return Err(ErrorProductoDespachado::SoloVendedorPuede);
        }

        // validar producto no despachado
        if matches!(venta.estado, EstadoPedido::Despachado(_)) {
            return Err(ErrorProductoDespachado::PedidoYaDespachado);
        }

        // validar compra no cancelada
        if matches!(venta.estado, EstadoPedido::Cancelado(_)) {
            return Err(ErrorProductoDespachado::PedidoCancelado);
        }

        // validar estado == pendiente
        if !matches!(venta.estado, EstadoPedido::Pendiente(_)) {
            return Err(ErrorProductoDespachado::EstadoNoPendiente);
        }

        // hacer cambios y guardar
        let mut venta = venta.clone();
        venta.estado = EstadoPedido::Despachado(timestamp);
        self.pedidos.insert(venta.id, venta);

        // fin
        Ok(())
    }

    //

    /// Si el pedido indicado fue despachado y el usuario es el comprador, se establece como recibido.
    /// Devuelve la ID del vendedor y la cantidad de criptomoneda que se le debe transferir por el pedido.
    /// 
    /// Puede dar error si el usuario no está registrado, la compra no existe,
    /// la compra no fue despachada, ya fue recibida, no es el comprador quien intenta recibirlo
    /// o ya fue cancelado.
    pub fn _pedido_recibido(&mut self, timestamp: u64, caller: AccountId, id_compra: u128) -> Result<(AccountId, u128), ErrorProductoRecibido> {
        // verificar usuario
        let Some(usuario) = self.usuarios.get(caller)
        else { return Err(ErrorProductoRecibido::UsuarioNoRegistrado); };

        // verificar que el usuario tenga compras
        let Some(compras) = usuario.obtener_compras()
        else { return Err(ErrorProductoRecibido::PedidoInexistente); };

        // verificar que el pedido exista en sus compras
        let Some(pedido) = compras.iter().find_map(|&id| if id == id_compra { Some(id) } else { None })
        else { return Err(ErrorProductoRecibido::PedidoInexistente); };

        // verificar que exista información del pedido
        let Some(pedido) = self.pedidos.get(&pedido)
        else { return Err(ErrorProductoRecibido::PedidoInexistente); };

        // verificar que el usuario sea el comprador
        if pedido.comprador != caller {
            return Err(ErrorProductoRecibido::SoloCompradorPuede);
        }

        // último chequeo: verificar que el pedido haya sido despachado
        match pedido.estado {
            EstadoPedido::Pendiente(_) => return Err(ErrorProductoRecibido::PedidoNoDespachado),
            EstadoPedido::Despachado(_) => (),
            EstadoPedido::Recibido(_) => return Err(ErrorProductoRecibido::PedidoYaRecibido),
            EstadoPedido::Cancelado(_) => return Err(ErrorProductoRecibido::PedidoCancelado),
        }

        let vendedor = pedido.vendedor;
        let valor_compra = pedido.valor_total;

        let mut compra = pedido.clone();
        compra.estado = EstadoPedido::Recibido(timestamp);
        compra.fondos_fueron_transferidos = true;
        self.pedidos.insert(compra.id, compra);

        Ok((vendedor, valor_compra))
    }

    //

    /// Dada una ID de pedido y una calificación (1..=5), se califica el mismo.
    /// Sólo se puede calificar una vez y sólo pueden calificar el comprador y vendedor de un pedido.
    /// 
    /// Devolverá error si el usuario no está registrado, la calificación no es válida (1..=5),
    /// la transacción no existe, no fue recibida o el usuario ya calificó esta transacción,
    pub fn _calificar_pedido(&mut self, caller: AccountId, id_compra: u128, calificacion: u8) -> Result<(), ErrorCalificarPedido> {
        // verificar que la calificacion este en el rango permitido
        if !(1..=5).contains(&calificacion) {
            return Err(ErrorCalificarPedido::CalificacionInvalida);
        }

        // verificar usurio registrado
        if !self.usuarios.contains(caller) {
            return Err(ErrorCalificarPedido::UsuarioNoRegistrado);
        }

        // verificar compra existente
        let Some(compra) = self.pedidos.get(&id_compra)
        else { return Err(ErrorCalificarPedido::PedidoInexistente); };

        // verificar que haya sido recibida
        if !matches!(compra.estado, EstadoPedido::Recibido(_)) {
            return Err(ErrorCalificarPedido::PedidoNoRecibido);
        }

        // procesar calificación del comprador
        if compra.comprador == caller {
            // verificar que no haya calificacion
            if compra.calificacion_comprador.is_some() {
                return Err(ErrorCalificarPedido::UsuarioYaCalifico);
            }

            // verificar comprador
            let Some(mut vendedor) = self.usuarios.get(compra.vendedor)
            else { return Err(ErrorCalificarPedido::VendedorInexistente); };

            // realizar calificación y guardar
            vendedor.calificar_como_vendedor(calificacion);
            self.usuarios.insert(vendedor.id, &vendedor);

            // guardar calificación en transaccion
            let mut compra = compra.clone();
            compra.calificacion_comprador = Some(calificacion);
            self.pedidos.insert(compra.id, compra);
            return Ok(())
        }

        // procesar calificación del vendedor
        if compra.vendedor == caller {
            // verificar que no haya calificacion
            if compra.calificacion_vendedor.is_some() {
                return Err(ErrorCalificarPedido::UsuarioYaCalifico);
            }

            // verificar comprador
            let Some(mut comprador) = self.usuarios.get(compra.comprador)
            else { return Err(ErrorCalificarPedido::CompradorInexistente); };

            // realizar calificación y guardar
            comprador.calificar_como_comprador(calificacion);
            self.usuarios.insert(comprador.id, &comprador);

            // guardar calificación en transaccion
            let mut compra = compra.clone();
            compra.calificacion_comprador = Some(calificacion);
            self.pedidos.insert(compra.id, compra);
            return Ok(())
        }

        // caller != compra.comprador && caller != compra.vendedor
        Err(ErrorCalificarPedido::UsuarioNoParticipa)
    }

    //

    /// Cancela el pedido si ambos participantes del mismo ejecutan esta misma función
    /// y si éste no fue recibida ni ya cancelada.
    /// Entrega automáticamente los fondos de la compra al comprador y el stock al vendedor.
    /// 
    /// Devuelve error si el usuario o pedido no existen, si el usuario no participa en el pedido,
    /// si el pedido ya fue cancelado o recibido y si quien solicita la cancelación ya la solicitó antes.
    pub fn _cancelar_pedido(&mut self, timestamp: u64, caller: AccountId, id_pedido: u128) -> Result<Option<(AccountId, u128)>, ErrorCancelarPedido> {
        // validar usuario
        let Some(usuario) = self.usuarios.get(caller)
        else { return Err(ErrorCancelarPedido::UsuarioNoRegistrado); };

        // validar compra #2
        let Some(pedido) = self.pedidos.get(&id_pedido)
        else { return Err(ErrorCancelarPedido::PedidoInexistente); };

        // validar comprador
        if pedido.comprador != caller && pedido.vendedor != caller {
            return Err(ErrorCancelarPedido::UsuarioNoParticipa);
        }

        // validar estado
        match pedido.estado {
            EstadoPedido::Pendiente(_) | EstadoPedido::Despachado(_) => (),
            EstadoPedido::Recibido(_) => return Err(ErrorCancelarPedido::PedidoYaRecibido),
            EstadoPedido::Cancelado(_) => return Err(ErrorCancelarPedido::PedidoYaCancelado),
        }
    
        //
        // validar si ya existe una solicitud de cancelación
        //

        let mut pedido = pedido.clone();
        let Some(primer_solicitud_cancelacion) = pedido.primer_solicitud_cancelacion 
        else {
            pedido.primer_solicitud_cancelacion = Some(caller);
            self.pedidos.insert(pedido.id, pedido);
            return Ok(None);
        };

        //
        // asegurar que exista mutualidad
        //

        if primer_solicitud_cancelacion == caller {
            return Err(ErrorCancelarPedido::EsperandoConfirmacionMutua);
        }

        let id_comprador = pedido.comprador;
        let valor_pedido = pedido.valor_total;

        // modificar publicación: devolver stock
        if let Some(mut publicacion) = self.publicaciones.get(&pedido.publicacion).cloned() {
            if let Some(nueva_cantidad_ofertada) = publicacion.cantidad_ofertada.checked_add(pedido.cantidad_comprada) {
                // modificar e insertar publicación con nueva cantidad ofertada
                publicacion.cantidad_ofertada = nueva_cantidad_ofertada;
                self.publicaciones.insert(pedido.publicacion, publicacion);
            }
        } // si la publicacion no existe, el stock se pierde. para evitarlo debo agregar "id_producto" a compra

        // modificar compra
        pedido.estado = EstadoPedido::Cancelado(timestamp);
        pedido.fondos_fueron_transferidos = true;
        self.pedidos.insert(pedido.id, pedido);

        // fin. se devolverán fondos en lib.rs
        Ok(Some((id_comprador, valor_pedido)))
    }

    //

    /// Devuelve las compras del usuario que lo ejecuta
    /// 
    /// Dará error si el usuario no está registrado como comprador o no tiene compras
    pub fn _ver_compras(&self, caller: AccountId) -> Result<Vec<Pedido>, ErrorVerCompras> {
        let Some(usuario) = self.usuarios.get(caller)
        else { return Err(ErrorVerCompras::UsuarioNoRegistrado) };

        if !usuario.es_comprador() {
            return Err(ErrorVerCompras::NoEsComprador);
        }

        let Some(compras) = usuario.obtener_compras()
        else { return Err(ErrorVerCompras::NoTieneCompras); };

        let compras: Vec<Pedido> = compras.iter().filter_map(|id_compraventa| {
            let Some(compra) = self.pedidos.get(&id_compraventa)
            else { return None };
            Some(compra)
        }).cloned().collect();

        if compras.is_empty() {
            return Err(ErrorVerCompras::NoTieneCompras);
        }

        Ok(compras)
    }

    //

    /// Devuelve las compras del usuario que lo ejecuta que estén en el estado especificado
    /// 
    /// Dará error si el usuario no está registrado como comprador o no tiene compras en ese estado
    pub fn _ver_compras_estado(&self, caller: AccountId, estado: EstadoPedido) -> Result<Vec<Pedido>, ErrorVerCompras> {
        let compras = self._ver_compras(caller)?;
        let compras = compras.iter().filter(|compra| compra.estado == estado).cloned().collect();
        Ok(compras)
    }

    //

    /// Devuelve las compras del usuario que lo ejecuta que estén en el estado especificado
    /// 
    /// Dará error si el usuario no está registrado como comprador o no tiene compras en ese estado
    pub fn _ver_compras_categoria(&self, caller: AccountId, categoria: CategoriaProducto) -> Result<Vec<Pedido>, ErrorVerCompras> {
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
    pub fn _ver_ventas(&self, caller: AccountId) -> Result<Vec<Pedido>, ErrorVerVentas> {
        let Some(usuario) = self.usuarios.get(caller)
        else { return Err(ErrorVerVentas::UsuarioNoRegistrado) };

        if !usuario.es_vendedor() {
            return Err(ErrorVerVentas::NoEsVendedor);
        }

        let Some(ventas) = usuario.obtener_ventas()
        else { return Err(ErrorVerVentas::NoTieneVentas); };
        
        let ventas: Vec<Pedido> = ventas.iter().filter_map(|id_compraventa| {
            let Some(venta) = self.pedidos.get(&id_compraventa)
            else { return None };
            Some(venta)
        }).cloned().collect();

        if ventas.is_empty() {
            return Err(ErrorVerVentas::NoTieneVentas);
        }

        Ok(ventas)
    }

    //

    /// Devuelve las compras del usuario que lo ejecuta que estén en el estado especificado
    /// 
    /// Dará error si el usuario no está registrado como comprador o no tiene compras en ese estado
    pub fn _ver_ventas_estado(&self, caller: AccountId, estado: EstadoPedido) -> Result<Vec<Pedido>, ErrorVerVentas> {
        let ventas = self._ver_ventas(caller)?;
        let ventas = ventas.iter().filter(|ventas| ventas.estado == estado).cloned().collect();
        Ok(ventas)
    }

    //

    /// Devuelve las compras del usuario que lo ejecuta que estén en el estado especificado
    /// 
    /// Dará error si el usuario no está registrado como comprador o no tiene compras en ese estado
    pub fn _ver_ventas_categoria(&self, caller: AccountId, categoria: CategoriaProducto) -> Result<Vec<Pedido>, ErrorVerVentas> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::{
        producto::CategoriaProducto,
        usuario::{Rol, RolDeSeleccion},
    };
    use ink::primitives::AccountId;


    #[ink::test]
    fn comprar_producto_funciona_correctamente() {
        let mut contrato = RustaceoLibre::default();

        let comprador = AccountId::from([0x1; 32]);
        let vendedor = AccountId::from([0x2; 32]);

        // Registrar usuarios
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Registrar producto
        let nombre = "Termo".into();
        let descripcion = "Acero inoxidable".into();
        let categoria = CategoriaProducto::Tecnologia;
        let stock = 10;
        let id_producto = contrato._registrar_producto(vendedor, nombre, descripcion, categoria, stock).unwrap();

        // Realizar publicación
        let precio_unitario = 100;
        let cantidad_ofertada = 5;
        let id_publicacion = contrato._realizar_publicacion(vendedor, id_producto, cantidad_ofertada, precio_unitario).unwrap();

        // Comprar producto
        let timestamp = 12345;
        let cantidad = 2;
        let valor_transferido = 200; // 2 * 100
        let resultado = contrato._comprar_producto(timestamp, comprador, id_publicacion, cantidad, valor_transferido);

        assert!(resultado.is_ok());
        let resultado_comprar_producto = resultado.unwrap();
        let id_compra = resultado_comprar_producto.id_nueva_transaccion;

        // Validar compra
        let compra = contrato.pedidos.get(&id_compra).unwrap();
        assert_eq!(compra.comprador, comprador);
        assert_eq!(compra.vendedor, vendedor);
        assert_eq!(compra.cantidad_comprada, cantidad);
        assert_eq!(compra.valor_total, valor_transferido);
    }


    #[ink::test]
    fn comprar_producto_falla_cantidad_cero() {
        let mut contrato = RustaceoLibre::default();

        let comprador = AccountId::from([0x1; 32]);
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Simular compra con cantidad = 0
        let resultado = contrato._comprar_producto(0, comprador, 999, 0, 100);

        assert_eq!(resultado, Err(ErrorComprarProducto::CantidadCero));
    }


    #[ink::test]
    fn comprar_producto_falla_usuario_inexistente() {
    let mut contrato = RustaceoLibre::default();

    let comprador = AccountId::from([0x1; 32]); // No lo registramos

    // Intentar comprar sin estar registrado
    let resultado = contrato._comprar_producto(0, comprador, 999, 1, 100);

    assert_eq!(resultado, Err(ErrorComprarProducto::UsuarioInexistente));
    }


    #[ink::test]
    fn comprar_producto_falla_usuario_no_es_comprador() {
        let mut contrato = RustaceoLibre::default();

        let vendedor = AccountId::from([0x1; 32]);

        // Registrar al usuario como VENDEDOR, no como comprador
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // Crear producto y publicarlo
        let nombre = "Cuadro".into();
        let descripcion = "Pintura".into();
        let categoria = CategoriaProducto::Hogar;
        let stock = 10;

        let id_producto = contrato._registrar_producto(vendedor, nombre, descripcion, categoria, stock).unwrap();
        let precio_unitario = 100;
        let id_publicacion = contrato._realizar_publicacion(vendedor, id_producto, stock, precio_unitario).unwrap();

        // El vendedor (no comprador) intenta comprar
        let resultado = contrato._comprar_producto(0, vendedor, id_publicacion, 1, 100);

        assert_eq!(resultado, Err(ErrorComprarProducto::UsuarioNoEsComprador));
    }


    #[ink::test]
    fn comprar_producto_falla_publicacion_inexistente() {
        let mut contrato = RustaceoLibre::default();

        let comprador = AccountId::from([0x1; 32]);

        // Registrar al usuario como Comprador
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Intentar comprar con una publicación que no existe
        let id_publicacion_invalido = 999;
        let resultado = contrato._comprar_producto(0, comprador, id_publicacion_invalido, 1, 100);

        assert_eq!(resultado, Err(ErrorComprarProducto::PublicacionInexistente));
    }


    #[ink::test]
    fn comprar_producto_falla_vendedor_inexistente() {
        let mut contrato = RustaceoLibre::default();

        let vendedor = AccountId::from([0x1; 32]);
        let comprador = AccountId::from([0x2; 32]);

        // Registrar ambos usuarios
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Registrar producto y publicación
        let id_producto = contrato._registrar_producto(
            vendedor,
            "Mate".into(),
            "Calabaza".into(),
            CategoriaProducto::Hogar,
            10,
        ).unwrap();

        let id_publicacion = contrato._realizar_publicacion(vendedor, id_producto, 5, 100).unwrap();

        // Simular que el vendedor fue eliminado
        contrato.usuarios.remove(vendedor);

        // Comprar el producto
        let resultado = contrato._comprar_producto(0, comprador, id_publicacion, 2, 200);

        assert_eq!(resultado, Err(ErrorComprarProducto::VendedorInexistente));
    }


    #[ink::test]
    fn comprar_producto_falla_stock_insuficiente() {
        let mut contrato = RustaceoLibre::default();

        let vendedor = AccountId::from([0x1; 32]);
        let comprador = AccountId::from([0x2; 32]);

        // Registrar usuarios
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Registrar producto y publicación con 5 unidades
        let id_producto = contrato._registrar_producto(
            vendedor,
            "Lapicera".into(),
            "Tinta negra".into(),
            CategoriaProducto::Tecnologia,
            5,
        ).unwrap();

        let id_publicacion = contrato._realizar_publicacion(vendedor, id_producto, 5, 50).unwrap();

        // El comprador intenta comprar 10 unidades (más de las ofertadas)
        let resultado = contrato._comprar_producto(0, comprador, id_publicacion, 10, 500);

        assert_eq!(resultado, Err(ErrorComprarProducto::StockInsuficiente));
    }


    #[ink::test]
    fn comprar_producto_falla_valor_transferido_insuficiente() {
        let mut contrato = RustaceoLibre::default();

        let vendedor = AccountId::from([0x1; 32]);
        let comprador = AccountId::from([0x2; 32]);

        // Registrar usuarios
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Registrar producto y publicación
        let id_producto = contrato._registrar_producto(
            vendedor,
            "Cuaderno".into(),
            "Rayado".into(),
            CategoriaProducto::Hogar,
            5,
        ).unwrap();

        let id_publicacion = contrato._realizar_publicacion(vendedor, id_producto, 5, 100).unwrap();

        // Intentar comprar 2 unidades con solo 150 transferidos (se necesitan 200)
        let resultado = contrato._comprar_producto(0, comprador, id_publicacion, 2, 150);

        assert_eq!(resultado, Err(ErrorComprarProducto::ValorTransferidoInsuficiente));
    }


    #[ink::test]
    fn comprar_producto_falla_error_desconocido() {
        let mut contrato = RustaceoLibre::default();

        let vendedor = AccountId::from([0x1; 32]);
        let comprador = AccountId::from([0x2; 32]);

        // Registrar usuarios
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Registrar producto y publicación con precio máximo
        let id_producto = contrato._registrar_producto(
            vendedor,
            "NFT".into(),
            "Muy caro".into(),
            CategoriaProducto::Tecnologia,
            5,
        ).unwrap();

        let precio_unitario = u128::MAX;
        let id_publicacion = contrato._realizar_publicacion(vendedor, id_producto, 5, precio_unitario).unwrap();

        // Intentar comprar 2 (precio_unitario * 2) → overflow
        let resultado = contrato._comprar_producto(0, comprador, id_publicacion, 2, u128::MAX);

        assert_eq!(resultado, Err(ErrorComprarProducto::Desconocido));
    }


    #[ink::test]
    fn reclamar_fondos_exitoso() {
        // Arrange
        let mut contrato = RustaceoLibre::new();

        // Simulación de cuentas
        let vendedor = AccountId::from([0x01; 32]);
        let comprador = AccountId::from([0x02; 32]);
        let id_compra = 0;
        let valor_total = 1000;
        let timestamp_despacho = 1_000_000;
        let timestamp_actual = timestamp_despacho + 5_184_000_000; // 60 días

        // Registrar vendedor y comprador
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Crear compra
        let compra = Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Despachado(timestamp_despacho),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        };
        contrato.pedidos.insert(id_compra, compra);

        // Act
        let resultado = contrato._reclamar_fondos(timestamp_actual, vendedor, id_compra);

        // Assert
        assert_eq!(resultado, Ok(valor_total));

        // Confirmar que la compra fue marcada como recibida y fondos transferidos
        let actualizada = contrato.pedidos.get(&id_compra).unwrap();
        assert!(actualizada.fondos_fueron_transferidos);
        assert_eq!(actualizada.estado, EstadoPedido::Recibido(timestamp_actual));
    }


    #[ink::test]
    fn reclamar_fondos_usuario_no_registrado() {
        // Arrange
        let mut contrato = RustaceoLibre::new();

        // Crear ID de compra válido y simular una compra
        let comprador = AccountId::from([0x02; 32]);
        let vendedor = AccountId::from([0x01; 32]);
        let id_compra = 0;
        let valor_total = 1000;
        let timestamp_despacho = 1_000_000;
        let timestamp_actual = timestamp_despacho + 5_184_000_000;

        // Registrar solo al comprador y no al vendedor
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Insertar la compra directamente
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Despachado(timestamp_despacho),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // Act
        let resultado = contrato._reclamar_fondos(timestamp_actual, vendedor, id_compra);

        // Assert
        assert_eq!(resultado, Err(ErrorReclamarFondos::UsuarioNoRegistrado));
    }


    #[ink::test]
    fn reclamar_fondos_compra_no_existe() {
        // Arrange
        let mut contrato = RustaceoLibre::new();
        let vendedor = AccountId::from([0x01; 32]);
        let timestamp_actual = 6_184_000_000;
        let id_compra_inexistente = 999;

        // Registrar al vendedor
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // No se inserta ninguna compra

        // Act
        let resultado = contrato._reclamar_fondos(timestamp_actual, vendedor, id_compra_inexistente);

        // Assert
        assert_eq!(resultado, Err(ErrorReclamarFondos::PedidoInexistente));
    }


    #[ink::test]
    fn reclamar_fondos_no_es_el_vendedor() {
        // Arrange
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);
        let vendedor_real = AccountId::from([0x02; 32]);
        let caller_falso = comprador; // el que intenta reclamar sin ser el vendedor
        let id_compra = 0;
        let timestamp_despacho = 0;
        let timestamp_actual = 6_184_000_000; // más de 60 días

        // Registrar ambos usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor_real, RolDeSeleccion::Vendedor).unwrap();

        // Crear compra despachada, pero el caller no es el vendedor
        let compra = Pedido {
            id: id_compra,
            timestamp: timestamp_despacho,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Despachado(timestamp_despacho),
            comprador,
            vendedor: vendedor_real,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        };
        contrato.pedidos.insert(id_compra, compra);

        // Act
        let resultado = contrato._reclamar_fondos(timestamp_actual, caller_falso, id_compra);

        // Assert
        assert_eq!(resultado, Err(ErrorReclamarFondos::SoloVendedorPuede));
    }


    #[ink::test]
    fn reclamar_fondos_ya_transferidos() {
        // Arrange
        let mut contrato = RustaceoLibre::new();
        let vendedor = AccountId::from([0x02; 32]);
        let comprador = AccountId::from([0x01; 32]);
        let id_compra = 0;
        let timestamp_despacho = 0;
        let timestamp_actual = 6_184_000_000; // más de 60 días

        // Registrar usuarios
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Crear compra con fondos ya transferidos
        let compra = Pedido {
            id: id_compra,
            timestamp: timestamp_despacho,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: true, // importante
            estado: EstadoPedido::Despachado(timestamp_despacho),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        };
        contrato.pedidos.insert(id_compra, compra);

        // Act
        let resultado = contrato._reclamar_fondos(timestamp_actual, vendedor, id_compra);

        // Assert
        assert_eq!(resultado, Err(ErrorReclamarFondos::FondosYaTransferidos));
    }



    #[ink::test]
    fn reclamar_fondos_estado_incorrecto() {
        // Arrange
        let mut contrato = RustaceoLibre::new();
        let vendedor = AccountId::from([0x02; 32]);
        let comprador = AccountId::from([0x01; 32]);
        let id_compra = 0;
        let timestamp_despacho = 0;
        let timestamp_actual = 6_184_000_000; // > 60 días

        // Registrar usuarios
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Crear compra en estado incorrecto (Pendiente)
        let compra = Pedido {
            id: id_compra,
            timestamp: timestamp_despacho,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Pendiente(timestamp_despacho), // debería ser Despachado
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        };
        contrato.pedidos.insert(id_compra, compra);

        // Act
        let resultado = contrato._reclamar_fondos(timestamp_actual, vendedor, id_compra);

        // Assert
        assert_eq!(resultado, Err(ErrorReclamarFondos::EstadoNoEsDespachado));
    }


    #[ink::test]
    fn reclamar_fondos_antes_de_tiempo() {
        // Arrange
        let mut contrato = RustaceoLibre::new();
        let vendedor = AccountId::from([0x02; 32]);
        let comprador = AccountId::from([0x01; 32]);
        let id_compra = 0;
        let timestamp_despacho = 0;
        let timestamp_actual = 2_000_000_000; // menos de 60 días después (~23 días)

        // Registrar usuarios
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Crear compra despachada recientemente
        let compra = Pedido {
            id: id_compra,
            timestamp: timestamp_despacho,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Despachado(timestamp_despacho),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        };
        contrato.pedidos.insert(id_compra, compra);

        // Act
        let resultado = contrato._reclamar_fondos(timestamp_actual, vendedor, id_compra);

        // Assert
        assert_eq!(resultado, Err(ErrorReclamarFondos::NoConvalidaPoliticaDeReclamo));
    }


    #[ink::test]
    fn reclamar_fondos_no_es_vendedor() {
        // Arrange
        let mut contrato = RustaceoLibre::new();
        let vendedor = AccountId::from([0x02; 32]);
        let otro_usuario = AccountId::from([0x03; 32]); // no es vendedor
        let comprador = AccountId::from([0x01; 32]);
        let id_compra = 0;
        let timestamp_despacho = 0;
        let timestamp_actual = 6_000_000_000; // más de 60 días

        // Registrar usuarios
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(otro_usuario, RolDeSeleccion::Vendedor).unwrap();

        // Crear compra despachada hace más de 60 días
        let compra = Pedido {
            id: id_compra,
            timestamp: timestamp_despacho,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Despachado(timestamp_despacho),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        };
        contrato.pedidos.insert(id_compra, compra);

        // Act
        let resultado = contrato._reclamar_fondos(timestamp_actual, otro_usuario, id_compra);

        // Assert
        assert_eq!(resultado, Err(ErrorReclamarFondos::SoloVendedorPuede));
    }


    #[ink::test]
    fn reclamar_fondos_fondos_ya_transferidos() {
        // Arrange
        let mut contrato = RustaceoLibre::new();
        let vendedor = AccountId::from([0x02; 32]);
        let comprador = AccountId::from([0x01; 32]);
        let id_compra = 0;
        let timestamp_despacho = 0;
        let timestamp_actual = 6_000_000_000; // más de 60 días

        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Compra ya con fondos transferidos
        let compra = Pedido {
            id: id_compra,
            timestamp: timestamp_despacho,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: true,
            estado: EstadoPedido::Despachado(timestamp_despacho),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        };
        contrato.pedidos.insert(id_compra, compra);

        // Act
        let resultado = contrato._reclamar_fondos(timestamp_actual, vendedor, id_compra);

        // Assert
        assert_eq!(resultado, Err(ErrorReclamarFondos::FondosYaTransferidos));
    }


    #[ink::test]
    fn reclamar_fondos_no_convalida_politica() {
        let mut contrato = RustaceoLibre::new();
        let vendedor = AccountId::from([0x02; 32]);
        let comprador = AccountId::from([0x01; 32]);
        let id_compra = 0;
        let timestamp_despacho = 0;
        // timestamp_actual menor a 60 días (en nanos)
        let timestamp_actual = 2_000_000_000; // ~23 días, menos de 60 días

        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: timestamp_despacho,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Despachado(timestamp_despacho),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        let resultado = contrato._reclamar_fondos(timestamp_actual, vendedor, id_compra);
        assert_eq!(resultado, Err(ErrorReclamarFondos::NoConvalidaPoliticaDeReclamo));
    }


    #[ink::test]
    fn compra_despachada_exitoso() {
        // Arrange
        let mut contrato = RustaceoLibre::new();

        // Crear cuentas
        let vendedor = AccountId::from([0x01; 32]);
        let comprador = AccountId::from([0x02; 32]);

        // Registrar cuentas
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Crear compra
        let id_compra = 123;
        let timestamp_pendiente = 1_000_000;

        let compra = Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 500,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Pendiente(timestamp_pendiente),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        };

        // Insertar la compra al contrato
        contrato.pedidos.insert(id_compra, compra);

        // Agregar la compra a la lista de ventas del vendedor
        {
            let mut usuario_vendedor = contrato.usuarios.get(vendedor).unwrap();
            assert!(usuario_vendedor.agregar_venta(id_compra));
            contrato.usuarios.insert(vendedor, &usuario_vendedor);
        }

        // Agregar la compra a la lista de compras del comprador (opcional, por coherencia)
        {
            let mut usuario_comprador = contrato.usuarios.get(comprador).unwrap();
            assert!(usuario_comprador.agregar_compra(id_compra));
            contrato.usuarios.insert(comprador, &usuario_comprador);
        }

        // Nuevo timestamp para marcar como despachado
        let timestamp_despacho = timestamp_pendiente + 100;

        // Act
        let resultado = contrato._pedido_despachado(timestamp_despacho, vendedor, id_compra);

        // Assert
        assert_eq!(resultado, Ok(()));
        let actualizada = contrato.pedidos.get(&id_compra).unwrap();
        assert!(matches!(actualizada.estado, EstadoPedido::Despachado(ts) if ts == timestamp_despacho));

    }


    #[ink::test]
    fn compra_despachada_usuario_no_registrado() {
        let mut contrato = RustaceoLibre::new();
        let vendedor = AccountId::from([0x01; 32]);
        let comprador = AccountId::from([0x02; 32]);

        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        // vendedor NO registrado

        let id_compra = 0;
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Pendiente(0),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        let resultado = contrato._pedido_despachado(123456, vendedor, id_compra);
        assert_eq!(resultado, Err(ErrorProductoDespachado::UsuarioNoRegistrado));
    }


    #[ink::test]
    fn compra_despachada_compra_inexistente() {
        let mut contrato = RustaceoLibre::new();

        let vendedor = AccountId::from([0x01; 32]);
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        let id_compra_invalido = 999;
        let resultado = contrato._pedido_despachado(123456, vendedor, id_compra_invalido);
        assert_eq!(resultado, Err(ErrorProductoDespachado::TransaccionInexistente));
    }

    #[ink::test]
    fn compra_recibida_exitoso() {
        // Arrange
        let mut contrato = RustaceoLibre::new();

        // Crear cuentas
        let vendedor = AccountId::from([0x01; 32]);
        let comprador = AccountId::from([0x02; 32]);

        // Registrar cuentas
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Crear compra
        let id_compra = 123;
        let timestamp_pendiente = 1_000_000;

        let compra = Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 500,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Despachado(timestamp_pendiente),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        };

        // Insertar la compra al contrato
        contrato.pedidos.insert(id_compra, compra.clone());

        {
            let mut usuario_vendedor = contrato.usuarios.get(vendedor).unwrap();
            assert!(usuario_vendedor.agregar_venta(id_compra));
            contrato.usuarios.insert(vendedor, &usuario_vendedor);
        }

        {
            let mut usuario_comprador = contrato.usuarios.get(comprador).unwrap();
            assert!(usuario_comprador.agregar_compra(id_compra));
            contrato.usuarios.insert(comprador, &usuario_comprador);
        }

        // Nuevo timestamp para marcar como despachado
        let timestamp_recibido = timestamp_pendiente + 100;

        // Act
        let resultado = contrato._pedido_recibido(timestamp_recibido, comprador, id_compra);

        // Assert
        let Ok((id_vendedor, monto_compra)) = resultado
        else { panic!("Debería ser Ok"); };

        assert_eq!(id_vendedor, vendedor);
        assert_eq!(monto_compra, compra.valor_total);

        let actualizada = contrato.pedidos.get(&id_compra).unwrap();
        assert!(matches!(actualizada.estado, EstadoPedido::Recibido(_)));
    }

    #[ink::test]
    fn cancelar_compra_exitoso() {
        // Arrange
        let mut contrato = RustaceoLibre::new();

        // Crear cuentas
        let vendedor = AccountId::from([0x01; 32]);
        let comprador = AccountId::from([0x02; 32]);

        // Registrar cuentas
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Crear compra
        let id_compra = 123;
        let timestamp = 1_000_000;

        let compra = Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 500,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Despachado(timestamp),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        };

        // Insertar la compra al contrato
        contrato.pedidos.insert(id_compra, compra.clone());

        {
            let mut usuario_vendedor = contrato.usuarios.get(vendedor).unwrap();
            assert!(usuario_vendedor.agregar_venta(id_compra));
            contrato.usuarios.insert(vendedor, &usuario_vendedor);
        }

        {
            let mut usuario_comprador = contrato.usuarios.get(comprador).unwrap();
            assert!(usuario_comprador.agregar_compra(id_compra));
            contrato.usuarios.insert(comprador, &usuario_comprador);
        }

        // cancelar compra por parte del comprador
        let res = contrato._cancelar_pedido(timestamp, comprador, id_compra);
        let Ok(res) = res else { panic!("Debería ser Ok"); };
        assert_eq!(res, None);

        // cancelar compra por parte del comprador
        let res = contrato._cancelar_pedido(timestamp, vendedor, id_compra);
        let Ok(res) = res else { panic!("Debería ser Ok"); };
        assert_eq!(res, Some((comprador, compra.valor_total)));

        let Some(pedido) = contrato.pedidos.get(&id_compra)
        else { panic!("Debería ser Some") };

        assert!(matches!(pedido.estado, EstadoPedido::Cancelado(_)));
    }

    #[ink::test]
    fn reclamar_fondos_estado_recibido() {
        let mut contrato = RustaceoLibre::new();
        let vendedor = AccountId::from([0x02; 32]);
        let comprador = AccountId::from([0x01; 32]);
        let id_compra = 0;
        let timestamp_recibido = 1_000_000_000;
        let timestamp_actual = 2_000_000_000;

        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: true,
            estado: EstadoPedido::Recibido(timestamp_recibido),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        let resultado = contrato._reclamar_fondos(timestamp_actual, vendedor, id_compra);
        assert_eq!(resultado, Err(ErrorReclamarFondos::FondosYaTransferidos));
    }


    #[ink::test]
    fn reclamar_fondos_estado_cancelado() {
        let mut contrato = RustaceoLibre::new();
        let vendedor = AccountId::from([0x02; 32]);
        let comprador = AccountId::from([0x01; 32]);
        let id_compra = 0;
        let timestamp_cancelado = 1_000_000_000;
        let timestamp_actual = 2_000_000_000;

        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Cancelado(timestamp_cancelado),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        let resultado = contrato._reclamar_fondos(timestamp_actual, vendedor, id_compra);
        assert_eq!(resultado, Err(ErrorReclamarFondos::EstadoNoEsDespachado));
    }


    #[ink::test]
    fn reclamar_fondos_estado_ya_recibido() {
        let mut contrato = RustaceoLibre::new();
        let vendedor = AccountId::from([0x02; 32]);
        let comprador = AccountId::from([0x01; 32]);
        let id_compra = 0;
        let timestamp_recibido = 1_000_000_000;
        let timestamp_actual = 2_000_000_000;

        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: true,
            estado: EstadoPedido::Recibido(timestamp_recibido),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        let resultado = contrato._reclamar_fondos(timestamp_actual, vendedor, id_compra);
        assert_eq!(resultado, Err(ErrorReclamarFondos::FondosYaTransferidos));
    }


    #[ink::test]
    fn compra_recibida_usuario_no_registrado() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x02; 32]);
        let id_compra = 1;

        // No registra el usuario comprador

        let resultado = contrato._pedido_recibido(1234, comprador, id_compra);
        assert_eq!(resultado, Err(ErrorProductoRecibido::UsuarioNoRegistrado));
    }


    #[ink::test]
    fn compra_recibida_ya_recibida() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x02; 32]);
        let vendedor = AccountId::from([0x01; 32]);
        let id_compra = 1;
        let timestamp_recibido = 1000;

        // Registrar usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // Insertar compra ya recibida
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: true,
            estado: EstadoPedido::Recibido(timestamp_recibido),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // Agregar compra a la lista de compras del usuario
        {
            let mut usuario = contrato.usuarios.get(comprador).unwrap();
            usuario.agregar_compra(id_compra);
            contrato.usuarios.insert(comprador, &usuario);
        }

        let resultado = contrato._pedido_recibido(2000, comprador, id_compra);
        assert_eq!(resultado, Err(ErrorProductoRecibido::PedidoYaRecibido));
    }


    #[ink::test]
    fn compra_recibida_cancelada() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x02; 32]);
        let vendedor = AccountId::from([0x01; 32]);
        let id_compra = 42;
        let timestamp_cancelado = 1234;

        // Registrar usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // Insertar compra cancelada
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Cancelado(timestamp_cancelado),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // Agregar compra a la lista de compras del usuario
        {
            let mut usuario = contrato.usuarios.get(comprador).unwrap();
            usuario.agregar_compra(id_compra);
            contrato.usuarios.insert(comprador, &usuario);
        }

        let resultado = contrato._pedido_recibido(2000, comprador, id_compra);
        assert_eq!(resultado, Err(ErrorProductoRecibido::PedidoCancelado));
    }


    #[ink::test]
    fn compra_recibida_no_despachada() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x02; 32]);
        let vendedor = AccountId::from([0x01; 32]);
        let id_compra = 7;
        let timestamp_pendiente = 1111;

        // Registrar usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // Insertar compra pendiente
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Pendiente(timestamp_pendiente),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // Agregar compra a la lista de compras del usuario
        {
            let mut usuario = contrato.usuarios.get(comprador).unwrap();
            usuario.agregar_compra(id_compra);
            contrato.usuarios.insert(comprador, &usuario);
        }

        let resultado = contrato._pedido_recibido(2000, comprador, id_compra);
        assert_eq!(resultado, Err(ErrorProductoRecibido::PedidoNoDespachado));
    }


    #[ink::test]
    fn compra_recibida_solo_comprador_puede() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x02; 32]);
        let vendedor = AccountId::from([0x01; 32]);
        let id_compra = 10;
        let timestamp_despachado = 2222;

        // Registrar usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // Insertar compra despachada
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Despachado(timestamp_despachado),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // Agregar compra a la lista de compras del comprador (solo del comprador)
        {
            let mut usuario = contrato.usuarios.get(comprador).unwrap();
            usuario.agregar_compra(id_compra);
            contrato.usuarios.insert(comprador, &usuario);
        }

        // El vendedor intenta marcar como recibida (no debe poder)
        let resultado = contrato._pedido_recibido(3000, vendedor, id_compra);
        assert_eq!(resultado, Err(ErrorProductoRecibido::PedidoInexistente));
    }


    #[ink::test]
    fn cancelar_compra_usuario_no_registrado() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x02; 32]);
        let id_compra = 1;

        // No registrar usuario

        let resultado = contrato._cancelar_pedido(1234, comprador, id_compra);
        assert_eq!(resultado, Err(ErrorCancelarPedido::UsuarioNoRegistrado));
    }


    #[ink::test]
    fn cancelar_compra_inexistente() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x02; 32]);
        let id_compra = 999;

        // Registrar usuario
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // No agregar la compra a la lista de compras del usuario

        let resultado = contrato._cancelar_pedido(1234, comprador, id_compra);
        assert_eq!(resultado, Err(ErrorCancelarPedido::PedidoInexistente));
    }


    #[ink::test]
    fn cancelar_compra_usuario_no_participa() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x02; 32]);
        let vendedor = AccountId::from([0x01; 32]);
        let otro_usuario = AccountId::from([0x03; 32]);
        let id_compra = 123;

        // Registrar usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(otro_usuario, RolDeSeleccion::Comprador).unwrap();

        // Insertar compra pendiente
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Pendiente(0),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // Agregar compra a la lista de compras del comprador
        {
            let mut usuario = contrato.usuarios.get(comprador).unwrap();
            usuario.agregar_compra(id_compra);
            contrato.usuarios.insert(comprador, &usuario);
        }
        // Agregar compra a la lista de ventas del vendedor
        {
            let mut usuario = contrato.usuarios.get(vendedor).unwrap();
            usuario.agregar_venta(id_compra);
            contrato.usuarios.insert(vendedor, &usuario);
        }
        // Agregar compra a la lista de compras del otro_usuario
        {
            let mut usuario = contrato.usuarios.get(otro_usuario).unwrap();
            usuario.agregar_compra(id_compra);
            contrato.usuarios.insert(otro_usuario, &usuario);
        }

        // El usuario que no participa intenta cancelar
        let resultado = contrato._cancelar_pedido(1234, otro_usuario, id_compra);
        assert_eq!(resultado, Err(ErrorCancelarPedido::UsuarioNoParticipa));
    }


    #[ink::test]
    fn cancelar_compra_ya_recibida() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x02; 32]);
        let vendedor = AccountId::from([0x01; 32]);
        let id_compra = 555;
        let timestamp_recibido = 8888;

        // Registrar usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // Insertar compra ya recibida
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: true,
            estado: EstadoPedido::Recibido(timestamp_recibido),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // Agregar compra a la lista de compras del comprador
        {
            let mut usuario = contrato.usuarios.get(comprador).unwrap();
            usuario.agregar_compra(id_compra);
            contrato.usuarios.insert(comprador, &usuario);
        }
        // Agregar compra a la lista de ventas del vendedor
        {
            let mut usuario = contrato.usuarios.get(vendedor).unwrap();
            usuario.agregar_venta(id_compra);
            contrato.usuarios.insert(vendedor, &usuario);
        }

        // El comprador intenta cancelar una compra ya recibida
        let resultado = contrato._cancelar_pedido(9999, comprador, id_compra);
        assert_eq!(resultado, Err(ErrorCancelarPedido::PedidoYaRecibido));
    }


    #[ink::test]
    fn cancelar_compra_ya_cancelada() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x02; 32]);
        let vendedor = AccountId::from([0x01; 32]);
        let id_compra = 777;
        let timestamp_cancelado = 9999;

        // Registrar usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // Insertar compra ya cancelada
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: true,
            estado: EstadoPedido::Cancelado(timestamp_cancelado),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // Agregar compra a la lista de compras del comprador
        {
            let mut usuario = contrato.usuarios.get(comprador).unwrap();
            usuario.agregar_compra(id_compra);
            contrato.usuarios.insert(comprador, &usuario);
        }
        // Agregar compra a la lista de ventas del vendedor
        {
            let mut usuario = contrato.usuarios.get(vendedor).unwrap();
            usuario.agregar_venta(id_compra);
            contrato.usuarios.insert(vendedor, &usuario);
        }

        // El comprador intenta cancelar una compra ya cancelada
        let resultado = contrato._cancelar_pedido(12345, comprador, id_compra);
        assert_eq!(resultado, Err(ErrorCancelarPedido::PedidoYaCancelado));
    }


    #[ink::test]
    fn ver_compras_usuario_no_registrado() {
        let contrato = RustaceoLibre::new();
        let usuario = AccountId::from([0x01; 32]);

        let resultado = contrato._ver_compras(usuario);
        assert_eq!(resultado, Err(ErrorVerCompras::UsuarioNoRegistrado));
    }

    #[ink::test]
    fn ver_compras_no_es_comprador() {
        let mut contrato = RustaceoLibre::new();
        let usuario = AccountId::from([0x01; 32]);

        // Registrar usuario como vendedor (no comprador)
        contrato._registrar_usuario(usuario, RolDeSeleccion::Vendedor).unwrap();

        let resultado = contrato._ver_compras(usuario);
        assert_eq!(resultado, Err(ErrorVerCompras::NoEsComprador));
    }


    #[ink::test]
    fn ver_compras_sin_compras() {
        let mut contrato = RustaceoLibre::new();
        let usuario = AccountId::from([0x01; 32]);

        // Registrar usuario como comprador
        contrato._registrar_usuario(usuario, RolDeSeleccion::Comprador).unwrap();

        // No agregar compras

        let resultado = contrato._ver_compras(usuario);
        assert_eq!(resultado, Err(ErrorVerCompras::NoTieneCompras));
    }


    #[ink::test]
    fn ver_compras_exitoso() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);
        let vendedor = AccountId::from([0x02; 32]);
        let id_compra = 123;

        // Registrar usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // Insertar compra
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 2,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Pendiente(0),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // Agregar compra a la lista de compras del comprador
        {
            let mut usuario = contrato.usuarios.get(comprador).unwrap();
            usuario.agregar_compra(id_compra);
            contrato.usuarios.insert(comprador, &usuario);
        }

        let resultado = contrato._ver_compras(comprador);
        assert!(resultado.is_ok());
        let compras = resultado.unwrap();
        assert_eq!(compras.len(), 1);
        assert_eq!(compras[0].id, id_compra);
    }


    #[ink::test]
    fn ver_ventas_usuario_no_registrado() {
        let contrato = RustaceoLibre::new();
        let usuario = AccountId::from([0x01; 32]);

        let resultado = contrato._ver_ventas(usuario);
        assert_eq!(resultado, Err(ErrorVerVentas::UsuarioNoRegistrado));
    }


    #[ink::test]
    fn ver_ventas_no_es_vendedor() {
        let mut contrato = RustaceoLibre::new();
        let usuario = AccountId::from([0x01; 32]);

        // Registrar usuario como comprador (no vendedor)
        contrato._registrar_usuario(usuario, RolDeSeleccion::Comprador).unwrap();

        let resultado = contrato._ver_ventas(usuario);
        assert_eq!(resultado, Err(ErrorVerVentas::NoEsVendedor));
    }


    #[ink::test]
    fn ver_ventas_sin_ventas() {
        let mut contrato = RustaceoLibre::new();
        let usuario = AccountId::from([0x01; 32]);

        // Registrar usuario como vendedor
        contrato._registrar_usuario(usuario, RolDeSeleccion::Vendedor).unwrap();

        // No agregar ventas

        let resultado = contrato._ver_ventas(usuario);
        assert_eq!(resultado, Err(ErrorVerVentas::NoTieneVentas));
    }


    #[ink::test]
    fn ver_ventas_exitoso() {
        let mut contrato = RustaceoLibre::new();
        let vendedor = AccountId::from([0x01; 32]);
        let comprador = AccountId::from([0x02; 32]);
        let id_compra = 456;

        // Registrar usuarios
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Insertar compra
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 3,
            valor_total: 150,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Pendiente(0),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // Agregar compra a la lista de ventas del vendedor
        {
            let mut usuario = contrato.usuarios.get(vendedor).unwrap();
            usuario.agregar_venta(id_compra);
            contrato.usuarios.insert(vendedor, &usuario);
        }

        let resultado = contrato._ver_ventas(vendedor);
        assert!(resultado.is_ok());
        let ventas = resultado.unwrap();
        assert_eq!(ventas.len(), 1);
        assert_eq!(ventas[0].id, id_compra);
    }


    #[ink::test]
    fn ver_compras_estado_vacio() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);
        let vendedor = AccountId::from([0x02; 32]);
        let id_compra = 1;

        // Registrar usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // Insertar compra en estado Pendiente
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Pendiente(0),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // Agregar compra a la lista de compras del comprador
        {
            let mut usuario = contrato.usuarios.get(comprador).unwrap();
            usuario.agregar_compra(id_compra);
            contrato.usuarios.insert(comprador, &usuario);
        }

        // Buscar compras en estado Despachado (no hay ninguna)
        let resultado = contrato._ver_compras_estado(comprador, EstadoPedido::Despachado(0));
        assert!(resultado.is_ok());
        let compras = resultado.unwrap();
        assert_eq!(compras.len(), 0);
    }


    #[ink::test]
    fn ver_compras_estado_exitoso() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);
        let vendedor = AccountId::from([0x02; 32]);
        let id_compra = 1;

        // Registrar usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // Insertar compra en estado Pendiente
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Pendiente(0),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // Agregar compra a la lista de compras del comprador
        {
            let mut usuario = contrato.usuarios.get(comprador).unwrap();
            usuario.agregar_compra(id_compra);
            contrato.usuarios.insert(comprador, &usuario);
        }

        // Buscar compras en estado Pendiente (hay una)
        let resultado = contrato._ver_compras_estado(comprador, EstadoPedido::Pendiente(0));
        assert!(resultado.is_ok());
        let compras = resultado.unwrap();
        assert_eq!(compras.len(), 1);
        assert_eq!(compras[0].id, id_compra);
    }


    #[ink::test]
    fn ver_ventas_estado_exitoso() {
        let mut contrato = RustaceoLibre::new();
        let vendedor = AccountId::from([0x01; 32]);
        let comprador = AccountId::from([0x02; 32]);
        let id_compra = 1;

        // Registrar usuarios
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Insertar compra en estado Pendiente
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Pendiente(0),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // Agregar compra a la lista de ventas del vendedor
        {
            let mut usuario = contrato.usuarios.get(vendedor).unwrap();
            usuario.agregar_venta(id_compra);
            contrato.usuarios.insert(vendedor, &usuario);
        }

        // Buscar ventas en estado Pendiente (hay una)
        let resultado = contrato._ver_ventas_estado(vendedor, EstadoPedido::Pendiente(0));
        assert!(resultado.is_ok());
        let ventas = resultado.unwrap();
        assert_eq!(ventas.len(), 1);
        assert_eq!(ventas[0].id, id_compra);
    }


    #[ink::test]
    fn ver_compras_categoria_vacio() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);
        let vendedor = AccountId::from([0x02; 32]);
        let id_compra = 1;

        // Registrar usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // Registrar producto y publicación en categoría Hogar
        let id_producto = contrato._registrar_producto(
            vendedor,
            "Silla".into(),
            "Plástica".into(),
            CategoriaProducto::Hogar,
            10,
        ).unwrap();
        let id_publicacion = contrato._realizar_publicacion(vendedor, id_producto, 5, 100).unwrap();

        // Insertar compra en categoría Hogar
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: id_publicacion,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Pendiente(0),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // Agregar compra a la lista de compras del comprador
        {
            let mut usuario = contrato.usuarios.get(comprador).unwrap();
            usuario.agregar_compra(id_compra);
            contrato.usuarios.insert(comprador, &usuario);
        }

        // Buscar compras en categoría Tecnología (no hay ninguna)
        let resultado = contrato._ver_compras_categoria(comprador, CategoriaProducto::Tecnologia);
        assert!(resultado.is_ok());
        let compras = resultado.unwrap();
        assert_eq!(compras.len(), 0);
    }


    #[ink::test]
    fn ver_compras_categoria_exitoso() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);
        let vendedor = AccountId::from([0x02; 32]);
        let id_compra = 2;

        // Registrar usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // Registrar producto y publicación en categoría Tecnología
        let id_producto = contrato._registrar_producto(
            vendedor,
            "Celular".into(),
            "Android".into(),
            CategoriaProducto::Tecnologia,
            10,
        ).unwrap();
        let id_publicacion = contrato._realizar_publicacion(vendedor, id_producto, 5, 500).unwrap();

        // Insertar compra en categoría Tecnología
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: id_publicacion,
            cantidad_comprada: 1,
            valor_total: 500,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Pendiente(0),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // Agregar compra a la lista de compras del comprador
        {
            let mut usuario = contrato.usuarios.get(comprador).unwrap();
            usuario.agregar_compra(id_compra);
            contrato.usuarios.insert(comprador, &usuario);
        }

        // Buscar compras en categoría Tecnología (hay una)
        let resultado = contrato._ver_compras_categoria(comprador, CategoriaProducto::Tecnologia);
        assert!(resultado.is_ok());
        let compras = resultado.unwrap();
        assert_eq!(compras.len(), 1);
        assert_eq!(compras[0].id, id_compra);
    }


    #[ink::test]
    fn ver_ventas_categoria_exitoso() {
        let mut contrato = RustaceoLibre::new();
        let vendedor = AccountId::from([0x01; 32]);
        let comprador = AccountId::from([0x02; 32]);
        let id_compra = 3;

        // Registrar usuarios
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Registrar producto y publicación en categoría Hogar
        let id_producto = contrato._registrar_producto(
            vendedor,
            "Mesa".into(),
            "Madera".into(),
            CategoriaProducto::Hogar,
            10,
        ).unwrap();
        let id_publicacion = contrato._realizar_publicacion(vendedor, id_producto, 5, 700).unwrap();

        // Insertar compra en categoría Hogar
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: id_publicacion,
            cantidad_comprada: 1,
            valor_total: 700,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Pendiente(0),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // Agregar compra a la lista de ventas del vendedor
        {
            let mut usuario = contrato.usuarios.get(vendedor).unwrap();
            usuario.agregar_venta(id_compra);
            contrato.usuarios.insert(vendedor, &usuario);
        }

        // Buscar ventas en categoría Hogar (hay una)
        let resultado = contrato._ver_ventas_categoria(vendedor, CategoriaProducto::Hogar);
        assert!(resultado.is_ok());
        let ventas = resultado.unwrap();
        assert_eq!(ventas.len(), 1);
        assert_eq!(ventas[0].id, id_compra);
    }


    #[ink::test]
    fn ver_ventas_categoria_vacio() {
        let mut contrato = RustaceoLibre::new();
        let vendedor = AccountId::from([0x01; 32]);
        let comprador = AccountId::from([0x02; 32]);
        let id_compra = 4;

        // Registrar usuarios
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Registrar producto y publicación en categoría Hogar
        let id_producto = contrato._registrar_producto(
            vendedor,
            "Silla".into(),
            "Plástica".into(),
            CategoriaProducto::Hogar,
            10,
        ).unwrap();
        let id_publicacion = contrato._realizar_publicacion(vendedor, id_producto, 5, 100).unwrap();

        // Insertar compra en categoría Hogar
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: id_publicacion,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Pendiente(0),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // Agregar compra a la lista de ventas del vendedor
        {
            let mut usuario = contrato.usuarios.get(vendedor).unwrap();
            usuario.agregar_venta(id_compra);
            contrato.usuarios.insert(vendedor, &usuario);
        }

        // Buscar ventas en categoría Tecnología (no hay ninguna)
        let resultado = contrato._ver_ventas_categoria(vendedor, CategoriaProducto::Tecnologia);
        assert!(resultado.is_ok());
        let ventas = resultado.unwrap();
        assert_eq!(ventas.len(), 0);
    }
    

    #[ink::test]
    fn calificar_transaccion_calificacion_invalida() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);
        let id_compra = 1;

        // Registrar usuario
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Intentar calificar con valor fuera de rango
        let resultado = contrato._calificar_pedido(comprador, id_compra, 0); // fuera de rango
        assert_eq!(resultado, Err(ErrorCalificarPedido::CalificacionInvalida));

        let resultado = contrato._calificar_pedido(comprador, id_compra, 6); // fuera de rango
        assert_eq!(resultado, Err(ErrorCalificarPedido::CalificacionInvalida));
    }
    

    #[ink::test]
    fn calificar_transaccion_usuario_no_registrado() {
        let mut contrato = RustaceoLibre::new();
        let usuario = AccountId::from([0x01; 32]);
        let id_compra = 1;

        // No registrar usuario

        let resultado = contrato._calificar_pedido(usuario, id_compra, 5);
        assert_eq!(resultado, Err(ErrorCalificarPedido::UsuarioNoRegistrado));
    }


    #[ink::test]
    fn calificar_transaccion_compra_inexistente() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);
        let id_compra = 999;

        // Registrar usuario
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // No insertar compra

        let resultado = contrato._calificar_pedido(comprador, id_compra, 5);
        assert_eq!(resultado, Err(ErrorCalificarPedido::PedidoInexistente));
    }


    #[ink::test]
    fn calificar_transaccion_compra_no_recibida() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);
        let vendedor = AccountId::from([0x02; 32]);
        let id_compra = 10;

        // Registrar usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // Insertar compra en estado Pendiente
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Pendiente(0),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        let resultado = contrato._calificar_pedido(comprador, id_compra, 5);
        assert_eq!(resultado, Err(ErrorCalificarPedido::PedidoNoRecibido));
    }


    #[ink::test]
    fn calificar_transaccion_usuario_ya_califico() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);
        let vendedor = AccountId::from([0x02; 32]);
        let id_compra = 20;

        // Registrar usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // Insertar compra recibida y ya calificada por el comprador
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: true,
            estado: EstadoPedido::Recibido(1234),
            comprador,
            vendedor,
            calificacion_comprador: Some(5), // ya calificó
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        let resultado = contrato._calificar_pedido(comprador, id_compra, 4);
        assert_eq!(resultado, Err(ErrorCalificarPedido::UsuarioYaCalifico));
    }


    #[ink::test]
    fn calificar_transaccion_usuario_no_participa() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);
        let vendedor = AccountId::from([0x02; 32]);
        let otro_usuario = AccountId::from([0x03; 32]);
        let id_compra = 30;

        // Registrar usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(otro_usuario, RolDeSeleccion::Comprador).unwrap();

        // Insertar compra recibida
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: true,
            estado: EstadoPedido::Recibido(1234),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // El usuario que no participa intenta calificar
        let resultado = contrato._calificar_pedido(otro_usuario, id_compra, 5);
        assert_eq!(resultado, Err(ErrorCalificarPedido::UsuarioNoParticipa));
    }


    #[ink::test]
    fn calificar_transaccion_vendedor_inexistente() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);
        let vendedor = AccountId::from([0x02; 32]);
        let id_compra = 40;

        // Registrar solo al comprador
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        // NO registrar al vendedor

        // Insertar compra recibida
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: true,
            estado: EstadoPedido::Recibido(1234),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // El comprador intenta calificar
        let resultado = contrato._calificar_pedido(comprador, id_compra, 5);
        assert_eq!(resultado, Err(ErrorCalificarPedido::VendedorInexistente));
    }


    #[ink::test]
    fn calificar_transaccion_comprador_inexistente() {
        let mut contrato = RustaceoLibre::new();
        let vendedor = AccountId::from([0x01; 32]);
        let comprador = AccountId::from([0x02; 32]);
        let id_compra = 41;

        // Registrar solo al vendedor
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        // NO registrar al comprador

        // Insertar compra recibida
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: true,
            estado: EstadoPedido::Recibido(1234),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // El vendedor intenta calificar
        let resultado = contrato._calificar_pedido(vendedor, id_compra, 5);
        assert_eq!(resultado, Err(ErrorCalificarPedido::CompradorInexistente));
    }


    #[ink::test]
    fn calificar_transaccion_exitoso() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);
        let vendedor = AccountId::from([0x02; 32]);
        let id_compra = 50;

        // Registrar usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // Insertar compra recibida
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: true,
            estado: EstadoPedido::Recibido(1234),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // El comprador califica la compra
        let resultado = contrato._calificar_pedido(comprador, id_compra, 5);
        assert_eq!(resultado, Ok(()));
        // Verificar que la calificación se guardó
        let compra_actualizada = contrato.pedidos.get(&id_compra).unwrap();
        assert_eq!(compra_actualizada.calificacion_comprador, Some(5));
    }


    #[ink::test]
    fn calificar_transaccion_vendedor_exitoso() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);
        let vendedor = AccountId::from([0x02; 32]);
        let id_compra = 51;

        // Registrar usuarios
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // Insertar compra recibida
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: true,
            estado: EstadoPedido::Recibido(1234),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        // El vendedor califica la compra
        let resultado = contrato._calificar_pedido(vendedor, id_compra, 4);
        assert_eq!(resultado, Ok(()));
        // Verificar que la calificación se guardó
        let compra_actualizada = contrato.pedidos.get(&id_compra).unwrap();
        assert_eq!(compra_actualizada.calificacion_comprador, Some(4));
    }


    #[ink::test]
    fn calificar_transaccion_compra_cancelada() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);
        let vendedor = AccountId::from([0x02; 32]);
        let id_compra = 60;

        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Cancelado(9999),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        let resultado = contrato._calificar_pedido(comprador, id_compra, 5);
        assert_eq!(resultado, Err(ErrorCalificarPedido::PedidoNoRecibido));
    }


    #[ink::test]
    fn reclamar_fondos_timestamp_invalido() {
        let mut contrato = RustaceoLibre::new();
        let vendedor = AccountId::from([0x01; 32]);
        let comprador = AccountId::from([0x02; 32]);
        let id_compra = 999;

        contrato._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        let timestamp_despacho = 1000;
        let timestamp_llamado = 500; // menor que el de despacho → causará None en `checked_sub`

        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 1,
            cantidad_comprada: 1,
            valor_total: 1000,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Despachado(timestamp_despacho),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        let resultado = contrato._reclamar_fondos(timestamp_llamado, vendedor, id_compra);
        assert_eq!(resultado, Err(ErrorReclamarFondos::NoConvalidaPoliticaDeReclamo));
    }


    #[ink::test]
    fn compra_recibida_sin_compras_registradas() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);

        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // No agregar compras

        let res = contrato._pedido_recibido(1000, comprador, 1234);
        assert_eq!(res, Err(ErrorProductoRecibido::PedidoInexistente));
    }


    #[ink::test]
    fn compra_recibida_id_no_en_lista_usuario() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);
        let id_compra = 1;

        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        // Usuario con otra compra
        {
            let mut usuario = contrato.usuarios.get(comprador).unwrap();
            usuario.agregar_compra(9999); // No es 1
            contrato.usuarios.insert(comprador, &usuario);
        }

        // Compra con id 1 sí existe, pero no asociada al usuario
        contrato.pedidos.insert(id_compra, Pedido {
            id: id_compra,
            timestamp: 0,
            publicacion: 1,
            cantidad_comprada: 1,
            valor_total: 500,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Despachado(0),
            comprador,
            vendedor: AccountId::from([0x02; 32]),
            calificacion_comprador: None,
            calificacion_vendedor: None,
            primer_solicitud_cancelacion: None,
        });

        let res = contrato._pedido_recibido(2000, comprador, id_compra);
        assert_eq!(res, Err(ErrorProductoRecibido::PedidoInexistente));
    }


    #[ink::test]
    fn compra_recibida_compra_inexistente_en_storage() {
        let mut contrato = RustaceoLibre::new();
        let comprador = AccountId::from([0x01; 32]);
        let id_compra = 1;

        contrato._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        {
            let mut usuario = contrato.usuarios.get(comprador).unwrap();
            usuario.agregar_compra(id_compra); // Está asociada
            contrato.usuarios.insert(comprador, &usuario);
        }

        // Pero NO insertamos la compra

        let res = contrato._pedido_recibido(2000, comprador, id_compra);
        assert_eq!(res, Err(ErrorProductoRecibido::PedidoInexistente));
    }


}

