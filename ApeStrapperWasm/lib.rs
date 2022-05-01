#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod ape_strapper_wasm {

    use ink_prelude::vec::*;
    use ink_storage::traits::SpreadAllocate;
    use ink_storage::Mapping;

    #[cfg(not(feature = "ink-as-dependency"))]
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

    pub type Result<T> = core::result::Result<T, Error>;

    #[ink(event)]
    pub struct AllocationSet {
        #[ink(topic)]
        caller: AccountId,
        num_apes: u32,
    }

    #[ink(event)]
    pub struct PayoutExecuted {
        #[ink(topic)]
        caller: AccountId,
        total_paid: Balance,
    }

    #[ink(event)]
    pub struct ApeApproved {
        #[ink(topic)]
        ape: AccountId,
        #[ink(topic)]
        value: bool,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Balance to low to call `nanner_time()`. Contract must have at least 1 UNIT.
        BalanceTooLow,
        /// One of the transfer calls in `nanner_time()` failed
        TransferFailed,
        /// Contract can only pay out if all apes are in agreement on allocations
        ApesNotInAgreement,
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
            let transformed_allocations = Self::percentage_to_units(allocations)
                .expect("Percentage_to_units conversion failed");
            let mut allocation_iter = transformed_allocations.iter();

            for (_, ape) in self.apes.iter().enumerate() {
                // Set the allocation for each ape
                self.ape_allocation
                    .insert(&ape, allocation_iter.next().unwrap_or(&(0 as Balance)));
                self.ape_approved.insert(ape, &false);
            }

            // allocations have been set/updated so all `apes` will need to re-approve
            self.set_agreement_false();
            self.env().emit_event(AllocationSet {
                caller: self.env().caller(),
                num_apes: self.apes.len() as u32,
            })
        }

        /// Converts from an integer representing a percentage to the full UNIT representation
        /// ex: 13% entered as 13 will be convted to 130_000_000_000.
        fn percentage_to_units(allocations: Vec<Balance>) -> Result<Vec<Balance>> {
            const PERCENTAGE_DECIMALS: u32 = 10;
            let transformed: Vec<Balance> = allocations
                .iter()
                .map(|percentage| percentage * ((10 as u128).pow(PERCENTAGE_DECIMALS)))
                .collect();
            Ok(transformed)
        }

        #[ink(message)]
        pub fn get_contract_balance(&self) -> Balance {
            self.env().balance()
        }

        /// Pays apes according to their allocation from TOKEN funds held by this contract
        #[ink(message)]
        pub fn nanner_time(&self) -> Result<Balance> {
            // do actual payout
            const DECIMALS: u32 = 12;
            // Actual contract balance
            let balance = self.env().balance();
            let contract_safe_balance = balance
                .checked_sub(self.env().minimum_balance())
                .unwrap_or(0);

            // Sum of
            let mut total_paid = 0;

            // Less than 1 UNIT
            if contract_safe_balance < 1 {
                return Err(Error::BalanceTooLow);
            } else if !self.apes_in_agreement() {
                return Err(Error::ApesNotInAgreement);
            }

            for (_, ape) in self.apes.iter().enumerate() {
                // Get individual ape allocation
                let allocation = self.ape_allocation.get(ape).unwrap();
                // Calculate amount to pay
                let amount: Balance = (contract_safe_balance
                    .checked_mul(allocation)
                    .expect("Overflow"))
                    / (10 as Balance).pow(DECIMALS);
                total_paid += amount;

                // Transfer `amount` from contract to `ape`
                if let Err(_) = self.env().transfer(*ape, amount) {
                    return Err(Error::TransferFailed);
                }
            }

            self.env().emit_event(PayoutExecuted {
                caller: self.env().caller(),
                total_paid,
            });
            Ok(total_paid)
        }

        /// Each ape in `apes` must call this function to give the allocation their stamp of approval
        /// Token payouts with `nanner_time()` cannot happen unless all apes have approved.
        #[ink(message)]
        pub fn ape_set_agreement(&mut self) {
            let caller = self.env().caller();
            if !self.ape_approved.get(caller).unwrap_or(false) {
                self.ape_approved.insert(caller, &true);
                self.env().emit_event(ApeApproved {
                    ape: caller,
                    value: true,
                });
            }
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

            allocation_setup(&mut contract);

            let mut allocations = contract.get_allocations();
            assert_eq!(allocations.len(), 2);

            let fifty_percent: Balance = 500_000_000_000;

            // Test Charlie account
            let (charlie_account_id, charlie_allocation) = allocations.pop().unwrap();
            // AccountId correct?
            assert_eq!(
                charlie_account_id,
                get_account_from_hex_string(
                    "0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22"
                )
            );
            // Allocation correct?
            assert_eq!(charlie_allocation, fifty_percent);

            // Test Bob account
            let (bob_account_id, bob_allocation) = allocations.pop().unwrap();
            // AccountId correct?
            assert_eq!(
                bob_account_id,
                get_account_from_hex_string(
                    "0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"
                )
            );
            // Allocation correct?
            assert_eq!(bob_allocation, fifty_percent);
        }

        fn get_account_from_hex_string(account_id: &str) -> AccountId {
            let alice_public_key = String::from(account_id);
            let mut alice_bytes: [u8; 32] = [0x0; 32];
            let _ = hex::decode_to_slice(alice_public_key, &mut alice_bytes);
            return AccountId::from(alice_bytes);
        }

        fn allocation_setup(contract: &mut ApeStrapperWasm) {
            let alice = get_account_from_hex_string(
                "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
            );
            let bob = get_account_from_hex_string(
                "0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48",
            );
            let charlie = get_account_from_hex_string(
                "0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22",
            );

            // Bob & Charlie are the only 2 apes
            let addresses = vec![bob, charlie];
            // Bob and Charlie are assigned 50% allocation each for payouts with `nanner_time()`
            let allocations = vec![50, 50];

            contract.set_ape_allocation(addresses, allocations);
        }

        #[ink::test]
        fn zero_balance() {
            let contract = ApeStrapperWasm::default();
            ink_env::debug_println!("Contract Balance: {}", contract.get_contract_balance());
        }

        #[ink::test]
        fn approval_initially_false() {
            let mut contract = ApeStrapperWasm::default();

            let alice = get_account_from_hex_string(
                "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
            );
            contract.set_ape_allocation(vec![alice], vec![1]);
            assert_eq!(contract.ape_approved.get(alice).unwrap(), false);
        }

        #[ink::test]
        fn nanner_time_works() {
            let mut contract = ApeStrapperWasm::default();

            allocation_setup(&mut contract);

            // let total_paid = contract.nanner_time().unwrap();
            // assert_ne!(total_paid, 100_000);
        }
    }
}
