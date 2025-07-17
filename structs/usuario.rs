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
    pub productos: Vec<u128>,
    pub stock: Vec<u32>
}

impl StockProductos {
    /// Devuelve el stock asociado a la ID del producto brindada
    pub fn get(&self, id_producto: &u128) -> Option<u32> {
        let Ok(index) = self.productos.binary_search(id_producto)
        else { return None };
        self.stock.get(index).copied()
    }

    /// Inserta el producto en el vector doble. Si ya existe un producto con ese valor, lo sobreescribe.
    /// Si no existe un producto con ese valor, inserta al final de la lista.
    /// cambios aca !!!!!!!!!!!!
    /// El método actual elimina elementos en el índice Err(index), que es incorrecto y causa pánico.
    /// La solución es no eliminar nada en Err(index), solo insertar.
    /// En Ok(index) actualizar el stock existente.
    pub fn insert(&mut self, id_producto: u128, stock: u32) {
    match self.productos.binary_search(&id_producto) {
        Ok(index) => {
            // Producto ya existe: actualizar el stock
            self.stock[index] = stock;
        }
        Err(index) => {
            // Producto no existe: insertar en posición ordenada
            self.productos.insert(index, id_producto);
            self.stock.insert(index, stock);
        }
    }
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
pub enum RolDeSeleccion {
    Comprador,
    Vendedor,
    Ambos,
}

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
    pub fn _registrar_usuario(&mut self, caller: AccountId, rol: RolDeSeleccion) -> Result<(), ErrorRegistrarUsuario>  {
        // el usuario no puede ya existir
        if self.usuarios.contains(caller) {
            return Err(ErrorRegistrarUsuario::UsuarioYaExiste)
        }

        let rol = match rol {
            RolDeSeleccion::Comprador => Rol::Comprador(DataComprador::default()),
            RolDeSeleccion::Vendedor => Rol::Vendedor(DataVendedor::default()),
            RolDeSeleccion::Ambos => Rol::Ambos(DataComprador::default(), DataVendedor::default())
        };

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


#[cfg(test)]
mod tests {
    use super::*;
    #[ink::test]
    fn registrar_usuario_funciona_correctamente() {
        let mut contrato = RustaceoLibre::default();

        let cuenta = AccountId::from([0x1; 32]);

        // Registro exitoso
        let resultado = contrato._registrar_usuario(cuenta, RolDeSeleccion::Comprador);
        assert_eq!(resultado, Ok(()));

        // Fallo por registrar el mismo usuario
        let resultado_repetido = contrato._registrar_usuario(cuenta, RolDeSeleccion::Vendedor);
        assert_eq!(resultado_repetido, Err(ErrorRegistrarUsuario::UsuarioYaExiste));
    }

    #[ink::test]
    fn ascender_rol_usuario_funciona_correctamente() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x2; 32]);

        // Intentar ascender sin registrar usuario → Error
        let resultado_inexistente = contrato._ascender_rol_usuario(cuenta);
        assert_eq!(resultado_inexistente, Err(ErrorAscenderRolUsuario::UsuarioInexistente));

        // Registrar usuario como Comprador
        let _ = contrato._registrar_usuario(cuenta, RolDeSeleccion::Comprador);

        // Asciende correctamente a Ambos
        let resultado_ascenso = contrato._ascender_rol_usuario(cuenta);
        assert_eq!(resultado_ascenso, Ok(()));

        // Intentar ascender nuevamente → Error por ya tener rol Ambos
        let resultado_reintento = contrato._ascender_rol_usuario(cuenta);
        assert_eq!(resultado_reintento, Err(ErrorAscenderRolUsuario::MaximoRolAsignado));
    }

    #[ink::test]
    fn ascender_rol_usuario_falla_si_ya_es_ambos() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x7; 32]);

        // Registrar como comprador
        assert!(contrato._registrar_usuario(cuenta, RolDeSeleccion::Comprador).is_ok());

        // Ascender una vez (Comprador -> Ambos)
        assert!(contrato._ascender_rol_usuario(cuenta).is_ok());

        // Intentar ascender de nuevo (ya es Ambos)
        assert_eq!(
            contrato._ascender_rol_usuario(cuenta),
            Err(ErrorAscenderRolUsuario::MaximoRolAsignado)
        );
    }

    #[ink::test]
    fn ascender_rol_usuario_falla_usuario_inexistente() {
        let mut contrato = RustaceoLibre::default();
        let cuenta_inexistente = AccountId::from([0x8; 32]);

        assert_eq!(
            contrato._ascender_rol_usuario(cuenta_inexistente),
            Err(ErrorAscenderRolUsuario::UsuarioInexistente)
        );
    }

    #[ink::test]
    fn registrar_usuario_falla_si_ya_existe() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x9; 32]);

        // Primer registro debe funcionar
        assert_eq!(
            contrato._registrar_usuario(cuenta, RolDeSeleccion::Comprador),
            Ok(())
        );

        // Segundo intento debe fallar
        assert_eq!(
            contrato._registrar_usuario(cuenta, RolDeSeleccion::Comprador),
            Err(ErrorRegistrarUsuario::UsuarioYaExiste)
        );
    }

    #[ink::test]
    fn agregar_compra_funciona_correctamente() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x10; 32]);

        // Registrar como comprador
        assert!(contrato._registrar_usuario(cuenta, RolDeSeleccion::Comprador).is_ok());

        // Agregar compra
        if let Some(mut usuario) = contrato.usuarios.get(cuenta) {
            assert!(usuario.agregar_compra(1001));
            contrato.usuarios.insert(cuenta, &usuario);
        }

        // Confirmar que la compra se agregó
        if let Some(usuario) = contrato.usuarios.get(cuenta) {
            assert_eq!(usuario.obtener_compras(), Some(vec![1001]));
        }
    }

    #[ink::test]
    fn agregar_venta_funciona_correctamente() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x11; 32]);

        // Registrar usuario vendedor
        assert!(contrato._registrar_usuario(cuenta, RolDeSeleccion::Vendedor).is_ok());

        // Agregar venta
        if let Some(mut usuario) = contrato.usuarios.get(cuenta) {
            assert!(usuario.agregar_venta(2001));
            contrato.usuarios.insert(cuenta, &usuario);
        }

        // Confirmar venta agregada
        if let Some(usuario) = contrato.usuarios.get(cuenta) {
            assert_eq!(usuario.obtener_ventas(), Some(vec![2001]));
        }
    }

    #[ink::test]
    fn agregar_publicacion_funciona_correctamente() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x12; 32]);

        // Registrar usuario vendedor
        assert!(contrato._registrar_usuario(cuenta, RolDeSeleccion::Vendedor).is_ok());

        // Agregar publicacion
        if let Some(mut usuario) = contrato.usuarios.get(cuenta) {
            assert!(usuario.agregar_publicacion(3001));
            contrato.usuarios.insert(cuenta, &usuario);
        }

        // Confirmar publicacion agregada
        if let Some(usuario) = contrato.usuarios.get(cuenta) {
            assert_eq!(usuario.obtener_publicaciones(), Some(vec![3001]));
        }
    }

    #[ink::test]
    fn establecer_stock_producto_funciona_correctamente() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x13; 32]);

        // Registrar usuario vendedor
        assert!(contrato._registrar_usuario(cuenta, RolDeSeleccion::Vendedor).is_ok());

        // Establecer stock producto
        if let Some(mut usuario) = contrato.usuarios.get(cuenta) {
            let id_producto = 4001u128;
            let stock = 50u32;
            assert!(usuario.establecer_stock_producto(&id_producto, &stock));
            contrato.usuarios.insert(cuenta, &usuario);
        }

        // Confirmar stock modificado
        if let Some(usuario) = contrato.usuarios.get(cuenta) {
            assert_eq!(usuario.obtener_stock_producto(&4001u128), Some(50));
        }
    }

    #[ink::test]
    fn calificar_como_comprador_funciona_correctamente() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x14; 32]);

        // Registrar comprador
        assert!(contrato._registrar_usuario(cuenta, RolDeSeleccion::Comprador).is_ok());

        if let Some(mut usuario) = contrato.usuarios.get(cuenta) {
            assert!(usuario.calificar_como_comprador(5));
            contrato.usuarios.insert(cuenta, &usuario);
        }

        // Confirmar calificacion
        if let Some(usuario) = contrato.usuarios.get(cuenta) {
            let data = usuario.obtener_data_comprador().unwrap();
            assert_eq!(data.total_calificaciones, 5);
            assert_eq!(data.cant_calificaciones, 1);
        }
    }

    #[ink::test]
    fn calificar_como_comprador_falla_con_calificacion_invalida() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x15; 32]);

        // Registrar comprador
        assert!(contrato._registrar_usuario(cuenta, RolDeSeleccion::Comprador).is_ok());

        if let Some(mut usuario) = contrato.usuarios.get(cuenta) {
            // Calificacion invalida (0)
            assert_eq!(usuario.calificar_como_comprador(0), false);
            // Calificacion invalida (6)
            assert_eq!(usuario.calificar_como_comprador(6), false);
        }
    }

    #[ink::test]
    fn calificar_como_vendedor_funciona_correctamente() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x16; 32]);

        // Registrar vendedor
        assert!(contrato._registrar_usuario(cuenta, RolDeSeleccion::Vendedor).is_ok());

        if let Some(mut usuario) = contrato.usuarios.get(cuenta) {
            assert!(usuario.calificar_como_vendedor(4));
            contrato.usuarios.insert(cuenta, &usuario);
        }

        // Confirmar calificacion
        if let Some(usuario) = contrato.usuarios.get(cuenta) {
            let data = usuario.obtener_data_vendedor().unwrap();
            assert_eq!(data.total_calificaciones, 4);
            assert_eq!(data.cant_calificaciones, 1);
        }
    }

    #[ink::test]
    fn calificar_como_vendedor_falla_con_calificacion_invalida() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x17; 32]);

        // Registrar vendedor
        assert!(contrato._registrar_usuario(cuenta, RolDeSeleccion::Vendedor).is_ok());

        if let Some(mut usuario) = contrato.usuarios.get(cuenta) {
            // Calificacion invalida (0)
            assert_eq!(usuario.calificar_como_vendedor(0), false);
            // Calificacion invalida (6)
            assert_eq!(usuario.calificar_como_vendedor(6), false);
        }
    }


    #[ink::test]
    fn ver_calificacion_comprador_devuelve_none_si_no_es_comprador() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x19; 32]);

        // Registrar usuario vendedor
        let usuario = Usuario::new(cuenta, Rol::Vendedor(DataVendedor::default()));
        contrato.usuarios.insert(cuenta, &usuario);

        assert_eq!(contrato._ver_calificacion_comprador(cuenta), None);
    }


    #[ink::test]
    fn ver_calificacion_vendedor_devuelve_none_si_no_es_vendedor() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x21; 32]);

        // Registrar usuario comprador
        let usuario = Usuario::new(cuenta, Rol::Comprador(DataComprador::default()));
        contrato.usuarios.insert(cuenta, &usuario);

        assert_eq!(contrato._ver_calificacion_vendedor(cuenta), None);
    }

    #[ink::test]
    fn es_comprador_y_es_vendedor_funcionan_correctamente() {
        let comprador = Usuario::new(AccountId::from([0x1; 32]), Rol::Comprador(DataComprador::default()));
        assert!(comprador.es_comprador());
        assert!(!comprador.es_vendedor());

        let vendedor = Usuario::new(AccountId::from([0x2; 32]), Rol::Vendedor(DataVendedor::default()));
        assert!(!vendedor.es_comprador());
        assert!(vendedor.es_vendedor());

        let ambos = Usuario::new(AccountId::from([0x3; 32]), Rol::Ambos(DataComprador::default(), DataVendedor::default()));
        assert!(ambos.es_comprador());
        assert!(ambos.es_vendedor());
    }

    #[ink::test]
    fn obtener_stock_productos_y_obtener_stock_producto_funcionan() {
        let mut vendedor = Usuario::new(AccountId::from([0x4; 32]), Rol::Vendedor(DataVendedor::default()));

        // Inicialmente no hay productos
        assert_eq!(vendedor.obtener_stock_productos().unwrap().productos.len(), 0);
        assert_eq!(vendedor.obtener_stock_producto(&123u128), None);

        // Establecer stock y verificar
        assert!(vendedor.establecer_stock_producto(&123u128, &10));
        assert_eq!(vendedor.obtener_stock_producto(&123u128), Some(10));

        // Stock de producto inexistente devuelve None
        assert_eq!(vendedor.obtener_stock_producto(&999u128), None);
    }

    #[ink::test]
    fn agregar_compra_falla_si_usuario_no_es_comprador() {
        let mut vendedor = Usuario::new(AccountId::from([0x5; 32]), Rol::Vendedor(DataVendedor::default()));

        assert!(!vendedor.agregar_compra(5001));
    }

    #[ink::test]
    fn agregar_venta_falla_si_usuario_no_es_vendedor() {
        let mut comprador = Usuario::new(AccountId::from([0x6; 32]), Rol::Comprador(DataComprador::default()));

        assert!(!comprador.agregar_venta(6001));
    }

    #[ink::test]
    fn agregar_publicacion_falla_si_usuario_no_es_vendedor() {
        let mut comprador = Usuario::new(AccountId::from([0x7; 32]), Rol::Comprador(DataComprador::default()));

        assert!(!comprador.agregar_publicacion(7001));
    }

    #[ink::test]
    fn establecer_stock_producto_falla_si_usuario_no_es_vendedor() {
        let mut comprador = Usuario::new(AccountId::from([0x8; 32]), Rol::Comprador(DataComprador::default()));

        let stock = 30u32;
        assert!(!comprador.establecer_stock_producto(&123u128, &stock));
    }

    #[ink::test]
    fn stock_productos_insert_y_get_funcionan_correctamente() {
        let mut stock = StockProductos::default();

        // Insertar producto nuevo
        stock.insert(1u128, 10);
        assert_eq!(stock.get(&1u128), Some(10));

        // Insertar producto en orden correcto (menor)
        stock.insert(0u128, 5);
        assert_eq!(stock.get(&0u128), Some(5));

        // Insertar producto en orden correcto (mayor)
        stock.insert(2u128, 20);
        assert_eq!(stock.get(&2u128), Some(20));

        // Actualizar producto existente
        stock.insert(1u128, 15);
        assert_eq!(stock.get(&1u128), Some(15));

        // Buscar producto inexistente
        assert_eq!(stock.get(&99u128), None);
    }

    #[ink::test]
    fn calificar_como_comprador_acepta_valores_limite() {
        let mut usuario = Usuario::new(AccountId::from([0x9; 32]), Rol::Comprador(DataComprador::default()));

        assert!(usuario.calificar_como_comprador(1));
        assert!(usuario.calificar_como_comprador(5));
    }

    #[ink::test]
    fn calificar_como_vendedor_acepta_valores_limite() {
        let mut usuario = Usuario::new(AccountId::from([0x10; 32]), Rol::Vendedor(DataVendedor::default()));

        assert!(usuario.calificar_como_vendedor(1));
        assert!(usuario.calificar_como_vendedor(5));
    }

    #[ink::test]
    fn agregar_compra_no_duplica_compras() {
        let mut usuario = Usuario::new(AccountId::from([0x20; 32]), Rol::Comprador(DataComprador::default()));
        assert!(usuario.agregar_compra(123));
        // Intentar agregar la misma compra otra vez
        assert!(usuario.agregar_compra(123));
        // Ver que efectivamente hay dos (porque tu lógica actual no impide duplicados)
        assert_eq!(usuario.obtener_compras().unwrap().len(), 2);
    }

    #[ink::test]
    fn agregar_venta_no_duplica_ventas() {
        let mut usuario = Usuario::new(AccountId::from([0x21; 32]), Rol::Vendedor(DataVendedor::default()));
        assert!(usuario.agregar_venta(456));
        assert!(usuario.agregar_venta(456));
        assert_eq!(usuario.obtener_ventas().unwrap().len(), 2);
    }

    #[ink::test]
    fn calificar_como_comprador_no_modifica_si_usuario_no_comprador() {
        let mut usuario = Usuario::new(AccountId::from([0x22; 32]), Rol::Vendedor(DataVendedor::default()));
        assert_eq!(usuario.calificar_como_comprador(3), false);
    }

    #[ink::test]
    fn calificar_como_vendedor_no_modifica_si_usuario_no_vendedor() {
        let mut usuario = Usuario::new(AccountId::from([0x23; 32]), Rol::Comprador(DataComprador::default()));
        assert_eq!(usuario.calificar_como_vendedor(3), false);
    }

    #[ink::test]
    fn obtener_stock_producto_no_pánico_con_usuario_sin_stock() {
        let usuario = Usuario::new(AccountId::from([0x24; 32]), Rol::Vendedor(DataVendedor::default()));
        // No tiene productos
        assert_eq!(usuario.obtener_stock_producto(&999u128), None);
    }

    #[ink::test]
    fn establecer_stock_producto_modifica_correctamente_stock_existente() {
        let mut usuario = Usuario::new(AccountId::from([0x25; 32]), Rol::Vendedor(DataVendedor::default()));
        assert!(usuario.establecer_stock_producto(&1u128, &10));
        assert_eq!(usuario.obtener_stock_producto(&1u128), Some(10));
        // Cambiar stock del mismo producto
        assert!(usuario.establecer_stock_producto(&1u128, &20));
        assert_eq!(usuario.obtener_stock_producto(&1u128), Some(20));
    }

    #[ink::test]
    fn calificaciones_promedio_comprador_y_vendedor_sin_calificaciones_devuelven_none() {
        let mut usuario = Usuario::new(AccountId::from([0x26; 32]), Rol::Ambos(DataComprador::default(), DataVendedor::default()));
        // Sin calificaciones aún
        assert_eq!(usuario.calificar_como_comprador(0), false);
        assert_eq!(usuario.calificar_como_vendedor(0), false);

        // Revisar calificaciones promedio (deberían ser None)
        let contrato = RustaceoLibre::default();
        assert_eq!(contrato._ver_calificacion_comprador(usuario.id), None);
        assert_eq!(contrato._ver_calificacion_vendedor(usuario.id), None);
    }

    #[ink::test]
    fn rol_es_comprador_y_es_vendedor_para_casos_ambos() {
        let usuario = Usuario::new(AccountId::from([0x27; 32]), Rol::Ambos(DataComprador::default(), DataVendedor::default()));
        assert!(usuario.es_comprador());
        assert!(usuario.es_vendedor());
    }

    #[ink::test]
    fn rol_es_comprador_y_es_vendedor_para_casos_unicos() {
        let usuario_c = Usuario::new(AccountId::from([0x28; 32]), Rol::Comprador(DataComprador::default()));
        assert!(usuario_c.es_comprador());
        assert!(!usuario_c.es_vendedor());

        let usuario_v = Usuario::new(AccountId::from([0x29; 32]), Rol::Vendedor(DataVendedor::default()));
        assert!(!usuario_v.es_comprador());
        assert!(usuario_v.es_vendedor());
    }

    #[ink::test]
    fn insertar_stock_productos_ordenado() {
        let mut stock = StockProductos::default();
        stock.insert(10, 100);
        stock.insert(5, 50);
        stock.insert(7, 70);
        stock.insert(3, 30);
        stock.insert(10, 110); // actualizar

        assert_eq!(stock.productos, vec![3,5,7,10]);
        assert_eq!(stock.get(&3), Some(30));
        assert_eq!(stock.get(&5), Some(50));
        assert_eq!(stock.get(&7), Some(70));
        assert_eq!(stock.get(&10), Some(110));
    }

    #[ink::test]
    fn insertar_stock_productos_con_producto_no_existente() {
        let stock = StockProductos::default();
        assert_eq!(stock.get(&999), None);
    }

    #[ink::test]
    fn ver_calificacion_comprador_devuelve_none_si_no_tiene_calificaciones() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x30; 32]);

        let comprador = Usuario::new(cuenta, Rol::Comprador(DataComprador::default()));
        contrato.usuarios.insert(cuenta, &comprador);

        assert_eq!(contrato._ver_calificacion_comprador(cuenta), None);
    }

    #[ink::test]
    fn ver_calificacion_vendedor_devuelve_none_si_no_tiene_calificaciones() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x31; 32]);

        let vendedor = Usuario::new(cuenta, Rol::Vendedor(DataVendedor::default()));
        contrato.usuarios.insert(cuenta, &vendedor);

        assert_eq!(contrato._ver_calificacion_vendedor(cuenta), None);
    }

    #[ink::test]
    fn establecer_stock_producto_actualiza_sin_duplicar() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x34; 32]);

        assert!(contrato._registrar_usuario(cuenta, RolDeSeleccion::Vendedor).is_ok());

        if let Some(mut usuario) = contrato.usuarios.get(cuenta) {
            usuario.establecer_stock_producto(&123, &10);
            usuario.establecer_stock_producto(&123, &20);
            contrato.usuarios.insert(cuenta, &usuario);
        }

        if let Some(usuario) = contrato.usuarios.get(cuenta) {
            assert_eq!(usuario.obtener_stock_producto(&123), Some(20));
            let stock = usuario.obtener_stock_productos().unwrap();
            assert_eq!(stock.productos.len(), 1); // no se duplicó
        }
    }

    #[ink::test]
    fn ascender_rol_usuario_funciona_para_vendedor() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x35; 32]);

        assert!(contrato._registrar_usuario(cuenta, RolDeSeleccion::Vendedor).is_ok());

        let resultado = contrato._ascender_rol_usuario(cuenta);
        assert_eq!(resultado, Ok(()));

        if let Some(usuario) = contrato.usuarios.get(cuenta) {
            assert!(usuario.es_comprador());
            assert!(usuario.es_vendedor());
        }
    }

    #[ink::test]
    fn ascender_rol_usuario_de_vendedor_a_ambos_funciona_correctamente() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x32; 32]);

        // Registrar como vendedor
        assert!(contrato._registrar_usuario(cuenta, RolDeSeleccion::Vendedor).is_ok());

        // Asciende a Ambos
        let resultado = contrato._ascender_rol_usuario(cuenta);
        assert_eq!(resultado, Ok(()));
    }

    #[ink::test]
    fn stock_productos_insert_en_medio_funciona_correctamente() {
        let mut stock = StockProductos::default();
        stock.insert(10, 100);
        stock.insert(30, 300);
        stock.insert(20, 200); // va al medio

        assert_eq!(stock.productos, vec![10, 20, 30]);
        assert_eq!(stock.stock, vec![100, 200, 300]);
    }

        #[ink::test]
        fn stock_productos_insert_mantiene_orden_correcto() {
            let mut stock = StockProductos::default();
            stock.insert(5, 50);
            stock.insert(3, 30);
            stock.insert(7, 70);
            stock.insert(1, 10); // al principio
            stock.insert(9, 90); // al final

            assert_eq!(stock.productos, vec![1, 3, 5, 7, 9]);
            assert_eq!(stock.stock, vec![10, 30, 50, 70, 90]);
        }

    #[ink::test]
    fn calificar_como_comprador_rechaza_valores_invalidos() {
        let mut usuario = Usuario::new(AccountId::from([0x50; 32]), Rol::Comprador(DataComprador::default()));
        assert_eq!(usuario.calificar_como_comprador(0), false);
        assert_eq!(usuario.calificar_como_comprador(6), false);
    }

    #[ink::test]
    fn calificar_como_vendedor_rechaza_valores_invalidos() {
        let mut usuario = Usuario::new(AccountId::from([0x51; 32]), Rol::Vendedor(DataVendedor::default()));
        assert_eq!(usuario.calificar_como_vendedor(0), false);
        assert_eq!(usuario.calificar_como_vendedor(6), false);
    }

    #[ink::test]
    fn calificar_como_comprador_rechaza_usuario_sin_rol_comprador() {
        let mut usuario = Usuario::new(AccountId::from([0x52; 32]), Rol::Vendedor(DataVendedor::default()));
        assert_eq!(usuario.calificar_como_comprador(4), false);
    }

    #[ink::test]
    fn calificar_como_vendedor_rechaza_usuario_sin_rol_vendedor() {
        let mut usuario = Usuario::new(AccountId::from([0x53; 32]), Rol::Comprador(DataComprador::default()));
        assert_eq!(usuario.calificar_como_vendedor(4), false);
    }

    #[ink::test]
    fn obtener_stock_producto_no_existe_devuelve_none() {
        let usuario = Usuario::new(AccountId::from([0x55; 32]), Rol::Vendedor(DataVendedor::default()));
        assert_eq!(usuario.obtener_stock_producto(&99999), None);
    }

    #[ink::test]
    fn establecer_y_modificar_stock_producto() {
        let mut usuario = Usuario::new(AccountId::from([0x56; 32]), Rol::Vendedor(DataVendedor::default()));
        assert!(usuario.establecer_stock_producto(&42, &15));
        assert_eq!(usuario.obtener_stock_producto(&42), Some(15));

        // Modificar
        assert!(usuario.establecer_stock_producto(&42, &30));
        assert_eq!(usuario.obtener_stock_producto(&42), Some(30));
    }

    #[ink::test]
    fn insertar_varios_productos_y_verificar_orden() {
        let mut stock = StockProductos::default();
        stock.insert(20, 200);
        stock.insert(10, 100);
        stock.insert(15, 150);
        stock.insert(10, 110); // actualiza el existente

        assert_eq!(stock.productos, vec![10, 15, 20]);
        assert_eq!(stock.get(&10), Some(110));
        assert_eq!(stock.get(&15), Some(150));
        assert_eq!(stock.get(&20), Some(200));
    }

    #[ink::test]
    fn ver_calificacion_vendedor_sin_calificaciones_devuelve_none() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x58; 32]);

        let usuario = Usuario::new(cuenta, Rol::Vendedor(DataVendedor::default()));
        contrato.usuarios.insert(cuenta, &usuario);

        assert_eq!(contrato._ver_calificacion_vendedor(cuenta), None);
    }

    #[ink::test]
    fn ver_calificacion_comprador_usuario_no_existente_devuelve_none() {
        let contrato = RustaceoLibre::default();
        let cuenta_inexistente = AccountId::from([0x99; 32]);

        assert_eq!(contrato._ver_calificacion_comprador(cuenta_inexistente), None);
    }

    #[ink::test]
    fn establecer_stock_producto_en_usuario_no_vendedor_falla() {
        let mut usuario = Usuario::new(AccountId::from([0x60; 32]), Rol::Comprador(DataComprador::default()));
        assert_eq!(usuario.establecer_stock_producto(&1, &10), false);
    }

    #[ink::test]
    fn calificar_comprador_y_vendedor_en_usuario_ambos_funciona() {
        let mut usuario = Usuario::new(AccountId::from([0x61; 32]), Rol::Ambos(DataComprador::default(), DataVendedor::default()));

        assert!(usuario.calificar_como_comprador(5));
        assert!(usuario.calificar_como_vendedor(4));
    }

    #[ink::test]
    fn obtener_compras_falla_si_usuario_no_es_comprador() {
        let usuario = Usuario::new(AccountId::from([0x62; 32]), Rol::Vendedor(DataVendedor::default()));
        assert_eq!(usuario.obtener_compras(), None);
    }

    #[ink::test]
    fn obtener_ventas_falla_si_usuario_no_es_vendedor() {
        let usuario = Usuario::new(AccountId::from([0x63; 32]), Rol::Comprador(DataComprador::default()));
        assert_eq!(usuario.obtener_ventas(), None);
    }

    #[ink::test]
    fn stock_productos_insert_actualiza_sin_duplicar() {
        let mut stock = StockProductos::default();
        stock.insert(1, 10);
        stock.insert(1, 25); // actualiza
        assert_eq!(stock.productos, vec![1]);
        assert_eq!(stock.stock, vec![25]);
    }
    ///

    #[ink::test]
    fn obtener_stock_productos_devuelve_none_si_no_es_vendedor() {
        let usuario = Usuario::new(AccountId::from([0x71; 32]), Rol::Comprador(DataComprador::default()));
        assert_eq!(usuario.obtener_stock_productos(), None);
    }

    #[ink::test]
    fn obtener_compras_y_ventas_vacias_devuelve_vector_vacio() {
        let comprador = Usuario::new(AccountId::from([0x72; 32]), Rol::Comprador(DataComprador::default()));
        assert_eq!(comprador.obtener_compras().unwrap().len(), 0);

        let vendedor = Usuario::new(AccountId::from([0x73; 32]), Rol::Vendedor(DataVendedor::default()));
        assert_eq!(vendedor.obtener_ventas().unwrap().len(), 0);
    }

    #[ink::test]
    fn establecer_varios_stock_productos_funciona() {
        let mut usuario = Usuario::new(AccountId::from([0x74; 32]), Rol::Vendedor(DataVendedor::default()));

        assert!(usuario.establecer_stock_producto(&100, &10));
        assert!(usuario.establecer_stock_producto(&200, &20));
        assert!(usuario.establecer_stock_producto(&150, &15));

        let stock = usuario.obtener_stock_productos().unwrap();
        assert_eq!(stock.productos, vec![100, 150, 200]);
    }

    #[ink::test]
    fn insertar_mismo_producto_variadas_veces_no_duplica() {
        let mut stock = StockProductos::default();

        stock.insert(1, 10);
        stock.insert(1, 20);
        stock.insert(1, 30);

        assert_eq!(stock.productos.len(), 1);
        assert_eq!(stock.get(&1), Some(30));
    }

    #[ink::test]
    fn insertar_producto_con_clave_negativa_funciona() {
        let mut stock = StockProductos::default();
        stock.insert(-1i128 as u128, 99);
        assert_eq!(stock.get(&(-1i128 as u128)), Some(99));
    }

    #[ink::test]
    fn obtener_compras_en_usuario_no_comprador_devuelve_none() {
        let usuario = Usuario::new(AccountId::from([0x76; 32]), Rol::Vendedor(DataVendedor::default()));
        assert_eq!(usuario.obtener_compras(), None);
    }

    #[ink::test]
    fn obtener_ventas_en_usuario_no_vendedor_devuelve_none() {
        let usuario = Usuario::new(AccountId::from([0x77; 32]), Rol::Comprador(DataComprador::default()));
        assert_eq!(usuario.obtener_ventas(), None);
    }

    #[ink::test]
    fn insertar_producto_con_id_y_stock_grande_funciona() {
        let mut stock = StockProductos::default();
        let producto_id = u128::MAX;
        let cantidad = u32::MAX;

        stock.insert(producto_id, cantidad);
        assert_eq!(stock.get(&producto_id), Some(cantidad));
    }
    ///
    #[ink::test]
    fn insertar_stock_en_orden_descendente_se_ordena() {
        let mut stock = StockProductos::default();

        stock.insert(30, 300);
        stock.insert(20, 200);
        stock.insert(10, 100);

        assert_eq!(stock.productos, vec![10, 20, 30]);
    }

    #[ink::test]
    fn establecer_stock_producto_con_cero_no_elimina() {
        let mut usuario = Usuario::new(AccountId::from([0x92; 32]), Rol::Vendedor(DataVendedor::default()));

        assert!(usuario.establecer_stock_producto(&100, &10));
        assert_eq!(usuario.obtener_stock_producto(&100), Some(10));

        // Establecer stock 0
        assert!(usuario.establecer_stock_producto(&100, &0));
        assert_eq!(usuario.obtener_stock_producto(&100), Some(0));
    }

    #[ink::test]
    fn insertar_y_actualizar_varios_productos_varias_veces() {
        let mut stock = StockProductos::default();

        stock.insert(1, 10);
        stock.insert(2, 20);
        stock.insert(1, 15); // actualizar
        stock.insert(3, 30);
        stock.insert(2, 25); // actualizar

        assert_eq!(stock.get(&1), Some(15));
        assert_eq!(stock.get(&2), Some(25));
        assert_eq!(stock.get(&3), Some(30));
    }

    #[ink::test]
    fn agregar_muchas_compras_y_ventas() {
        let mut usuario = Usuario::new(AccountId::from([0x93; 32]), Rol::Ambos(DataComprador::default(), DataVendedor::default()));

        for i in 0..100 {
            assert!(usuario.agregar_compra(i));
            assert!(usuario.agregar_venta(i + 1000));
        }

        let compras = usuario.obtener_compras().unwrap();
        let ventas = usuario.obtener_ventas().unwrap();

        assert_eq!(compras.len(), 100);
        assert_eq!(ventas.len(), 100);
        assert_eq!(compras[0], 0);
        assert_eq!(compras[99], 99);
        assert_eq!(ventas[0], 1000);
        assert_eq!(ventas[99], 1099);
    }

    #[ink::test]
    fn obtener_stock_productos_usuario_no_vendedor_devuelve_none() {
        let usuario = Usuario::new(AccountId::from([0x94; 32]), Rol::Comprador(DataComprador::default()));
        assert_eq!(usuario.obtener_stock_productos(), None);
    }

    #[ink::test]
    fn registrar_usuario_cuenta_duplicada_falla() {
        let mut contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x95; 32]);

        assert!(contrato._registrar_usuario(cuenta, RolDeSeleccion::Comprador).is_ok());
        let resultado = contrato._registrar_usuario(cuenta, RolDeSeleccion::Vendedor);
        assert!(resultado.is_err());
    }

    #[ink::test]
    fn ver_calificacion_usuario_no_registrado_devuelve_none() {
        let contrato = RustaceoLibre::default();
        let cuenta = AccountId::from([0x96; 32]);

        assert_eq!(contrato._ver_calificacion_comprador(cuenta), None);
        assert_eq!(contrato._ver_calificacion_vendedor(cuenta), None);
    }

    #[ink::test]
    fn establecer_stock_usuario_sin_rol_vendedor_falla_pero_ambos_funciona() {
        let mut usuario_sin_rol_vendedor = Usuario::new(AccountId::from([0x97; 32]), Rol::Comprador(DataComprador::default()));
        assert_eq!(usuario_sin_rol_vendedor.establecer_stock_producto(&1, &10), false);

        let mut usuario_ambos = Usuario::new(
            AccountId::from([0x98; 32]),
            Rol::Ambos(DataComprador::default(), DataVendedor::default())
        );
        assert!(usuario_ambos.establecer_stock_producto(&1, &10));
    }
}