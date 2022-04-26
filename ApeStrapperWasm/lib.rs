#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod ApeStrapperWasm {

    use ink_storage::traits::SpreadAllocate;
    use ink_storage::Mapping;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct ApeStrapperWasm {
        // CHAOSDAO_MULTISIG_ADDRESS: AccountId,
        // NODE_OPERATOR_ADDRESS: AccountId,
        // contract_creator: AccountId,
        ape_allocation: Mapping<AccountId, u32>,
        ape_approved: Mapping<AccountId, bool>,
        apes: Vec<AccountId>,
    }

    impl ApeStrapperWasm {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(apes: Vec<AccountId>) -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                for ape in apes {
                    contract.apes.push(ape);
                }
            })
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        // #[ink(constructor)]
        // pub fn default() -> Self {
        //     Self::new(Default::default())
        // }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn flip(&mut self) {}

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get_apes(&self) -> Vec<AccountId> {
            self.apes.clone()
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let ApeStrapperWasm = ApeStrapperWasm::default();
            assert_eq!(ApeStrapperWasm.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut ApeStrapperWasm = ApeStrapperWasm::new(false);
            assert_eq!(ApeStrapperWasm.get(), false);
            ApeStrapperWasm.flip();
            assert_eq!(ApeStrapperWasm.get(), true);
        }
    }
}
