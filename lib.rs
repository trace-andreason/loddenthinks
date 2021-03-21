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
        pub turn: AccountId,
        pub guess: u64,
        pub current_over: u64,
        pub wait_for_reveal: bool,
    }

    #[ink(event)]
    pub struct NewLodden {
        address: AccountId,
    }

    #[ink(event)]
    pub struct NewPlayer {
        address: AccountId,
    }

    #[ink(event)]
    pub struct Bet {
        current_over: u64,
        turn: AccountId,
    }

    #[ink(event)]
    pub struct Result {
        winner: AccountId,
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
                turn: Default::default(),
                guess: Default::default(),
                current_over: Default::default(),
                wait_for_reveal: false,
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
        // TODO: is this right?
        pub fn be_lodden(&mut self) -> bool {
            if self.is_player() {
                return false;
            }
            let caller = Self::env().caller();
            // if self.lodden != AccountId::from([0x0; 32]) {
            //     return false;
            // }
            self.lodden = caller;
            self.env()
                .emit_event(
                    NewLodden {
                        address: self.lodden,
                    }
                );
            true
        }

        #[ink(message)]
        // TODO: convert to 2 phase commit
        pub fn reveal(&mut self) -> bool {
            if !self.is_lodden() {
                return false;
            }

            if !self.wait_for_reveal {
                return false;
            }

            true
        }

        #[ink(message)]
        pub fn be_player(&mut self) -> bool {
            let caller = Self::env().caller();
            // can't be both a player and lodden
            if self.is_lodden() {
                return false;
            }
            // max two players
            if self.players.len() == 2 {
                return false;
            }

            self.players.insert(caller, 0);

            if self.turn == AccountId::from([0x0; 32]) {
                self.turn = caller;
            }

            self.env()
                .emit_event(
                    NewPlayer {
                        address: caller,
                    }
                );

            true
        }

        #[ink(message)]
        pub fn bet(&mut self, bet: u64) -> bool {
            if !self.is_turn() {
                return false;
            }
            if bet < self.current_over {
                return false;
            }
            self.current_over = bet;
            self.swap_turn();

            self.env()
                .emit_event(
                    Bet {
                        current_over: self.current_over,
                        turn: self.turn,
                    }
                );

            true
        }

        #[ink(message)]
        pub fn stay(&mut self) -> bool {
            if !self.is_turn() {
                return false;
            }
            self.wait_for_reveal = true;
            self.env()
                .emit_event(
                    Result {
                        winner: self.winner(),
                    }
                );
            true
        }

        #[ink(message)]
        pub fn guess(&mut self, guess: u64) -> bool {
            if !self.is_lodden() {
                return false;
            }

            self.guess = guess;
            true
        }

        fn swap_turn(&mut self) {
            self.turn = self.get_not_turn();
        }

        fn get_not_turn(&self) -> AccountId {
            for (key, _) in &self.players {
                if key != &self.turn {
                    return *key;
                }
            }
            // should never hit this
            return AccountId::from([0x0; 32]);
        }

        fn is_player(&self) -> bool {
            self.players.contains_key(&Self::env().caller())
        }

        fn is_lodden(&self) -> bool {
            self.lodden == Self::env().caller()
        }

        fn is_turn(&self) -> bool {
            self.turn == Self::env().caller()
        }

        fn winner(&self) -> AccountId {
            if self.current_over >= self.guess {
                return self.turn;
            }
            return self.get_not_turn();
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

        /// We test if is turn works
        #[test]
        fn is_turn_works() {
            let mut loddenthinks = Loddenthinks::new();
            set_sender(AccountId::from([0x1; 32]));
            assert_eq!(loddenthinks.be_player(), true);
            assert_eq!(loddenthinks.is_turn(), true);

            set_sender(AccountId::from([0x2; 32]));
            assert_eq!(loddenthinks.be_player(), true);
            assert_eq!(loddenthinks.is_turn(), false);
        }

        /// We test if get_not_turn works
        #[test]
        fn get_not_turn_works() {
            let mut loddenthinks = Loddenthinks::new();
            set_sender(AccountId::from([0x1; 32]));
            assert_eq!(loddenthinks.be_player(), true);
            assert_eq!(loddenthinks.get_not_turn(), AccountId::from([0x0; 32]));

            set_sender(AccountId::from([0x2; 32]));
            assert_eq!(loddenthinks.be_player(), true);
            assert_eq!(loddenthinks.get_not_turn(), AccountId::from([0x2; 32]));
        }

        /// We test if a full game works
        #[test]
        fn test_full_game() {
            let lodden = AccountId::from([0x1; 32]);
            let player1 = AccountId::from([0x2; 32]);
            let player2 = AccountId::from([0x3; 32]);

            let mut loddenthinks = Loddenthinks::new();
            set_sender(lodden);
            assert_eq!(loddenthinks.be_lodden(), true);
            assert_eq!(loddenthinks.guess(10), true);


            set_sender(player1);
            assert_eq!(loddenthinks.be_player(), true);

            set_sender(player2);
            assert_eq!(loddenthinks.be_player(), true);

            set_sender(player1);
            assert_eq!(loddenthinks.bet(2), true);

            set_sender(player2);
            assert_eq!(loddenthinks.bet(4), true);

            set_sender(player1);
            assert_eq!(loddenthinks.bet(11), true);

            set_sender(player2);
            assert_eq!(loddenthinks.stay(), true);
        }

    }
}
