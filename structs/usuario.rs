use ink::{prelude::vec::Vec, primitives::AccountId};

use crate::rustaceo_libre::RustaceoLibre;

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
    pub compraventas: Vec<u128>, // Guarda compras y ventas. Si vendedor == id, Usuario es el vendedor. Caso contrario, es comprador.
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
            publicaciones: None,
        }
    }

    pub fn es_comprador(&self) -> bool {
        self.rol == Rol::Comprador || self.rol == Rol::Ambos
    }

    pub fn es_vendedor(&self) -> bool {
        self.rol == Rol::Vendedor || self.rol == Rol::Ambos
    }
}

//
// impl Usuario -> RustaceoLibre
//

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorRegistrarUsuario {
    UsuarioYaExiste,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorModificarRolUsuario {
    UsuarioInexistente,
    MismoRolAsignado,
}

impl RustaceoLibre {
    /// Registra un usuario en el Mapping de usuarios.
    /// 
    /// Devuelve error si el usuario ya existe.
    pub fn _registrar_usuario(&mut self, caller: AccountId, rol: Rol) -> Result<(), ErrorRegistrarUsuario>  {
        // el usuario no puede ya existir
        if self.usuarios.contains(caller) {
            return Err(ErrorRegistrarUsuario::UsuarioYaExiste)
        }

        let usuario = Usuario::new(caller, rol);
        self.usuarios.insert(caller, &usuario); // por algún motivo es un préstamo, se supone que se clona.
        Ok(())
    }

    /// Recibe un rol y lo modifica para ese usuario si ya está registrado.
    /// 
    /// Devuelve error si el usuario no existe o ya posee ese rol.
    pub fn _modificar_rol_usuario(&mut self, caller: AccountId, rol: Rol) -> Result<(), ErrorModificarRolUsuario> {
        // si no existe, es imposible modificar
        let Some(usuario) = self.usuarios.get(caller)
        else {
            return Err(ErrorModificarRolUsuario::UsuarioInexistente);
        };

        // si ya posee ese rol, hacerlo saber
        if usuario.rol == rol {
            return Err(ErrorModificarRolUsuario::MismoRolAsignado)
        }

        // todo bien: asignar nuevo rol. creo que no se pueden hacer modificaciones directas
        // normalmente obtendría usuario como mut (self.usuarios.get_mut()) y modificaría esa instancia mutable
        // tengo entendido que no es posible, hay que modificar una copia y reemplazarlo
        let mut usuario = usuario;
        usuario.rol = rol;
        self.usuarios.insert(caller, &usuario);

        Ok(())
    }
}