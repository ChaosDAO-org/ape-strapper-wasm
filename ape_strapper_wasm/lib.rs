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

        /// Payout all payees at once.
        #[ink(message)]
        pub fn release_all(&mut self) -> Result<(), PaymentSplitterError> {
            // `_release_all()` is an internal method defined by the `PaymentSplitterInternal` trait
            self._release_all()
        }
    }

    impl PaymentSplitter for ApeStrapperWasm {}

    #[cfg(test)]
    mod tests {
        use super::*;

        // use alloc::fmt;
        use brush::test_utils::accounts;
        use ink_env::AccountId;
        use ink_lang as ink;
        use ink_prelude::vec;

        use ink::codegen::Env;

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
                ApeStrapperWasm::new(vec![(accounts.charlie, 100), (accounts.bob, 200)]);
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(accounts.charlie, 0);
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(accounts.bob, 0);
            let amount = 1000000;
            add_funds(instance.env().account_id(), amount);

            assert_eq!(100 + 200, instance.total_shares());
            assert!(instance._release_all().is_ok());
            assert_eq!(999999, instance.total_released());
            assert_eq!(333333, instance.released(accounts.charlie));
            assert_eq!(
                333333,
                ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(accounts.charlie)
                    .unwrap()
            );
            assert_eq!(2 * 333333, instance.released(accounts.bob));
            assert_eq!(
                2 * 333333,
                ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(accounts.bob)
                    .unwrap()
            );
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
