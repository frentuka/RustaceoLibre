#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[allow(non_local_definitions)] // error molesto
#[ink::contract]
mod rustaceo_libre {
    use ink::storage::{Mapping, StorageVec};
    // structs propias

    //
    // RustaceoLibre: main struct
    //

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
    pub enum Rol {
        #[default]
        Cliente, Vendedor
    }

    /// Definición de la estructura del contrato
    #[ink(storage)]
    pub struct RustaceoLibre {
        pub compradores: Mapping<AccountId, Rol>,
        pub vendedores: StorageVec<AccountId>,
        pub publicaciones: StorageVec<u128>,
        pub compras: StorageVec<u128>,
        /// Lleva un recuento de la próxima ID disponible para los productos.
        pub publicaciones_siguiente_id: u128,
        /// Lleva un recuento de la próxima ID disponible para las compras.
        pub compras_siguiente_id: u128,
        /// ID del dueño del contrato
        pub owner: AccountId,
    }

    impl RustaceoLibre {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self::_new()
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::_new()
        }

        /// Crea una nueva instancia de RustaceoLibre
        fn _new() -> Self {
            Self {
                compradores: Default::default(),
                vendedores: Default::default(),
                publicaciones: Default::default(),
                compras: Default::default(),
                publicaciones_siguiente_id: 0,
                compras_siguiente_id: 0,
                owner: Self::env().caller(),
            }
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn next_id_compras(&mut self) -> Option<u128> {
            if self.owner != self.env().caller() {
                return None;
            }

            let id = self.compras_siguiente_id; // obtener actual
            self.compras_siguiente_id.checked_add(1)?; // sumarle 1 al actual para que apunte a un id desocupado
            Some(id) // devolver
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn next_id_publicaciones(&mut self) -> Option<u128> {
            if self.owner != self.env().caller() {
                return None;
            }

            let id = self.publicaciones_siguiente_id; // obtener actual
            self.publicaciones_siguiente_id.checked_add(1)?; // sumarle 1 al actual para que apunte a un id desocupado
            Some(id) // devolver
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let rustaceo_libre = RustaceoLibre::default();
            assert_eq!(rustaceo_libre.compras_siguiente_id, 0);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut rustaceo_libre = RustaceoLibre::new();
            assert_eq!(rustaceo_libre.next_id_compras(), Some(0));
            assert_eq!(rustaceo_libre.next_id_compras(), Some(1));
            assert_eq!(rustaceo_libre.next_id_publicaciones(), Some(0));
            assert_eq!(rustaceo_libre.next_id_publicaciones(), Some(1));
        }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::ContractsBackend;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut c: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = RustaceoLibreRef::default();

            // // When
            // let contract = c
            //     .instantiate("RustaceoLibre", &ink_e2e::alice(), &mut constructor)
            //     .submit()
            //     .await
            //     .expect("instantiate failed");
            // let mut call_builder = contract.call_builder::<RustaceoLibre>();
            //
            // // Then
            // let get = call_builder.next_id_compras();
            // let get_result = c.call(&ink_e2e::alice(), &get).dry_run().await?;
            // assert!(matches!(get_result.return_value(), Some(0)));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = RustaceoLibreRef::new();
            // let contract = client
            //     .instantiate("RustaceoLibre", &ink_e2e::bob(), &mut constructor)
            //     .submit()
            //     .await
            //     .expect("instantiate failed");
            // let mut call_builder = contract.call_builder::<RustaceoLibre>();
            //
            // let get = call_builder.next_id_compras();
            // let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            // assert!(matches!(get_result.return_value(), Some(0)));
            //
            // // When
            // let next_id_compras = call_builder.next_id_compras();
            // let _flip_result = client
            //     .call(&ink_e2e::bob(), &next_id_compras)
            //     .submit()
            //     .await
            //     .expect("flip failed");
            //
            // // Then
            // let get = call_builder.next_id_compras();
            // let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            // assert!(matches!(get_result.return_value(), Some(1)));

            Ok(())
        }
    }
}