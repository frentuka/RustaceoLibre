use std::mem::discriminant;

use ink::primitives::AccountId;

use crate::rustaceo_libre::RustaceoLibre;

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum DisputaEnCurso {
    PendienteContraargumentacion,
    PendienteDefinicion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum DisputaResuelta {
    FavorComprador{ argumento_interventor: String },
    FavorVendedor{ argumento_interventor: String }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum EstadoDisputa {
    EnCurso(DisputaEnCurso),
    Resuelta(DisputaResuelta)
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub struct Disputa {
    pub id: u128,
    pub timestamp: u64,
    pub pedido: u128,
    pub estado: EstadoDisputa,
    pub argumento_comprador: String,
    pub argumento_vendedor: Option<String>,
    pub interventor: Option<AccountId>
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorDisputarPedido {
    UsuarioNoRegistrado,
    PedidoInexistente,
    UsuarioNoParticipa,
    DisputaEnCurso,
    DisputaFinalizada,
    PlazoDeDisputaExpirado,
    DisputaPendienteResolucion,
    SoloCompradorPuedeDisputar,
    SoloVendedorPuedeContraargumentar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorResolverDisputa {
    UsuarioNoStaff,
    PedidoInexistente,
    DisputaNoEnCurso,
    DisputaFinalizada
}

impl Disputa {
    pub fn en_curso(&self) -> bool {
        let EstadoDisputa::EnCurso(_) = &self.estado
        else { return false; };

        true
    }

    pub fn en_curso_pendiente_contraargumentacion(&self) -> bool {
        let EstadoDisputa::EnCurso(curso) = &self.estado
        else { return false; };

        matches!(curso, DisputaEnCurso::PendienteContraargumentacion)
    }

    pub fn en_curso_pendiente_definicion(&self) -> bool {
        let EstadoDisputa::EnCurso(curso) = &self.estado
        else { return false; };

        matches!(curso, DisputaEnCurso::PendienteDefinicion)
    }

    /*
        Resuelta
     */

    pub fn resuelta(&self) -> bool {
        let EstadoDisputa::Resuelta(_) = &self.estado
        else { return false; };

        true
    }

    pub fn resuelta_favor_comprador(&self) -> bool {
        let EstadoDisputa::Resuelta(favor) = &self.estado
        else { return false; };

        matches!(favor, DisputaResuelta::FavorComprador { argumento_interventor: _ })
    }

    pub fn resuelta_favor_vendedor(&self) -> bool {
        let EstadoDisputa::Resuelta(favor) = &self.estado
        else { return false; };

        matches!(favor, DisputaResuelta::FavorVendedor { argumento_interventor: _ })
    }
}

impl RustaceoLibre {

    //

    /// Un comprador, después de recibir el producto, puede disputar su pedido aclarando el argumento para la disputa.
    /// Sólo el comprador puede hacer esto y el vendedor tiene la posibilidad de, con esta misma función,
    /// contraargumentar la disputa en su favor.
    /// 
    /// Sólo un miembro del Personal puede visualizar todas las disputas y darles finalización.
    /// Una disputa que no tenga contraargumento del vendedor deberá esperar 14 días
    /// para poder concluirse a favor del comprador.
    /// 
    /// Devolverá error si el usuario no existe, la compra no existe, no es el comprador o vendedor,
    /// es el vendedor y no hay una disputa, es el comprador y ya realizó la disputa o la disputa ya concluyó.
    pub fn _disputar_pedido(&mut self, timestamp: u64, caller: AccountId, id_pedido: u128, argumento: String) -> Result<(), ErrorDisputarPedido> {
        // validar usuario
        if !self.usuarios.contains(caller) {
            return Err(ErrorDisputarPedido::UsuarioNoRegistrado);
        }

        // validar pedido
        let Some(pedido) = self.pedidos.get(&id_pedido)
        else { return Err(ErrorDisputarPedido::PedidoInexistente); };

        // validar usuario participa en pedido
        if pedido.comprador != caller && pedido.vendedor != caller {
            return Err(ErrorDisputarPedido::UsuarioNoParticipa);
        }

        // validar disputa existente
        if let Some(id_disputa) = pedido.disputa {
            // la disputa existe. si caller es comprador, está volviendo a crear una disputa
            if caller == pedido.comprador {
                return Err(ErrorDisputarPedido::SoloVendedorPuedeContraargumentar);
            }

            // caller es vendedor. puede contraargumentar o ya lo hizo?
            let Some(disputa) = self.disputas_en_curso.get(&id_disputa)
            else { return Err(ErrorDisputarPedido::DisputaFinalizada); };

            if disputa.en_curso_pendiente_contraargumentacion() {
                // vendedor ya contraargumentó
                return Err(ErrorDisputarPedido::DisputaPendienteResolucion);
            }

            // caller es vendedor, quiere contraargumentar la disputa existente y puede.

            // modificar disputa
            let mut disputa = disputa.clone();
            disputa.argumento_vendedor = Some(argumento);
            disputa.estado = EstadoDisputa::EnCurso(DisputaEnCurso::PendienteDefinicion);

            // guardar disputa
            self.disputas_en_curso.insert(id_disputa, disputa);

            return Ok(());
        }

        // no existe disputa asociada al pedido

        // validar que quien inicia la disputa sea el comprador
        if caller != pedido.comprador {
            return Err(ErrorDisputarPedido::SoloCompradorPuedeDisputar);
        }

        // si los fondos del pedido ya fueron transferidos no hay disputa posible sin embargar (embargar es imposible)
        if pedido.fondos_fueron_transferidos {
            return Err(ErrorDisputarPedido::PlazoDeDisputaExpirado);
        }

        let mut pedido = pedido.clone();

        // crear nueva disputa
        let id_nueva_disputa = self.next_id_disputas();
        let nueva_disputa = Disputa {
            id: id_nueva_disputa,
            timestamp,
            pedido: id_pedido,
            estado: EstadoDisputa::EnCurso(DisputaEnCurso::PendienteContraargumentacion),
            argumento_comprador: argumento,
            argumento_vendedor: None,
            interventor: None
        };

        // guardar nueva disputa
        self.disputas_en_curso.insert(id_nueva_disputa, nueva_disputa);

        // actualizar disputas en curso en comprador
        if let Some(mut comprador) = self.usuarios.get(pedido.comprador) {
            comprador.agregar_disputa_comprador(id_nueva_disputa);
            self.usuarios.insert(comprador.id, &comprador);
        }

        // actualizar disputas en curso en vendedor
        if let Some(mut vendedor) = self.usuarios.get(pedido.vendedor) {
            vendedor.agregar_disputa_vendedor(id_nueva_disputa);
            self.usuarios.insert(vendedor.id, &vendedor);
        }

        // actualizar y guardar pedido para reflejar disputa
        pedido.disputa = Some(id_nueva_disputa);
        self.pedidos.insert(pedido.id, pedido);

        Ok(())
    }

    //

    /// Da una disputa por resuelta según la información que brinda el miembro del Staff.
    /// Entregará los fondos del pedido a quien corresponda.
    /// 
    /// Devolverá la información de pago correspondiente si la operación concretó correctamente.
    /// Devolverá None si no es miembro del Staff, la disputa no existe o no está en curso.
    pub fn _staff_resolver_disputa(&mut self, caller: AccountId, id_disputa: u128, resultado: DisputaResuelta) -> Result<(AccountId, u128), ErrorResolverDisputa> {
        // validar que caller sea staff ni owner
        if !self.staff.contains(&caller) && caller != self.owner {
            return Err(ErrorResolverDisputa::UsuarioNoStaff);
        }

        // validar que la disputa exista o esté en curso
        // si la disputa está en el listado de disputas en curso, no deberia poder estar resuelta
        let Some(mut disputa) = self.disputas_en_curso.get(&id_disputa).cloned()
        else { return Err(ErrorResolverDisputa::DisputaNoEnCurso); };

        // todo bien

        // eliminar de "en curso" de ambos usuarios (comprador y vendedor)
        let Some(mut pedido) = self.pedidos.get(&disputa.pedido).cloned()
        else {
            // el pedido no existe ¿? eliminar la disputa
            self.disputas_en_curso.remove_entry(&id_disputa);
            return Err(ErrorResolverDisputa::PedidoInexistente);
        };

        // actualizar disputa pendiente comprador
        if let Some(mut comprador) = self.usuarios.get(pedido.comprador) {
            comprador.eliminiar_disputa_comprador(id_disputa);
            self.usuarios.insert(comprador.id, &comprador);
        }

        // actualizar disputa pendiente vendedor
        if let Some(mut vendedor) = self.usuarios.get(pedido.vendedor) {
            vendedor.eliminiar_disputa_vendedor(id_disputa);
            self.usuarios.insert(vendedor.id, &vendedor);
        }

        // actualizar data de disputa
        disputa.estado = EstadoDisputa::Resuelta(resultado.clone());
        disputa.interventor = Some(caller);

        // eliminar de "en curso"
        self.disputas_en_curso.remove_entry(&id_disputa);

        // agregar a "resueltas"
        self.disputas_resueltas.insert(id_disputa, disputa.clone());

        // almacenar datos para después
        let id_comprador = pedido.comprador;
        let id_vendedor = pedido.vendedor;
        let valor_total = pedido.valor_total;

        // disputa: finalizar devolviendo fondos
        // Se deben devolver fondos en lib.rs. Actualizar pedido marcando los fondos como entregados.

        pedido.fondos_fueron_transferidos = true;
        self.pedidos.insert(disputa.pedido, pedido);

        let id_ganador: AccountId = match resultado {
            DisputaResuelta::FavorComprador{ argumento_interventor: _ } => id_comprador,
            DisputaResuelta::FavorVendedor{ argumento_interventor: _ } => id_vendedor
        };

        Ok((id_ganador, valor_total))
    }

}