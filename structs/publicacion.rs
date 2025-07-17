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
    pub precio_unitario: u128,
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
            precio_unitario: precio,
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

        //verificar cantidad ofertada
        if cantidad_ofertada == 0{
            return Err(ErrorRealizarPublicacion::StockInsuficiente);
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

        // actualizado:
        // se modificaba publicacion.cantidad_ofertada localmente, pero no se guardaba la publicación actualizada en self.publicaciones ni en self.usuarios
        self.publicaciones.insert(id_publicacion, publicacion);
        self.usuarios.insert(usuario.id, &usuario);

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


#[cfg(test)]
mod tests {
    
    use super::*;
    use ink::primitives::AccountId;
    use crate::structs::{producto::{CategoriaProducto, Producto}, usuario::{DataComprador, DataVendedor, Rol, StockProductos, Usuario}};

    #[test]
    fn test_publicacion_new_success() {
        let vendedor = AccountId::from([0x1; 32]);
        let publicacion = Publicacion::new(
            vendedor,
            1,
            10,
            100,
        );
        assert_eq!(publicacion.vendedor, vendedor);
        assert_eq!(publicacion.producto, 1);
        assert_eq!(publicacion.cantidad_ofertada, 10);
        assert_eq!(publicacion.precio_unitario, 100);
    }

    #[ink::test]
    fn test_realizar_publicacion_precio_cero() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);
        let mut usuario = Usuario::new(caller, Rol::Vendedor(DataVendedor {
            ventas: Vec::new(),
            publicaciones: Vec::new(),
            stock_productos: StockProductos::default(),
            total_calificaciones: 0,
            cant_calificaciones: 0,
        }));
        usuario.establecer_stock_producto(&1, &15); // Configura stock inicial
        rustaceo.usuarios.insert(caller, &usuario);

        let result = rustaceo._realizar_publicacion(caller, 1, 10, 0);
        assert!(matches!(result, Err(ErrorRealizarPublicacion::PrecioCero)));
    }

    #[ink::test]
    fn test_realizar_publicacion_cantidad_cero() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);
        let mut usuario = Usuario::new(caller, Rol::Vendedor(DataVendedor {
            ventas: Vec::new(),
            publicaciones: Vec::new(),
            stock_productos: StockProductos::default(),
            total_calificaciones: 0,
            cant_calificaciones: 0,
        }));
        usuario.establecer_stock_producto(&1, &15); // Configura stock inicial
        rustaceo.usuarios.insert(caller, &usuario);

        let result = rustaceo._realizar_publicacion(caller, 1, 0, 100);
        assert!(matches!(result, Err(ErrorRealizarPublicacion::StockInsuficiente))); // Cantidad 0 implica stock insuficiente
    }

    #[ink::test]
    fn test_realizar_publicacion_usuario_no_registrado() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);

        let result = rustaceo._realizar_publicacion(caller, 1, 10, 100);
        assert!(matches!(result, Err(ErrorRealizarPublicacion::UsuarioNoRegistrado)));
    }

    #[ink::test]
    fn test_realizar_publicacion_no_es_vendedor() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);
        let usuario = Usuario::new(caller, Rol::Comprador(DataComprador {
            compras: Vec::new(),
            total_calificaciones: 0,
            cant_calificaciones: 0,
        }));
        rustaceo.usuarios.insert(caller, &usuario);

        let result = rustaceo._realizar_publicacion(caller, 1, 10, 100);
        assert!(matches!(result, Err(ErrorRealizarPublicacion::NoEsVendedor)));
    }

    #[ink::test]
    fn test_realizar_publicacion_stock_insuficiente() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);
        let mut usuario = Usuario::new(caller, Rol::Vendedor(DataVendedor {
            ventas: Vec::new(),
            publicaciones: Vec::new(),
            stock_productos: StockProductos::default(),
            total_calificaciones: 0,
            cant_calificaciones: 0,
        }));
        usuario.establecer_stock_producto(&1, &5); // Stock menor que cantidad ofertada
        rustaceo.usuarios.insert(caller, &usuario);
        rustaceo.productos.insert(1, Producto::new(String::from("Test"), String::from("Desc"), CategoriaProducto::Hogar));

        let result = rustaceo._realizar_publicacion(caller, 1, 10, 100);
        assert!(matches!(result, Err(ErrorRealizarPublicacion::StockInsuficiente)));
    }

    #[ink::test]
    fn test_realizar_publicacion_producto_inexistente() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);
        let mut usuario = Usuario::new(caller, Rol::Vendedor(DataVendedor {
            ventas: Vec::new(),
            publicaciones: Vec::new(),
            stock_productos: StockProductos::default(),
            total_calificaciones: 0,
            cant_calificaciones: 0,
        }));
        usuario.establecer_stock_producto(&1, &15); // Stock suficiente
        rustaceo.usuarios.insert(caller, &usuario);

        let result = rustaceo._realizar_publicacion(caller, 1, 10, 100);
        assert!(matches!(result, Err(ErrorRealizarPublicacion::ProductoInexistente)));
    }

    #[ink::test]
    fn test_realizar_publicacion_success() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);
        let mut usuario = Usuario::new(caller, Rol::Vendedor(DataVendedor {
            ventas: Vec::new(),
            publicaciones: Vec::new(),
            stock_productos: StockProductos::default(),
            total_calificaciones: 0,
            cant_calificaciones: 0,
        }));
        usuario.establecer_stock_producto(&1, &15); // Stock inicial > cantidad ofertada
        rustaceo.usuarios.insert(caller, &usuario);
        rustaceo.productos.insert(1, Producto::new(String::from("Test"), String::from("Desc"), CategoriaProducto::Hogar));

        let result = rustaceo._realizar_publicacion(caller, 1, 10, 100);
        assert!(result.is_ok());
        let id = result.unwrap();
        assert_eq!(id, 0); // Primer ID generado
        let publicacion = rustaceo.publicaciones.get(&id).unwrap();
        assert_eq!(publicacion.vendedor, caller);
        assert_eq!(publicacion.producto, 1);
        assert_eq!(publicacion.cantidad_ofertada, 10);
        assert_eq!(publicacion.precio_unitario, 100);
        let updated_user = rustaceo.usuarios.get(caller).unwrap();
        let updated_stock = updated_user.obtener_stock_producto(&1).unwrap();
        assert_eq!(updated_stock, 5); // 15 - 10
    }

    #[ink::test]
    fn test_modificar_cantidad_ofertada_usuario_inexistente() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);

        let result = rustaceo._modificar_cantidad_ofertada(caller, 0, 15);
        assert!(matches!(result, Err(ErrorModificarCantidadOfertada::UsuarioInexistente)));
    }

    #[ink::test]
    fn test_modificar_cantidad_ofertada_no_es_vendedor() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);
        let usuario = Usuario::new(caller, Rol::Comprador(DataComprador {
            compras: Vec::new(),
            total_calificaciones: 0,
            cant_calificaciones: 0,
        }));
        rustaceo.usuarios.insert(caller, &usuario);

        let result = rustaceo._modificar_cantidad_ofertada(caller, 0, 15);
        assert!(matches!(result, Err(ErrorModificarCantidadOfertada::NoEsVendedor)));
    }

    #[ink::test]
    fn test_modificar_cantidad_ofertada_publicacion_inexistente() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);
        let usuario = Usuario::new(caller, Rol::Vendedor(DataVendedor {
            ventas: Vec::new(),
            publicaciones: Vec::new(),
            stock_productos: StockProductos::default(),
            total_calificaciones: 0,
            cant_calificaciones: 0,
        }));
        rustaceo.usuarios.insert(caller, &usuario);

        let result = rustaceo._modificar_cantidad_ofertada(caller, 0, 15);
        assert!(matches!(result, Err(ErrorModificarCantidadOfertada::PublicacionInexistente)));
    }

    #[ink::test]
    fn test_modificar_cantidad_ofertada_no_es_el_vendedor() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);
        let otro_vendedor = AccountId::from([0x2; 32]);
        let usuario = Usuario::new(caller, Rol::Vendedor(DataVendedor {
            ventas: Vec::new(),
            publicaciones: Vec::new(),
            stock_productos: StockProductos::default(),
            total_calificaciones: 0,
            cant_calificaciones: 0,
        }));
        rustaceo.usuarios.insert(caller, &usuario);
        let publicacion = Publicacion::new(otro_vendedor, 1, 10, 100);
        rustaceo.publicaciones.insert(0, publicacion);

        let result = rustaceo._modificar_cantidad_ofertada(caller, 0, 15);
        assert!(matches!(result, Err(ErrorModificarCantidadOfertada::NoEsElVendedor)));
    }

    #[ink::test]
    fn test_modificar_cantidad_ofertada_sin_cambios() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);
        let mut usuario = Usuario::new(caller, Rol::Vendedor(DataVendedor {
            ventas: Vec::new(),
            publicaciones: Vec::new(),
            stock_productos: StockProductos::default(),
            total_calificaciones: 0,
            cant_calificaciones: 0,
        }));
        usuario.establecer_stock_producto(&1, &15);
        rustaceo.usuarios.insert(caller, &usuario);
        let publicacion = Publicacion::new(caller, 1, 10, 100);
        rustaceo.publicaciones.insert(0, publicacion);

        let result = rustaceo._modificar_cantidad_ofertada(caller, 0, 10);
        assert!(matches!(result, Err(ErrorModificarCantidadOfertada::SinCambios)));
    }

    #[ink::test]
    fn test_modificar_cantidad_ofertada_success_increase() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);
        let mut usuario = Usuario::new(caller, Rol::Vendedor(DataVendedor {
            ventas: Vec::new(),
            publicaciones: Vec::new(),
            stock_productos: StockProductos::default(),
            total_calificaciones: 0,
            cant_calificaciones: 0,
        }));
        usuario.establecer_stock_producto(&1, &15); // Stock suficiente
        rustaceo.usuarios.insert(caller, &usuario);
        let publicacion = Publicacion::new(caller, 1, 10, 100);
        rustaceo.publicaciones.insert(0, publicacion);
        rustaceo.productos.insert(1, Producto::new(String::from("Test"), String::from("Desc"), CategoriaProducto::Hogar));

        let result = rustaceo._modificar_cantidad_ofertada(caller, 0, 12); // Aumenta de 10 a 12
        assert!(result.is_ok());
        let updated_pub = rustaceo.publicaciones.get(&0).unwrap();
        assert_eq!(updated_pub.cantidad_ofertada, 12);
        let updated_user = rustaceo.usuarios.get(caller).unwrap();
        let updated_stock = updated_user.obtener_stock_producto(&1).unwrap();
        assert_eq!(updated_stock, 13); // 15 - (12 - 10) = 13
    }

    #[ink::test]
    fn test_modificar_cantidad_ofertada_success_decrease() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);
        let mut usuario = Usuario::new(caller, Rol::Vendedor(DataVendedor {
            ventas: Vec::new(),
            publicaciones: Vec::new(),
            stock_productos: StockProductos::default(),
            total_calificaciones: 0,
            cant_calificaciones: 0,
        }));
        usuario.establecer_stock_producto(&1, &5); // Stock inicial
        rustaceo.usuarios.insert(caller, &usuario);
        let publicacion = Publicacion::new(caller, 1, 10, 100);
        rustaceo.publicaciones.insert(0, publicacion);
        rustaceo.productos.insert(1, Producto::new(String::from("Test"), String::from("Desc"), CategoriaProducto::Hogar));

        let result = rustaceo._modificar_cantidad_ofertada(caller, 0, 8); // Disminuye de 10 a 8
        assert!(result.is_ok());
        let updated_pub = rustaceo.publicaciones.get(&0).unwrap();
        assert_eq!(updated_pub.cantidad_ofertada, 8); // Now should pass with insert
        let updated_user = rustaceo.usuarios.get(caller).unwrap();
        let updated_stock = updated_user.obtener_stock_producto(&1).unwrap();
        assert_eq!(updated_stock, 7); // 5 + (10 - 8) = 7
    }

    #[ink::test]
    fn test_modificar_cantidad_ofertada_stock_vendedor_insuficiente() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);
        let mut usuario = Usuario::new(caller, Rol::Vendedor(DataVendedor {
            ventas: Vec::new(),
            publicaciones: Vec::new(),
            stock_productos: StockProductos::default(),
            total_calificaciones: 0,
            cant_calificaciones: 0,
        }));
        usuario.establecer_stock_producto(&1, &5); // Stock insuficiente
        rustaceo.usuarios.insert(caller, &usuario);
        let publicacion = Publicacion::new(caller, 1, 10, 100);
        rustaceo.publicaciones.insert(0, publicacion);
        rustaceo.productos.insert(1, Producto::new(String::from("Test"), String::from("Desc"), CategoriaProducto::Hogar));

        let result = rustaceo._modificar_cantidad_ofertada(caller, 0, 15); // Aumenta a 15
        assert!(result.is_ok());
        let updated_pub = rustaceo.publicaciones.get(&0).unwrap();
        assert_eq!(updated_pub.cantidad_ofertada, 15); // Verifica que se actualice a 15
        let updated_user = rustaceo.usuarios.get(caller).unwrap();
        let updated_stock = updated_user.obtener_stock_producto(&1).unwrap();
        assert_eq!(updated_stock, 0); // 5 - 5 = 0
    }

    #[ink::test]
    fn test_ver_publicacion_usuario_no_registrado() {
        let rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);

        let result = rustaceo._ver_publicacion(caller, 0);
        assert!(result.is_none());
    }

    #[ink::test]
    fn test_ver_publicacion_publicacion_inexistente() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);
        let usuario = Usuario::new(caller, Rol::Vendedor(DataVendedor {
            ventas: Vec::new(),
            publicaciones: Vec::new(),
            stock_productos: StockProductos::default(),
            total_calificaciones: 0,
            cant_calificaciones: 0,
        }));
        rustaceo.usuarios.insert(caller, &usuario);

        let result = rustaceo._ver_publicacion(caller, 0);
        assert!(result.is_none());
    }

    #[ink::test]
    fn test_ver_publicaciones_vendedor_usuario_no_registrado() {
        let rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);

        let result = rustaceo._ver_publicaciones_vendedor(caller);
        assert!(matches!(result, Err(ErrorVerPublicacionesVendedor::UsuarioNoRegistrado)));
    }

    #[ink::test]
    fn test_ver_publicaciones_vendedor_no_es_vendedor() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);
        let usuario = Usuario::new(caller, Rol::Comprador(DataComprador {
            compras: Vec::new(),
            total_calificaciones: 0,
            cant_calificaciones: 0,
        }));
        rustaceo.usuarios.insert(caller, &usuario);

        let result = rustaceo._ver_publicaciones_vendedor(caller);
        assert!(matches!(result, Err(ErrorVerPublicacionesVendedor::NoEsVendedor)));
    }
    
    #[ink::test]
    fn test_ver_publicaciones_vendedor_no_tiene_publicaciones() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);
        let usuario = Usuario::new(caller, Rol::Vendedor(DataVendedor {
            ventas: Vec::new(),
            publicaciones: Vec::new(), // Sin publicaciones
            stock_productos: StockProductos::default(),
            total_calificaciones: 0,
            cant_calificaciones: 0,
        }));
        rustaceo.usuarios.insert(caller, &usuario);

        let result = rustaceo._ver_publicaciones_vendedor(caller);
        assert!(matches!(result, Err(ErrorVerPublicacionesVendedor::NoTienePublicaciones)));
    }

    #[ink::test]
    fn test_ver_publicaciones_vendedor_success() {
        let mut rustaceo = RustaceoLibre::new();
        let caller = AccountId::from([0x1; 32]);
        let usuario = Usuario::new(caller, Rol::Vendedor(DataVendedor {
            ventas: Vec::new(),
            publicaciones: vec![0], // Con una publicación
            stock_productos: StockProductos::default(),
            total_calificaciones: 0,
            cant_calificaciones: 0,
        }));
        rustaceo.usuarios.insert(caller, &usuario);
        let publicacion = Publicacion::new(caller, 1, 10, 100);
        rustaceo.publicaciones.insert(0, publicacion);

        let result = rustaceo._ver_publicaciones_vendedor(caller);
        assert!(result.is_ok());
        let publicaciones = result.unwrap();
        assert_eq!(publicaciones.len(), 1);
        let publicacion = &publicaciones[0];
        assert_eq!(publicacion.vendedor, caller);
        assert_eq!(publicacion.producto, 1);
        assert_eq!(publicacion.cantidad_ofertada, 10);
        assert_eq!(publicacion.precio_unitario, 100);
    }
}