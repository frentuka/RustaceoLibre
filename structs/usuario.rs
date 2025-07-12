use ink::{prelude::vec::Vec, primitives::AccountId};

use crate::rustaceo_libre::RustaceoLibre;

//
// struct custom para almacenar stock de productos
// no puedo utilizar Mapping en Usuario porque no implementa el trait Decode
// los datos son privados porque se deben manejar con las funciones del impl StockProductos
//

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub struct StockProductos {
    productos: Vec<u128>,
    stock: Vec<u32>
}

impl StockProductos {
    /// Devuelve el stock asociado a la ID del producto brindada
    fn get(&self, id_producto: &u128) -> Option<u32> {
        let Ok(index) = self.productos.binary_search(id_producto)
        else { return None };
        self.stock.get(index).copied()
    }

    /// Inserta el producto en el vector doble. Si ya existe un producto con ese valor, lo sobreescribe.
    /// Si no existe un producto con ese valor, inserta al final de la lista.
    fn insert(&mut self, id_producto: u128, stock: u32) {
        let index = self.productos.binary_search(&id_producto);
        let index = match index {
            Ok(index) => index,
            Err(index) => {
                // ya existe: eliminar ocurrencia actual
                self.productos.remove(index);
                self.stock.remove(index);
                index
            }
        };

        // insertar en su posición correspondiente
        self.productos.insert(index, id_producto);
        self.stock.insert(index, stock);
    }
}

//
// data compra
//

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub struct DataComprador {
    pub compras: Vec<u128>,
    pub total_calificaciones: u64,
    pub cant_calificaciones: u32,
}

//
// data vendedor
//

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub struct DataVendedor {
    pub ventas: Vec<u128>,
    pub publicaciones: Vec<u128>,
    pub stock_productos: StockProductos,
    pub total_calificaciones: u64,
    pub cant_calificaciones: u32,
}

//
// rol
//

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout)
)]
pub enum Rol {
    Comprador(DataComprador), // compras
    Vendedor(DataVendedor), // ventas, publicaciones, stock_productos
    Ambos(DataComprador, DataVendedor), // compras, ventas, publicaciones, stock_productos
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
}

//
// impl usuario
//

impl Usuario {
    /// Inicializa una nueva instancia de Usuario
    pub fn new(id: AccountId, rol: Rol) -> Self {
        Self {
            id,
            rol,
        }
    }

    /// Devuelve true si el rol del usuario es Comprador o Ambos
    /// Devuelve false en caso contrario
    pub fn es_comprador(&self) -> bool {
        match &self.rol {
            Rol::Vendedor(_) => false,
            _ => true,
        }
    }

    /// Devuelve true si el rol del usuario es Vendedor o Ambos
    /// Devuelve false en caso contrario
    pub fn es_vendedor(&self) -> bool {
        match &self.rol {
            Rol::Comprador(_) => false,
            _ => true,
        }
    }

    /// Devuelve el registro completo de DataVendedor del usuario.
    /// Devolverá None si no es vendedor.
    pub fn obtener_data_comprador(&self) -> Option<DataComprador> {
        match &self.rol {
            Rol::Comprador(data_comprador) => return Some(data_comprador.clone()),
            Rol::Vendedor(_) => return None,
            Rol::Ambos(data_comprador, _) => return Some(data_comprador.clone()),
        }
    }

    /// Devuelve las compras que haya realizado el usuario.
    /// Devolverá None si no es comprador.
    pub fn obtener_compras(&self) -> Option<Vec<u128>> {
        self.obtener_data_comprador().map(|data| data.compras)
    }

    /// Devuelve el registro completo de DataVendedor del usuario.
    /// Devolverá None si no es vendedor.
    pub fn obtener_data_vendedor(&self) -> Option<DataVendedor> {
        match &self.rol {
            Rol::Comprador(_) => return None,
            Rol::Vendedor(data_vendedor) => return Some(data_vendedor.clone()),
            Rol::Ambos(_, data_vendedor) => return Some(data_vendedor.clone()),
        }
    }

    /// Devuelve las ventas que haya realizado el usuario.
    /// Devolverá None si no es vendedor.
    pub fn obtener_ventas(&self) -> Option<Vec<u128>> {
        self.obtener_data_vendedor().map(|data| data.ventas)
    }

    /// Devuelve las publicaciones que haya realizado el usuario.
    /// Devolverá None si no es vendedor.
    pub fn obtener_publicaciones(&self) -> Option<Vec<u128>> {
        self.obtener_data_vendedor().map(|data| data.publicaciones)
    }

    /// Devuelve el stock de todos los productos que tenga el usuario.
    /// 
    /// Devolverá None si no es vendedor.
    pub fn obtener_stock_productos(&self) -> Option<StockProductos> {
        self.obtener_data_vendedor().map(|data| data.stock_productos)
    }

    /// Devuelve el stock de un producto que tenga el usuario.
    /// 
    /// Devolverá None si no es vendedor o si el mismo no tiene registro de stock del producto.
    pub fn obtener_stock_producto(&self, id_producto: &u128) -> Option<u32> {
        let stocks = self.obtener_stock_productos()?;
        stocks.get(id_producto)
    }

    /// Añade una compra al vector de compras del rol del usuario.
    /// 
    /// Devuelve true si la compra pudo agregarse.
    /// Devolverá false si esa compra ya está añadida o el usuario no es comprador.
    /// No verifica que la compra exista.
    pub fn agregar_compra(&mut self, id_compra: u128) -> bool {
        let Some(mut data_comprador) = self.obtener_data_comprador()
        else { return false; };

        data_comprador.compras.push(id_compra);
        let nuevo_rol = match &self.rol {
            Rol::Comprador(_) => Rol::Comprador(data_comprador),
            Rol::Vendedor(_) => return false, // no debería nunca poder pasar
            Rol::Ambos(_, data_vendedor) => Rol::Ambos(data_comprador, data_vendedor.clone()),
        };

        self.rol = nuevo_rol;
        true
    }

    /// Añade una venta al vector de ventas del rol del usuario.
    /// 
    /// Devuelve true si la venta pudo agregarse.
    /// Devolverá false si esa venta ya está añadida o el usuario no es vendedor.
    /// No verifica que la venta exista.
    pub fn agregar_venta(&mut self, id_venta: u128) -> bool {
        let Some(mut nuevo_data_vendedor) = self.obtener_data_vendedor()
        else { return false; };

        nuevo_data_vendedor.ventas.push(id_venta);
        let nuevo_rol = match &self.rol {
            Rol::Comprador(_) => return false,
            Rol::Vendedor(_) => Rol::Vendedor(nuevo_data_vendedor), // no debería nunca poder pasar
            Rol::Ambos(compras, _) => Rol::Ambos(compras.clone(), nuevo_data_vendedor),
        };

        self.rol = nuevo_rol;
        true
    }

    /// Añade una publicación al vector de publicaciones del rol del usuario.
    /// 
    /// Devuelve true si la publicación pudo agregarse.
    /// Devolverá false si esa publicación ya está añadida o el usuario no es vendedor.
    /// No verifica que la publicación exista.
    pub fn agregar_publicacion(&mut self, id_publicacion: u128) -> bool {
        let Some(mut nuevo_data_vendedor) = self.obtener_data_vendedor()
        else { return false; };

        nuevo_data_vendedor.publicaciones.push(id_publicacion);
        let nuevo_rol = match &self.rol {
            Rol::Comprador(_) => return false,
            Rol::Vendedor(_) => Rol::Vendedor(nuevo_data_vendedor), // no debería nunca poder pasar
            Rol::Ambos(compras, _) => Rol::Ambos(compras.clone(), nuevo_data_vendedor),
        };

        self.rol = nuevo_rol;
        true
    }

    /// Modifica el stock de un producto en el mapa de stocks del rol del usuario.
    /// 
    /// Devuelve true si la el stock pudo modificarse.
    /// Devolverá false si el usuario no es vendedor.
    /// No verifica que el producto exista.
    pub fn establecer_stock_producto(&mut self, id_producto: &u128, stock: &u32) -> bool {
        let Some(mut nuevo_data_vendedor) = self.obtener_data_vendedor()
        else { return false; };

        nuevo_data_vendedor.stock_productos.insert(*id_producto, *stock);
        let nuevo_rol = match &self.rol {
            Rol::Comprador(_) => return false,
            Rol::Vendedor(_) => Rol::Vendedor(nuevo_data_vendedor), // no debería nunca poder pasar
            Rol::Ambos(compras, _) => Rol::Ambos(compras.clone(), nuevo_data_vendedor),
        };

        self.rol = nuevo_rol;
        true
    }

    /// Calificar al usuario como comprador
    /// 
    /// Devolverá true si la operación fue exitosa o false en caso contrario.
    pub fn calificar_como_comprador(&mut self, calificacion: u8) -> bool {
        // verificar que sea una calificación válida
        if !(1..=5).contains(&calificacion) {
            return false;
        }

        // verificar que exista data comprador
        let Some(mut nuevo_data_comprador) = self.obtener_data_comprador()
        else { return false; };

        // safe sum
        let Some(nuevo_total_calificaciones) = nuevo_data_comprador.total_calificaciones.checked_add(u64::from(calificacion)) // safe cast
        else { return false; };

        // safe sum
        let Some(nuevo_cant_calificaciones) = nuevo_data_comprador.cant_calificaciones.checked_add(1)
        else { return false; };

        // asignar total de calificaciones
        nuevo_data_comprador.total_calificaciones = nuevo_total_calificaciones;
        nuevo_data_comprador.cant_calificaciones = nuevo_cant_calificaciones;
        
        // crear nuevo rol con la nueva informacion
        let nuevo_rol = match &self.rol {
            Rol::Comprador(_) => Rol::Comprador(nuevo_data_comprador),
            Rol::Vendedor(_) => return false, // no debería nunca poder pasar
            Rol::Ambos(_, data_vendedor) => Rol::Ambos(nuevo_data_comprador, data_vendedor.clone()),
        };

        // modificar y finalizar
        self.rol = nuevo_rol;
        true
    }

    /// Calificar al usuario como vendedor
    /// 
    /// Devolverá true si la operación fue exitosa o false en caso contrario.
    pub fn calificar_como_vendedor(&mut self, calificacion: u8) -> bool {
        // verificar que sea una calificación válida
        if !(1..=5).contains(&calificacion) {
            return false;
        }

        // verificar que exista data comprador
        let Some(mut nuevo_data_vendedor) = self.obtener_data_vendedor()
        else { return false; };

        // safe sum
        let Some(nuevo_total_calificaciones) = nuevo_data_vendedor.total_calificaciones.checked_add(u64::from(calificacion)) // safe cast
        else { return false; };

        // safe sum
        let Some(nuevo_cant_calificaciones) = nuevo_data_vendedor.cant_calificaciones.checked_add(1)
        else { return false; };

        // asignar total de calificaciones
        nuevo_data_vendedor.total_calificaciones = nuevo_total_calificaciones;
        nuevo_data_vendedor.cant_calificaciones = nuevo_cant_calificaciones;
        
        // crear nuevo rol con la nueva informacion
        let nuevo_rol = match &self.rol {
            Rol::Comprador(_) => return false,
            Rol::Vendedor(_) => Rol::Vendedor(nuevo_data_vendedor), // no debería nunca poder pasar
            Rol::Ambos(data_comprador, _) => Rol::Ambos(data_comprador.clone(), nuevo_data_vendedor),
        };

        // modificar y finalizar
        self.rol = nuevo_rol;
        true
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
pub enum ErrorAscenderRolUsuario {
    UsuarioInexistente,
    MaximoRolAsignado,
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

    /// Si el usuario es Vendedor o Comprador, cambia su rol a Ambos sin perder información
    /// 
    /// Devuelve error si el usuario no existe o ya posee el rol Ambos.
    pub fn _ascender_rol_usuario(&mut self, caller: AccountId) -> Result<(), ErrorAscenderRolUsuario> {
        // si no existe, es imposible modificar
        let Some(usuario) = self.usuarios.get(caller)
        else {
            return Err(ErrorAscenderRolUsuario::UsuarioInexistente);
        };

        let nuevo_rol = match &usuario.rol {
            Rol::Comprador(compras) => Rol::Ambos(compras.clone(), Default::default()),
            Rol::Vendedor(data_vendedor) => Rol::Ambos(Default::default(), data_vendedor.clone()),
            _ => return Err(ErrorAscenderRolUsuario::MaximoRolAsignado),
        };

        let mut usuario = usuario;
        usuario.rol = nuevo_rol;
        self.usuarios.insert(caller, &usuario);

        Ok(())
    }

    /// Ver la calificación histórica promedio del usuario como comprador.
    /// 
    /// Devolverá None si no es comprador o no tiene calificaciones.
    pub fn _ver_calificacion_comprador(&self, caller: AccountId) -> Option<u8> {
        let Some(usuario) = self.usuarios.get(caller)
        else { return None; };

        // obtener información de calificaciones
        let (total_calificaciones, cant_calificaciones) = match usuario.rol {
            Rol::Comprador(data_comprador) => (data_comprador.total_calificaciones, data_comprador.cant_calificaciones),
            Rol::Vendedor(_) => return None,
            Rol::Ambos(data_comprador, _) => (data_comprador.total_calificaciones, data_comprador.cant_calificaciones),
        };

        // devuelve None si cant_calificaciones == 0
        let Some(total_calificaciones) = total_calificaciones.checked_div_euclid(u64::from(cant_calificaciones))
        else { return None; };

        // si las cantidades se manejan bien, que así debería ser por las funciones de calificar_como_*,
        // esta operación no debería jamás dar como resultado un número mayor a 5.
        // por lo tanto, el cast u64 -> u8 deberia ser seguro.

        total_calificaciones.to_be_bytes().first().copied()
    }

    /// Ver la calificación histórica promedio del usuario como vendedor.
    /// 
    /// Devolverá None si no es vendedor o no tiene calificaciones.
    pub fn _ver_calificacion_vendedor(&self, caller: AccountId) -> Option<u8> {
        let Some(usuario) = self.usuarios.get(caller)
        else { return None; };

        // obtener información de calificaciones
        let (total_calificaciones, cant_calificaciones) = match usuario.rol {
            Rol::Comprador(_) => return None,
            Rol::Vendedor(data_vendedor) => (data_vendedor.total_calificaciones, data_vendedor.cant_calificaciones),
            Rol::Ambos(_, data_vendedor) => (data_vendedor.total_calificaciones, data_vendedor.cant_calificaciones),
        };

        // devuelve None si cant_calificaciones == 0
        let Some(total_calificaciones) = total_calificaciones.checked_div_euclid(u64::from(cant_calificaciones))
        else { return None; };

        // si las cantidades se manejan bien, que así debería ser por las funciones de calificar_como_*,
        // esta operación no debería jamás dar como resultado un número mayor a 5.
        // por lo tanto, el cast u64 -> u8 deberia ser seguro.

        total_calificaciones.to_be_bytes().first().copied()
    }
}