#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[brush::contract]
pub mod my_payment_splitter {
    use brush::contracts::payment_splitter::*;
    use ink_prelude::vec::Vec;
    use ink_storage::traits::SpreadAllocate;

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, PaymentSplitterStorage)]
    pub struct ApeStrapperWasm {
        #[PaymentSplitterStorageField]
        splitter: PaymentSplitterData,
    }

    impl ApeStrapperWasm {
        #[ink(constructor)]
        pub fn new(payees_and_shares: Vec<(AccountId, Balance)>) -> Self {
            ink_lang::utils::initialize_contract(|instance: &mut Self| {
                instance._init(payees_and_shares).expect("Should init");
            })
        }

        #[ink(message)]
        pub fn release_all(&mut self) -> Result<(), PaymentSplitterError> {
            let payees = self.get().payees.clone();

            for account in payees.into_iter() {
                self.release(account)?;
            }

            Ok(())
        }
    }

    impl PaymentSplitter for ApeStrapperWasm {}

    #[cfg(test)]
    mod tests {
        use super::*;

        use brush::test_utils::accounts;
        use ink_env::{AccountId, *};
        use ink_lang as ink;
        use ink_prelude::vec;
        use ink_storage::traits::SpreadAllocate;

        use ink::codegen::{EmitEvent, Env};

        #[ink::test]
        fn new_constructor_works() {
            let accounts = accounts();
            let instance = ApeStrapperWasm::new(vec![(accounts.bob, 50), (accounts.charlie, 50)]);
            assert_eq!(instance.get().payees.len(), 2);
        }

        #[ink::test]
        fn release_all_works() {
            let accounts = accounts();
            let mut instance =
                ApeStrapperWasm::new(vec![(accounts.bob, 60), (accounts.charlie, 40)]);

            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(accounts.charlie, 0);
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(accounts.bob, 0);
            let amount = 1_000;

            // Add `amount` to the contract balance
            let caller = instance.caller();
            // add_funds(instance.account_id(), amount);

            // Verifications
            // assert_eq!(instance.get().balance, amount);

            // instance.release_all();

            //----------------------

            // let accounts = accounts();
            // let mut instance =
            //     ApeStrapperWasm::new(vec![(accounts.charlie, 100), (accounts.bob, 200)]);
            // ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(accounts.charlie, 0);
            // ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(accounts.bob, 0);
            // let amount = 1000000;
            // add_funds(instance.env().account_id(), amount);

            // assert_eq!(100 + 200, instance.total_shares());
            // assert!(instance.release(accounts.charlie).is_ok());
            // assert_eq!(333333, instance.total_released());
            // assert!(instance.release(accounts.bob).is_ok());
            // assert_eq!(999999, instance.total_released());
            // assert_eq!(333333, instance.released(accounts.charlie));
            // assert_eq!(
            //     333333,
            //     ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(accounts.charlie)
            //         .unwrap()
            // );
            // assert_eq!(2 * 333333, instance.released(accounts.bob));
            // assert_eq!(
            //     2 * 333333,
            //     ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(accounts.bob)
            //         .unwrap()
            // );
        }

        fn add_funds(account: AccountId, amount: Balance) {
            let balance = ink_env::balance::<ink_env::DefaultEnvironment>();
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                account,
                balance + amount,
            );
        }
    }
}
