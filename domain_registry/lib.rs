#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod domain_registry {
    
    extern crate alloc;
    
    use ink::{storage::Mapping};
    use alloc::{string::{String}};
    use scale::HasCompact;
    use sha3::{Digest};

    /// The Domain registry result type.
    pub type Result<T> = core::result::Result<T, Error>;

    /// The Domain registry error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        DurationIsNotEnough,
        DomainLengthIsZero,
        SecretAlreadyUsed,
        NotTheOriginalRequester,
        RentCannotBeDoneInSameBlock,
        SentValueIsNotEnough,
        DomainUnavailable,
    }


    /// A Transaction is what every `owner` can submit for confirmation by other owners.
    /// If enough owners agree it will be executed by the contract.
    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive( Debug, PartialEq, Eq, scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct DomainData {
        owner: AccountId,
        expiration_date: u64,
        metadata: String,
    }
    
    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive( Debug, PartialEq, Eq, scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct RefundData {
        expiration_date: u64,
        paid_price: Balance
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct DomainRegistry {
        domains: Mapping<String, DomainData>,
        refunds: Mapping<String, RefundData>,
        
        requested_domain: Mapping<String, AccountId>,
        reserve_time: Mapping<String, u64>,

        locked_balance: Mapping<AccountId, Balance>,

        default_fee_by_letter: Balance,
        min_lock_time: u64,
        locked: bool,

        owner: AccountId,
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
                locked: bool::default(),
                
                owner: Self::env().caller(),
            }
        }

        /**
         * @dev Change the minimum duration of a domain registration
         */
        #[ink(message)]
        pub fn update_min_lock_time(&mut self, locking_time: u64) {
            self.min_lock_time = locking_time;
        }
        
        /**
         * @dev Create a combination with the domain and other information
         * @param domain desired domain
         * @param salt random information
         */
        #[ink(message)]
        pub fn generate_secret(&self, domain: String, salt: Hash) -> String {
            let mut hasher = sha3::Keccak256::new();
            hasher.update(domain);
            hasher.update(salt);
            let secret: String = format!("{:X}", hasher.finalize());

            return secret;
        }

        #[ink(message)]
        pub fn rent_price(&mut self, domain: String, duration: u64) -> Result<u128> {
            let domain_length = self.domain_length(domain);
            if domain_length == 0 {
                return Err(Error::DomainLengthIsZero)
            }
            if duration < self.min_lock_time {
                return Err(Error::DurationIsNotEnough)
            }
            let duration: u128 = duration.into();
            return Ok(domain_length * duration);
        }

        #[ink(message)]
        pub fn request_domain(&mut self, secret: String) -> Result<()> {
            let secret_slice: &str = &secret;
            if self.requested_domain.get(secret_slice) != None {
                    return Err(Error::SecretAlreadyUsed);
            }
            self.requested_domain.insert(secret_slice, &self.env().caller());
            self.reserve_time.insert(secret_slice, &self.env().block_timestamp());
            
            Ok(())
        }

        

        fn domain_length(&mut self, domain: String) -> u128 {
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

        fn default_accounts(
        ) -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
            ink::env::test::default_accounts::<Environment>()
        }

        fn set_next_caller(caller: AccountId) {
            ink::env::test::set_caller::<Environment>(caller);
        }

        /// We test if the default constructor does its job.
        #[ink::test]
        fn deploy_works() {
            let mut domain_registry = DomainRegistry::new();

            let salt = Hash::from([0x1; 32]);
            let hash = domain_registry.generate_secret("aaaaaaaaaa".to_string(), salt);
            println!("{}", hash);

            domain_registry.request_domain("aaaaaaaaaa".to_string());

            let hash = domain_registry.testFunc("aaaaaaaaaa".to_string(), salt);
            println!("{:?}", hash);

            let hash = domain_registry.rent_price("aaaaaaaaa".to_string(), 10000000000000);
            println!("{}", hash.unwrap());

            let hash = domain_registry.request_domain("aaaaaaa".to_string());
            println!("{:?}", hash);

            assert_eq!(domain_registry.domain_length("casa".to_string()), 4);
        }

    }
}
