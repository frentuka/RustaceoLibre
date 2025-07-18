<p align="center">
  <img src="https://github.com/frentuka/RustaceoLibre/blob/master/logo.png" />
</p>

# Rustaceo Libre: Estructura

Rustaceo Libre es un marketplace «al estilo Mercado Libre» desarrollado en Ink!

Despliegue del contrato: aaUgbgCYnjHr6MVU2rNNnrp37zLM5jyZmpkUeXr48Zrvccx

Esta es una descripción de la estructura del sistema, implementando todo lo solicitado por el enunciado y agregando o reimaginando algunas funcionalidades

## Usuario

Un usuario puede ser tanto vendedor como comprador o ambos a la vez. Puede registrarse como comprador o vendedor y posteriormente ascender su rol para ser ambos.

Las funcionalidades relacionadas a la compra y manejo de pedidos como comprador están reservadas para usuarios que se hayan registrado como compradores y lo mismo sucede para los vendedores: sólo ellos pueden registrar productos, listar publicaciones o administrar su stock.

A partir de ahora se referirá como "compradores" a los usuarios que se hayan registrado como comprador o ambos y como "vendedores" a quienes se hayan registrado como vendedor o ambos.

### Calificaciones

Tanto los compradores como los vendedores tienen una calificación brindada por sus contrapartes en sus pedidos realizados. Si un usuario es comprador y vendedor, tendrá dos calificaciones: una como comprador y otra como vendedor.

> Las clasificaciones de otros usuarios son públicas para todo aquel que esté registrado en el marketplace.

## Productos

Los vendedores serán capaces de hacer uso de la funcionalidad de registro de productos y hacer ingresos y retiros de stock de productos al Marketplace.

Todos los vendedores pueden utilizar todos los productos siempre que hayan ingresado stock del mismo, indiferentemente de quién haya registrado la información de ese producto en el Marketplace.

> La totalidad de los usuarios registrados pueden ver la información de un producto si conocen la ID del mismo.

## Publicación

Cuando un vendedor realiza una publicación, este debe brindar información como el producto, la cantidad a ofertar y el precio unitario.

Cuando el vendedor asigna cantidad a ofertar a una publicación, esta cantidad se descuenta del stock personal del vendedor. El vendedor puede, posterior a crear la publicación, administrar el stock de la misma asignando siempre desde su stock personal. No podrá asignar stock si no cuenta con la cantidad suficiente en su inventario personal para hacerlo ni podrá retirar stock de la publicación si esta no tiene la cantidad suficiente para hacerlo.

> Los compradores pueden acceder a la información de cualquier publicación si conocen su ID o al listado de publicaciones de cualquier vendedor si conocen su ID.

## Pedido

Los compradores pueden realizar pedidos a una publicación siempre en cuanto la misma tenga el stock suficiente para realizar el pedido y el comprador haya transferido los fondos suficientes para realizar la misma.

Si el comprador no transifirió los fondos suficientes de la publicación la operación no será exitosa y se le devolverán los fondos. Si transfirió de más, se le devolverá la cantidad correspondiente. Este Marketplace no cobra impuestos de ningún tipo.

> Los vendedores no reciben el dinero inmediatamente luego de la creación del pedido, el mismo queda almacenado en la cuenta del contrato hasta que el pedido se considere finalizado.

### Estados de un pedido

- **Pendiente:** Es el estado por defecto de cualquier pedido luego de su exitosa creación. Significa que el pedido está pendiente de despacharse por parte del vendedor y solo el mismo puede marcar el pedido como despachado.
- **Despachado:** El vendedor marcó el pedido como despachado y depende de que el usuario lo marque como recibido. El comprador tiene 60 dias para hacer esto. En caso contrario, el vendedor podría ejecutar la Cláusula de Reclamo de Fondos del pedido, _lo cual le enviará la totalidad de los fondos del pedido al vendedor_.
- **Recibido:** El comprador marcó este pedido como recibido y la operación se considera finalizada, ya no puede sufrir cambios y _la totalidad de los fondos involucrados se transfirieron al vendedor_.
- **Cancelado:** La cancelación de un pedido requiere de que ambas partes (comprador y vendedor) presten voluntad para hacerlo. No puede cancelarse un pedido ya cancelado o recibido. En caso de cancelarse un pedido, _todos los fondos involucrados se transfieren al comprador_ y la operación se considera finalizada.

### Calificación de un pedido

Una vez que un pedido se considera finalizado, ya sea por haber sido recibido o cancelado, ambas partes pueden calificarse mutuamente: el comprador al vendedor y viceversa.

> El usuario solo puede prestar una calificación por pedido.

# Setup infalible

En caso de que no pueda hacer un setup limpio del entorno, hemos encontrado la forma infalible de conseguir un entorno compatible.

Primero lo primero, deshacerse de Rust:

```bash
  rustup self uninstall
```

Seguido de volver a instalarlo desde la página oficial: https://rustup.rs

Instalar toolchain nightly-2024-06-20:

```bash
  rustup component add rust-src --toolchain nightly-2024-06-20
```

y Cargo Contract:

```bash
  cargo install cargo-contract --version 4.1.3
```

Si se desea utilizar Cargo Tarpaulin, la versión que recomendamos instalar es la 0.29.0, con el comando:

```bash
  cargo install cargo-tarpaulin --version 0.29.0 --locked
```

# Autores

- [@franIribarren](https://github.com/franIribarren) (Francisco Iribarren)
- [@ValentinoKvolek](https://github.com/ValentinoKvolek) (Valentino Kvolek)
- [@LucasZuazo](https://github.com/LucasZuazo) (Lucas Zuazo)
- [@frentuka](https://github.com/frentuka) (Fermín Franco)
