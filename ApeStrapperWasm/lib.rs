#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod ape_strapper_wasm {

    use ink_prelude::vec::*;
    use ink_storage::traits::SpreadAllocate;
    use ink_storage::Mapping;

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct ApeStrapperWasm {
        // CHAOSDAO_MULTISIG_ADDRESS: AccountId,
        // NODE_OPERATOR_ADDRESS: AccountId,
        // contract_creator: AccountId,
        ape_allocation: Mapping<AccountId, Balance>,
        ape_approved: Mapping<AccountId, bool>,
        apes: Vec<AccountId>,
    }

    impl ApeStrapperWasm {
        #[ink(constructor)]
        pub fn default() -> Self {
            let apes = ink_prelude::vec![];

            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.apes = apes;
            })
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get_apes(&self) -> Vec<AccountId> {
            self.apes.clone()
        }

        #[ink(message, payable)]
        pub fn get_allocations(&self) -> Vec<(AccountId, Balance)> {
            let mut allocations = ink_prelude::vec![];

            for (_, ape) in self.apes.iter().enumerate() {
                let ape = ape.clone();
                let allocation = self.ape_allocation.get(ape).unwrap_or(0);
                allocations.push((ape, allocation));
            }

            allocations
        }

        #[ink(message)]
        pub fn set_ape_allocation(&mut self, addresses: Vec<AccountId>, allocations: Vec<Balance>) {
            self.apes = addresses;
            let mut allocation_iter = allocations.iter();
            for (_, ape) in self.apes.iter().enumerate() {
                self.ape_allocation
                    .insert(ape, allocation_iter.next().unwrap_or(&(0 as Balance)))
            }

            // allocations have been set/updated so all `apes` will need to re-approve
            self.set_agreement_false();
        }

        #[ink(message)]
        pub fn get_contract_balance(&self) -> Balance {
            self.env().balance()
        }

        // #[ink(message)]

        /// Return true if all apes have agreed on `ape_allocation`
        fn apes_in_agreement(&self) -> bool {
            let consensus = true;
            for (_, ape) in self.apes.iter().enumerate() {
                if self.ape_approved.get(ape).unwrap_or(false) == false {
                    return false;
                }
            }

            consensus
        }

        /// Set all ape approval to false if `self.ape_allocation` changes at all.
        fn set_agreement_false(&mut self) {
            for (_, ape) in self.apes.iter().enumerate() {
                self.ape_approved.insert(ape, &false);
            }
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        use hex;
        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;
        use ink_prelude::vec;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let contract = ApeStrapperWasm::default();
            assert_eq!(contract.apes.len(), 0);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn set_ape_allocation_works() {
            let mut contract = ApeStrapperWasm::default();
            assert_eq!(contract.get_apes().len(), 0);
            assert_eq!(contract.get_allocations(), vec![]);
            // Add alice
            // let alice = AccountId::from([
            //     0x31, 0x44, 0x57, 0x54, 0x78, 0x78, 0x58, 0x39, 0x30, 0x78, 0x78, 0x68, 0x46, 0x42,
            //     0x71, 0x39, 0x42, 0x4b, 0x6d, 0x66, 0x31, 0x6f, 0x49, 0x73, 0x68, 0x56, 0x69, 0x46,
            //     0x54, 0x4d, 0x33, 0x6a, 0x6d, 0x6c, 0x61, 0x45, 0x35, 0x36, 0x56, 0x74, 0x6f, 0x6e,
            //     0x30, 0x3d,
            // ]);

            let alice = get_account_from_hex_string(
                "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
            );

            let addresses = vec![alice];
            let allocations = vec![1];

            contract.set_ape_allocation(addresses, allocations);

            let mut allocations = contract.get_allocations();
            assert_eq!(allocations.len(), 1);

            let single_allocation = allocations.pop().unwrap();
            assert_eq!(
                single_allocation.0,
                get_account_from_hex_string(
                    "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
                )
            );
            assert_eq!(single_allocation.1, 1);
        }

        fn get_account_from_hex_string(account_id: &str) -> AccountId {
            let alice_public_key = String::from(account_id);
            let mut alice_bytes: [u8; 32] = [0x0; 32];
            let _ = hex::decode_to_slice(alice_public_key, &mut alice_bytes);
            return AccountId::from(alice_bytes);
        }

        #[ink::test]
        fn zero_balance() {
            let contract = ApeStrapperWasm::default();
            ink_env::debug_println!("Contract Balance: {}", contract.get_contract_balance());
        }
    }
}
