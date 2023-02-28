#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod domain_registry {
    
    extern crate alloc;
    
    use ink::{storage::Mapping};
    use alloc::{string::{String}};

    /// A Transaction is what every `owner` can submit for confirmation by other owners.
    /// If enough owners agree it will be executed by the contract.
    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive( Debug, PartialEq, Eq, scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct DomainData {
        owner: AccountId,
        expiration_date: u128,
        metadata: String,
    }
    
    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive( Debug, PartialEq, Eq, scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct RefundData {
        expiration_date: u128,
        paid_price: u128
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct DomainRegistry {
        domains: Mapping<Hash, DomainData>,
        refunds: Mapping<Hash, RefundData>,
        
        requested_domain: Mapping<Hash, AccountId>,
        reserve_time: Mapping<Hash, u128>,

        locked_balance: Mapping<AccountId, u128>,

        default_fee_by_letter: u128,
        min_lock_time: u128,
        locked: bool,
    }

    impl DomainRegistry {
        /// Constructor
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {domains: Mapping::default(),
                refunds: Mapping::default(),
                requested_domain: Mapping::default(),
                reserve_time: Mapping::default(),
                locked_balance: Mapping::default(),
        
                default_fee_by_letter: 500000000,
                min_lock_time: 30 * 24 * 60 * 60,
                locked: bool::default() }
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn setter(&mut self) {
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn getter(&self) -> u128 {
            return self.default_fee_by_letter;
        }

        #[ink(message)]
        pub fn generate_secret(&self, domain: String, salt: Hash) -> Hash {
            return Hash::from([0x1; 32]);
        }

        #[ink(message)]
        pub fn domain_length(&mut self, domain: String) -> u128{
            return domain.len().try_into().unwrap();
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
        fn deploy_works() {
            let mut domain_registry = DomainRegistry::new();
            assert_eq!(domain_registry.getter(), 500000000);
            assert_eq!(domain_registry.domain_length("casa".to_string()), 4);
        }

    }
}
