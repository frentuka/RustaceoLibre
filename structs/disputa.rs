use ink::{prelude::string::String, primitives::AccountId};

use crate::{rustaceo_libre::RustaceoLibre, structs::pedido::EstadoPedido};

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
        if !self.usuarios.contains_key(&caller) {
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

            if disputa.en_curso_pendiente_definicion() {
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
        if let Some(mut comprador) = self.usuarios.get(&pedido.comprador).cloned() {
            comprador.agregar_disputa_comprador(id_nueva_disputa);
            self.usuarios.insert(comprador.id, comprador);
        }

        // actualizar disputas en curso en vendedor
        if let Some(mut vendedor) = self.usuarios.get(&pedido.vendedor).cloned() {
            vendedor.agregar_disputa_vendedor(id_nueva_disputa);
            self.usuarios.insert(vendedor.id, vendedor);
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
        if let Some(mut comprador) = self.usuarios.get(&pedido.comprador).cloned() {
            comprador.eliminiar_disputa_comprador(id_disputa);
            self.usuarios.insert(comprador.id, comprador);
        }

        // actualizar disputa pendiente vendedor
        if let Some(mut vendedor) = self.usuarios.get(&pedido.vendedor).cloned() {
            vendedor.eliminiar_disputa_vendedor(id_disputa);
            self.usuarios.insert(vendedor.id, vendedor);
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

#[cfg(test)]    

mod tests_disputas {
    use crate::structs::{
        pedido::{
            EstadoPedido,
            Pedido
        },
        
        usuario::RolDeSeleccion
    };

    use super::*;
    use ink::primitives::AccountId;

    fn acc(byte: u8) -> AccountId {
        AccountId::from([byte; 32])
    }

    fn pedido_base(id: u128, comprador: AccountId, vendedor: AccountId) -> Pedido {
        Pedido {
            id,
            timestamp: 0,
            publicacion: 0,
            cantidad_comprada: 1,
            valor_total: 100,
            fondos_fueron_transferidos: false,
            estado: EstadoPedido::Recibido(1234),
            comprador,
            vendedor,
            calificacion_comprador: None,
            calificacion_vendedor: None,
            disputa: None,
            primer_solicitud_cancelacion: None,
        }
    }

    #[ink::test]
    fn disputar_usuario_no_registrado() {
        let mut c = RustaceoLibre::new(0);
        let comprador = acc(1);
        let vendedor = acc(2);
        let tercero = acc(3);

        // Crear pedido
        let id_pedido = 1;
        c.pedidos.insert(id_pedido, pedido_base(id_pedido, comprador, vendedor));

        // tercero NO registrado
        let r = c._disputar_pedido(0, tercero, id_pedido, "hola".into());
        assert_eq!(r, Err(ErrorDisputarPedido::UsuarioNoRegistrado));
    }

    #[ink::test]
    fn disputar_pedido_inexistente() {
        let mut c = RustaceoLibre::new(0);
        let comprador = acc(1);

        c._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();

        let r = c._disputar_pedido(0, comprador, 999, "hola".into());
        assert_eq!(r, Err(ErrorDisputarPedido::PedidoInexistente));
    }

    #[ink::test]
    fn disputar_usuario_no_participa() {
        let mut c = RustaceoLibre::new(0);
        let comprador = acc(1);
        let vendedor = acc(2);
        let tercero = acc(3);
        let id_pedido = 10;

        c._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        c._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();
        c._registrar_usuario(tercero, RolDeSeleccion::Comprador).unwrap();

        c.pedidos.insert(id_pedido, pedido_base(id_pedido, comprador, vendedor));

        let r = c._disputar_pedido(0, tercero, id_pedido, "hola".into());
        assert_eq!(r, Err(ErrorDisputarPedido::UsuarioNoParticipa));
    }

    #[ink::test]
    fn disputar_solo_comprador_puede_iniciar() {
        let mut c = RustaceoLibre::new(0);
        let comprador = acc(1);
        let vendedor = acc(2);
        let id_pedido = 11;

        c._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        c._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        c.pedidos.insert(id_pedido, pedido_base(id_pedido, comprador, vendedor));

        // vendedor intenta iniciar disputa
        let r = c._disputar_pedido(0, vendedor, id_pedido, "contra".into());
        assert_eq!(r, Err(ErrorDisputarPedido::SoloCompradorPuedeDisputar));
    }

    #[ink::test]
    fn disputar_plazo_expirado_si_fondos_transferidos() {
        let mut c = RustaceoLibre::new(0);
        let comprador = acc(1);
        let vendedor = acc(2);
        let id_pedido = 12;

        c._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        c._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        let mut p = pedido_base(id_pedido, comprador, vendedor);
        p.fondos_fueron_transferidos = true;
        c.pedidos.insert(id_pedido, p);

        let r = c._disputar_pedido(0, comprador, id_pedido, "quiero disputar".into());
        assert_eq!(r, Err(ErrorDisputarPedido::PlazoDeDisputaExpirado));
    }

    #[ink::test]
    fn disputar_crea_disputa_ok() {
        let mut c = RustaceoLibre::new(0);
        let comprador = acc(1);
        let vendedor = acc(2);
        let id_pedido = 13;

        c._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        c._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        c.pedidos.insert(id_pedido, pedido_base(id_pedido, comprador, vendedor));

        let arg = "producto no llegó".to_string();
        let r = c._disputar_pedido(1000, comprador, id_pedido, arg.clone());
        assert_eq!(r, Ok(()));

        let pedido = c.pedidos.get(&id_pedido).unwrap();
        let id_disputa = pedido.disputa.expect("pedido debería tener disputa");

        let disputa = c.disputas_en_curso.get(&id_disputa).expect("disputa debe existir");
        assert_eq!(disputa.pedido, id_pedido);
        assert_eq!(disputa.timestamp, 1000);
        assert_eq!(disputa.argumento_comprador, arg);
        assert!(disputa.argumento_vendedor.is_none());
        assert!(matches!(
            disputa.estado,
            EstadoDisputa::EnCurso(DisputaEnCurso::PendienteContraargumentacion)
        ));
    }

    #[ink::test]
    fn disputar_con_disputa_existente_comprador_no_puede() {
        let mut c = RustaceoLibre::new(0);
        let comprador = acc(1);
        let vendedor = acc(2);
        let id_pedido = 14;

        c._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        c._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // Crear disputa ya asociada
        let id_disputa = c.next_id_disputas();
        let disputa = Disputa {
            id: id_disputa,
            timestamp: 0,
            pedido: id_pedido,
            estado: EstadoDisputa::EnCurso(DisputaEnCurso::PendienteContraargumentacion),
            argumento_comprador: "x".into(),
            argumento_vendedor: None,
            interventor: None,
        };
        c.disputas_en_curso.insert(id_disputa, disputa);

        let mut p = pedido_base(id_pedido, comprador, vendedor);
        p.disputa = Some(id_disputa);
        c.pedidos.insert(id_pedido, p);

        let r = c._disputar_pedido(0, comprador, id_pedido, "otra".into());
        assert_eq!(r, Err(ErrorDisputarPedido::SoloVendedorPuedeContraargumentar));
    }

    #[ink::test]
    fn disputar_contraargumento_vendedor_ok() {
        let mut c = RustaceoLibre::new(0);
        let comprador = acc(1);
        let vendedor = acc(2);
        let id_pedido = 15;

        c._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        c._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        let id_disputa = c.next_id_disputas();
        let disputa = Disputa {
            id: id_disputa,
            timestamp: 0,
            pedido: id_pedido,
            estado: EstadoDisputa::EnCurso(DisputaEnCurso::PendienteContraargumentacion),
            argumento_comprador: "x".into(),
            argumento_vendedor: None,
            interventor: None,
        };
        c.disputas_en_curso.insert(id_disputa, disputa);

        let mut p = pedido_base(id_pedido, comprador, vendedor);
        p.disputa = Some(id_disputa);
        c.pedidos.insert(id_pedido, p);

        let r = c._disputar_pedido(2000, vendedor, id_pedido, "yo sí envié".into());
        assert_eq!(r, Ok(()));

        let d = c.disputas_en_curso.get(&id_disputa).unwrap();
        assert_eq!(d.argumento_vendedor, Some("yo sí envié".into()));
        assert!(matches!(
            d.estado,
            EstadoDisputa::EnCurso(DisputaEnCurso::PendienteDefinicion)
        ));
    }

    #[ink::test]
    fn disputar_disputa_finalizada_si_no_esta_en_curso() {
        let mut c = RustaceoLibre::new(0);
        let comprador = acc(1);
        let vendedor = acc(2);
        let id_pedido = 16;

        c._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        c._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        // pedido tiene referencia a disputa pero no está en disputas_en_curso
        let mut p = pedido_base(id_pedido, comprador, vendedor);
        p.disputa = Some(999);
        c.pedidos.insert(id_pedido, p);

        let r = c._disputar_pedido(0, vendedor, id_pedido, "x".into());
        assert_eq!(r, Err(ErrorDisputarPedido::DisputaFinalizada));
    }

    // ---------------- staff resolver ----------------

    #[ink::test]
    fn staff_resolver_usuario_no_staff() {
        let mut c = RustaceoLibre::new(0);
        let cualquiera = acc(9);

        let r = c._staff_resolver_disputa(
            cualquiera,
            1,
            DisputaResuelta::FavorComprador { argumento_interventor: "x".into() }
        );
        assert_eq!(r, Err(ErrorResolverDisputa::UsuarioNoStaff));
    }

    #[ink::test]
    fn staff_resolver_disputa_no_en_curso() {
        let mut c = RustaceoLibre::new(0);

        // setear staff u owner según tu implementación:
        // ejemplo: c.owner = acc(1);
        // o: c.staff.push(acc(1));
        let staff = acc(1);
        c.staff.push(staff); // <-- ajustá si staff no es Vec

        let r = c._staff_resolver_disputa(
            staff,
            123,
            DisputaResuelta::FavorVendedor { argumento_interventor: "ok".into() }
        );
        assert_eq!(r, Err(ErrorResolverDisputa::DisputaNoEnCurso));
    }

    #[ink::test]
    fn staff_resolver_pedido_inexistente_elimina_disputa() {
        let mut c = RustaceoLibre::new(0);
        let staff = acc(1);
        c.staff.push(staff); // ajustar

        let id_disputa = 1;
        let d = Disputa {
            id: id_disputa,
            timestamp: 0,
            pedido: 9999, // pedido inexistente
            estado: EstadoDisputa::EnCurso(DisputaEnCurso::PendienteDefinicion),
            argumento_comprador: "x".into(),
            argumento_vendedor: Some("y".into()),
            interventor: None,
        };
        c.disputas_en_curso.insert(id_disputa, d);

        let r = c._staff_resolver_disputa(
            staff,
            id_disputa,
            DisputaResuelta::FavorComprador { argumento_interventor: "resuelvo".into() }
        );
        assert_eq!(r, Err(ErrorResolverDisputa::PedidoInexistente));

        // debe haberse eliminado de "en curso"
        assert!(c.disputas_en_curso.get(&id_disputa).is_none());
    }

    #[ink::test]
    fn staff_resolver_favor_comprador_ok() {
        let mut c = RustaceoLibre::new(0);
        let staff = acc(1);
        c.staff.push(staff); // ajustar

        let comprador = acc(2);
        let vendedor = acc(3);
        let id_pedido = 100;
        let id_disputa = 200;

        c._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        c._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        let mut p = pedido_base(id_pedido, comprador, vendedor);
        p.valor_total = 777;
        p.disputa = Some(id_disputa);
        c.pedidos.insert(id_pedido, p);

        let d = Disputa {
            id: id_disputa,
            timestamp: 0,
            pedido: id_pedido,
            estado: EstadoDisputa::EnCurso(DisputaEnCurso::PendienteDefinicion),
            argumento_comprador: "x".into(),
            argumento_vendedor: Some("y".into()),
            interventor: None,
        };
        c.disputas_en_curso.insert(id_disputa, d);

        let res = DisputaResuelta::FavorComprador { argumento_interventor: "a favor comprador".into() };
        let r = c._staff_resolver_disputa(staff, id_disputa, res.clone());
        assert_eq!(r, Ok((comprador, 777)));

        // en curso removida
        assert!(c.disputas_en_curso.get(&id_disputa).is_none());

        // resuelta guardada
        let d2 = c.disputas_resueltas.get(&id_disputa).unwrap();
        assert!(d2.resuelta_favor_comprador());
        assert_eq!(d2.interventor, Some(staff));

        // pedido fondos transferidos
        let p2 = c.pedidos.get(&id_pedido).unwrap();
        assert!(p2.fondos_fueron_transferidos);
    }

    #[ink::test]
    fn staff_resolver_favor_vendedor_ok() {
        let mut c = RustaceoLibre::new(0);
        let staff = acc(1);
        c.staff.push(staff); // ajustar

        let comprador = acc(2);
        let vendedor = acc(3);
        let id_pedido = 101;
        let id_disputa = 201;

        c._registrar_usuario(comprador, RolDeSeleccion::Comprador).unwrap();
        c._registrar_usuario(vendedor, RolDeSeleccion::Vendedor).unwrap();

        let mut p = pedido_base(id_pedido, comprador, vendedor);
        p.valor_total = 555;
        p.disputa = Some(id_disputa);
        c.pedidos.insert(id_pedido, p);

        let d = Disputa {
            id: id_disputa,
            timestamp: 0,
            pedido: id_pedido,
            estado: EstadoDisputa::EnCurso(DisputaEnCurso::PendienteDefinicion),
            argumento_comprador: "x".into(),
            argumento_vendedor: Some("y".into()),
            interventor: None,
        };
        c.disputas_en_curso.insert(id_disputa, d);

        let res = DisputaResuelta::FavorVendedor { argumento_interventor: "a favor vendedor".into() };
        let r = c._staff_resolver_disputa(staff, id_disputa, res.clone());
        assert_eq!(r, Ok((vendedor, 555)));
    }
}
