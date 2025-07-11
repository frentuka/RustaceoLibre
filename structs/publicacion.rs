//
// publicacion
//

use ink::primitives::AccountId;
use ink::prelude::vec::Vec;

use crate::rustaceo_libre::RustaceoLibre;

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub struct Publicacion {
    pub vendedor: AccountId,
    pub producto: u128,
    pub cantidad_ofertada: u32,
    pub precio: u128,
}

//
// impl Publicacion
//

impl Publicacion {
    pub fn new(vendedor: AccountId, producto: u128, cantidad_ofertada: u32, precio: u128) -> Self {
        Self {
            vendedor,
            producto,
            cantidad_ofertada,
            precio,
        }
    }
}

//
// impl Publicacion -> RustaceoLibre
//

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorRealizarPublicacion {
    UsuarioNoRegistrado,
    ProductoInexistente,
    NoEsVendedor,
    StockInsuficiente,
    PrecioCero,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorPausarReanudarPublicacion {
    UsuarioNoRegistrado,
    PublicacionInexistente,
    NoEsElVendedor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorVerPublicacionesVendedor {
    UsuarioNoRegistrado,
    NoEsVendedor,
    NoTienePublicaciones,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum ErrorModificarCantidadOfertada {
    UsuarioInexistente,
    NoEsVendedor,
    PublicacionInexistente,
    NoEsElVendedor,
    SinCambios,
    Desconocido,
    StockVendedorInsuficiente,
}

impl RustaceoLibre {
    /// Realiza una publicación con producto, precio y cantidad.
    /// 
    /// Devuelve Error si el precio o la cantidad son 0, o si `caller` no existe o no es vendedor.
    pub(crate) fn _realizar_publicacion(&mut self, caller: AccountId, id_producto: u128, cantidad_ofertada: u32, precio: u128) -> Result<u128, ErrorRealizarPublicacion> {
        // verificar precio
        if precio == 0 {
            return Err(ErrorRealizarPublicacion::PrecioCero);
        }

        // validar usuario
        let Some(mut usuario) = self.usuarios.get(caller) else {
            return Err(ErrorRealizarPublicacion::UsuarioNoRegistrado);
        };

        // validar que sea vendedor
        if !usuario.es_vendedor() {
            return Err(ErrorRealizarPublicacion::NoEsVendedor);
        }

        // verificar que haya stock (en el vendedor)
        let Some(stock_vendedor) = usuario.obtener_stock_producto(&id_producto)
        else { return Err(ErrorRealizarPublicacion::StockInsuficiente) };

        if stock_vendedor < cantidad_ofertada {
            return Err(ErrorRealizarPublicacion::StockInsuficiente);
        }

        // último check: verificar que el producto exista
        let Some(_) = self.productos.get(&id_producto)
        else { return Err(ErrorRealizarPublicacion::ProductoInexistente); };

        // sustraer cantidad ofertada del stock del vendedor (pasará a formar parte de la oferta de la publicación)
        let Some(nuevo_stock_vendedor) = stock_vendedor.checked_sub(cantidad_ofertada)
        else { return Err(ErrorRealizarPublicacion::StockInsuficiente); };
        usuario.establecer_stock_producto(&id_producto, &nuevo_stock_vendedor);

        // obtener id de publicación e instanciarla
        let id_publicacion = self.next_id_publicaciones();
        let publicacion = Publicacion::new(caller, id_producto, cantidad_ofertada, precio);

        // agregar al map principal
        self.publicaciones.insert(id_publicacion, publicacion);

        // agregar al vendedor
        usuario.agregar_publicacion(id_publicacion);
        self.usuarios.insert(usuario.id, &usuario);

        // fin
        Ok(id_publicacion)
    }

    /// Modifica la cantidad ofertada en una publicación,
    /// modificando también el stock del vendedor.
    /// 
    /// Devuelve Error si el usuario no está registrado, la venta no existe,
    /// el usuario no es el vendedor o la operación es imposible por falta de stock/cantidad ofertada.
    pub(crate) fn _modificar_cantidad_ofertada(&mut self, caller: AccountId, id_publicacion: u128, nueva_cantidad_ofertada: u32) -> Result<(), ErrorModificarCantidadOfertada> {
        let Some(usuario) = self.usuarios.get(caller)
        else { return Err(ErrorModificarCantidadOfertada::UsuarioInexistente); };

        if !usuario.es_vendedor() {
            return Err(ErrorModificarCantidadOfertada::NoEsVendedor);
        }

        let Some(publicacion) = self.publicaciones.get(&id_publicacion)
        else { return Err(ErrorModificarCantidadOfertada::PublicacionInexistente); };

        if publicacion.vendedor != caller {
            return Err(ErrorModificarCantidadOfertada::NoEsElVendedor);
        }

        if nueva_cantidad_ofertada == publicacion.cantidad_ofertada {
            return Err(ErrorModificarCantidadOfertada::SinCambios);
        }

        let Some(stock_vendedor) = usuario.obtener_stock_producto(&publicacion.producto)
        else { return Err(ErrorModificarCantidadOfertada::StockVendedorInsuficiente) };

        let nuevo_stock_vendedor: u32;
        // op. 1: stock se transfiere del vendedor a la publicacion
        if nueva_cantidad_ofertada > publicacion.cantidad_ofertada {
            let Some(diff) = nueva_cantidad_ofertada.checked_sub(publicacion.cantidad_ofertada)
            else { return Err(ErrorModificarCantidadOfertada::Desconocido); };

            let Some(nuevo_stock_vendedor_int) = stock_vendedor.checked_sub(diff)
            else { return Err(ErrorModificarCantidadOfertada::StockVendedorInsuficiente) };
            
            nuevo_stock_vendedor = nuevo_stock_vendedor_int;
        } else { // op. 2: stock se transfiere de la publicacion al vendedor
            let Some(diff) = publicacion.cantidad_ofertada.checked_sub(nueva_cantidad_ofertada)
            else { return Err(ErrorModificarCantidadOfertada::Desconocido); };

            let Some(nuevo_stock_vendedor_int) = stock_vendedor.checked_add(diff)
            else { return Err(ErrorModificarCantidadOfertada::Desconocido); };

            nuevo_stock_vendedor = nuevo_stock_vendedor_int;
        }

        // todo perfecto: ejecutar cambios
        let mut publicacion = publicacion.clone();
        let mut usuario = usuario;
        publicacion.cantidad_ofertada = nueva_cantidad_ofertada;
        usuario.establecer_stock_producto(&publicacion.producto, &nuevo_stock_vendedor);

        Ok(())
    }

    //

    /// Dada una ID, devuelve la publicación
    /// 
    /// Devolverá None si la publicación no existe o el usuario no está registrado
    pub(crate) fn _ver_publicacion(&self, caller: AccountId, id_publicacion: u128) -> Option<Publicacion> {
        if !self.usuarios.contains(caller) {
            return None;
        }
        self.publicaciones.get(&id_publicacion).cloned()
    }

    //

    /// Devuelve todos los productos que correspondan al vendedor que ejecute esta función.
    /// 
    /// Dará error si el usuario no existe, no está registrado como vendedor o no tiene publicaciones.
    pub(crate) fn _ver_publicaciones_vendedor(&self, caller: AccountId) -> Result<Vec<Publicacion>, ErrorVerPublicacionesVendedor> {
        let Some(caller) = self.usuarios.get(caller)
        else { return Err(ErrorVerPublicacionesVendedor::UsuarioNoRegistrado); };

        if !caller.es_vendedor() {
            return Err(ErrorVerPublicacionesVendedor::NoEsVendedor);
        }

        let Some(publicaciones) = caller.obtener_publicaciones()
        else { return Err(ErrorVerPublicacionesVendedor::NoTienePublicaciones); };

        if publicaciones.is_empty() {
            return Err(ErrorVerPublicacionesVendedor::NoTienePublicaciones);
        }

        // todo bien
        // mapear las publicaciones: Vec<u128> -> Vec<Producto> y devolver

        let vec_publicaciones: Vec<Publicacion> = publicaciones.iter().filter_map(| p | {
            self.publicaciones.get(p)
        }).cloned().collect();

        // si no está vacío, devolver
        if vec_publicaciones.is_empty() {
            Err(ErrorVerPublicacionesVendedor::NoTienePublicaciones)
        } else {
            Ok(vec_publicaciones)
        }
    }
}