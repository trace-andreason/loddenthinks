#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod loddenthinks {

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Loddenthinks {
        pub lodden: AccountId,
        pub players: ink_storage::collections::HashMap<AccountId, Balance>,
        pub question: String,
        pub turn: AccountId,
        pub guess: u64,
    }

    impl Loddenthinks {
        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                lodden: Default::default(),
                players: Default::default(),
                question: String::from(""),
                turn: Default::default(),
                guess: Default::default(),
            }
        }

        #[ink(message)]
        pub fn current_guess(&self) -> u64 {
            self.guess
        }

        #[ink(message)]
        pub fn get_lodden(&self) -> AccountId {
            self.lodden
        }

        #[ink(message)]
        pub fn be_lodden(&mut self) -> bool {
            let caller = Self::env().caller();
            if caller == AccountId::from([0x0; 32]) {
                return false
            }
            self.lodden = caller;
            return true
        }

        #[ink(message)]
        pub fn be_player(&mut self) -> bool {
            let caller = Self::env().caller();
            // can't be both a player and lodden
            if caller == self.lodden {
                return false;
            }
            // max two players
            if self.players.len() == 2 {
                return false;
            }
            self.players.insert(caller, 0);
            return true
        }

        fn is_player(&self) -> bool {
            return self.players.contains_key(&Self::env().caller());
        }

        #[ink(message)]
        pub fn set_question(&mut self, question: String) -> bool {
            if !self.is_player() {
                return false;
            }
            self.question = question;
            return true
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_env::{
            call,
            test,
        };

        const WALLET: [u8; 32] = [7; 32];
        fn set_sender(sender: AccountId) {
            test::push_execution_context::<Environment>(
                sender,
                WALLET.into(),
                1000000,
                1000000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
        }

        /// We test if the default constructor does its job.
        #[test]
        fn new_works() {
            let loddenthinks = Loddenthinks::new();
            assert_eq!(loddenthinks.current_guess(), Default::default());
        }

        /// We test if be lodden works
        #[test]
        fn be_lodden_works() {
            let mut loddenthinks = Loddenthinks::new();
            set_sender(AccountId::from([0x1; 32]));
            assert_eq!(loddenthinks.get_lodden(), AccountId::from([0x0; 32]));
            assert_eq!(loddenthinks.be_lodden(), true);
            assert_eq!(loddenthinks.get_lodden(), AccountId::from([0x1; 32]));
        }

        /// We test if be player works
        #[test]
        fn be_players_works() {
            let mut loddenthinks = Loddenthinks::new();
            // uninitialized, 0x0 is technically lodden
            set_sender(AccountId::from([0x0; 32]));
            assert_eq!(loddenthinks.be_player(), false);
            set_sender(AccountId::from([0x1; 32]));
            assert_eq!(loddenthinks.be_player(), true);
            set_sender(AccountId::from([0x2; 32]));
            assert_eq!(loddenthinks.be_player(), true);
            set_sender(AccountId::from([0x3; 32]));
            assert_eq!(loddenthinks.be_player(), false);
        }

        /// We test if is player works
        #[test]
        fn is_players_works() {
            let mut loddenthinks = Loddenthinks::new();
            set_sender(AccountId::from([0x0; 32]));
            assert_eq!(loddenthinks.is_player(), false);
            set_sender(AccountId::from([0x1; 32]));
            assert_eq!(loddenthinks.be_player(), true);
            assert_eq!(loddenthinks.is_player(), true);
        }

        /// We test if set question works
        #[test]
        fn set_question_works() {
            let mut loddenthinks = Loddenthinks::new();
            set_sender(AccountId::from([0x1; 32]));
            assert_eq!(loddenthinks.be_player(), true);
            assert_eq!(loddenthinks.set_question(String::from("works?")), true);
            assert_eq!(loddenthinks.question, String::from("works?"));
            set_sender(AccountId::from([0x2; 32]));
            assert_eq!(loddenthinks.set_question(String::from("no effect")), false);
            assert_eq!(loddenthinks.question, String::from("works?"));
        }
    }
}
