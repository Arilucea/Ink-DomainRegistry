#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod domain_registry {
    
    extern crate alloc;
    
    use ink::{storage::Mapping};
    use ink::prelude::{string::String};

    /// The Domain registry result type.
    pub type Result<T> = core::result::Result<T, Error>;

    // Event emitted when a new domain is registered
    #[ink(event)]
    pub struct DomainRegistered {
        domain: String,
        owner: AccountId,
        expiration_data: u64,
    }

    // Event emitted when a domain is renewed
    #[ink(event)]
    pub struct DomainRenewed {
        domain: String,
        owner: AccountId,
        expiration_data: u64,
    }

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
        DomainExpired,
        NotDomainOwner,
        DomainNotExpired,
        NotContractOwner,
    }


    /// A Transaction is what every `owner` can submit for confirmation by other owners.
    /// If enough owners agree it will be executed by the contract.
    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive( Debug, PartialEq, Eq, scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct DomainData {
        owner: AccountId,
        expiration_date: u64,
        metadata: ink::prelude::string::String,
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
        refunds: Mapping<[u8; 32], RefundData>,
        
        requested_domain: Mapping<[u8; 32], AccountId>,
        reserve_time: Mapping<[u8; 32], u64>,

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
        
                default_fee_by_letter: 500,
                min_lock_time: 30 * 24 * 60 * 60,
                locked: bool::default(),
                
                owner: Self::env().caller(),
            }
        }

        /**
         * @dev Change the minimum duration of a domain registration
         */
        #[ink(message)]
        pub fn update_min_lock_time(&mut self, locking_time: u64) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::NotContractOwner);
            } 
            self.min_lock_time = locking_time;
            
            Ok(())
        }

        /**
         * @dev Returns the minimum duration of a domain registration
         */
        #[ink(message)]
        pub fn get_min_lock_time(&mut self) -> u64 {
            return self.min_lock_time;
        }
        
        /**
         * @dev Create a combination with the domain and other information
         * @param domain desired domain
         * @param salt random information
         */
        #[ink(message)]
        pub fn generate_secret(&mut self, domain: String, salt: Hash) -> [u8; 32] {
            return self.generate_secret_internal(&domain, salt);
        }

        /**
         * @dev Return how much cost to rent a specific domain in a period
         * @param domain desired domain
         * @param duration how long is the domain rent (in seconds)
         */
        #[ink(message)]
        pub fn rent_price(&mut self, domain: String, duration: u64) -> u128 {
            return self.rent_price_internal(&domain, duration).unwrap_or_default();
        }

        /**
         * @dev Reserve a domain using the secret generated with the function above
         * @param secret combination of domain and salt 
         */
        #[ink(message)]
        pub fn request_domain(&mut self, secret: [u8; 32]) -> Result<()> {
            // let secret_slice: &str = &secret;
            if self.requested_domain.get(secret) != None {
                    return Err(Error::SecretAlreadyUsed);
            }
            self.requested_domain.insert(secret, &self.env().caller());
            self.reserve_time.insert(secret, &self.env().block_timestamp());
            
            Ok(())
        }
        
        /**
         * @dev Confirm a domain reserve, transaction must be send with enough ether to pay for the duration of the rent
         * @param domain desired domain
         * @param salt random information
         * @param duration how long is the domain rent (in seconds)
         * @param metadata other information realted with the domain
         */
        #[ink(message, payable)]
        pub fn rent_domain(&mut self, domain: String, salt: Hash, duration: u64, metadata: String) -> Result<()> {
            let secret: [u8; 32] = self.generate_secret_internal(&domain, salt);
            
            let requester: AccountId = self.requested_domain.get(secret).unwrap(); 
            if requester != self.env().caller() {
                return Err(Error::NotTheOriginalRequester);
            }
            let reserve_time: u64 = self.reserve_time.get(secret).unwrap(); 
            if reserve_time >= self.env().block_timestamp() {
                return Err(Error::RentCannotBeDoneInSameBlock);
            } 

            let domain_cost: u128 = self.rent_price_internal(&domain, duration).unwrap();
            if self.env().transferred_value() < domain_cost {
                return Err(Error::SentValueIsNotEnough);
            }

            let domain_key: &str = &domain;

            if let Some(domain_data) = self.domains.get(domain_key) {
                if domain_data.expiration_date >= self.env().block_timestamp() {
                    return Err(Error::DomainUnavailable);
                } 
            } 

            let domain_data: DomainData = DomainData {
                owner: self.env().caller(),
                expiration_date: self.env().block_timestamp() + duration,
                metadata: metadata,
            };

            self.domains.insert(domain_key, &domain_data);

            // Refunds
            let refund_key = self.generate_key(&domain);
            let refund_data: RefundData = RefundData { expiration_date: domain_data.expiration_date, paid_price: domain_cost };
            self.refunds.insert(refund_key, &refund_data);

            let lock_balance: u128 = self.locked_balance.get(requester).unwrap_or_default();
            self.locked_balance.insert(requester, &(lock_balance+domain_cost));

            let refund_amount: u128 = self.env().transferred_value() - domain_cost;            
            if self.env().transfer(self.env().caller(), refund_amount).is_err() {
                panic!("Transfer failed")
            }

            self.env().emit_event(DomainRegistered {
                domain: domain,
                owner: self.env().caller(),
                expiration_data: self.env().block_timestamp() + duration,
            });

            Ok(())
        }

    
        /**
         * @dev Extend the renting period of an owned domain 
         * @param domain desired domain
         * @param duration how long is the domain rent (in seconds)
         */
        #[ink(message, payable)]
        pub fn renew_domain(&mut self, domain: String, duration: u64) -> Result<()> {
            let domain_cost: u128 = self.rent_price_internal(&domain, duration).unwrap();
            if self.env().transferred_value() < domain_cost {
                return Err(Error::SentValueIsNotEnough);
            }

            let domain_key: &str = &domain;
            let mut domain_data: DomainData = self.domains.get(domain_key).unwrap();
            if domain_data.expiration_date <= self.env().block_timestamp() {
                return Err(Error::DomainExpired);
            }
            if domain_data.owner != self.env().caller() {
                return Err(Error::NotDomainOwner);
            }

            domain_data.expiration_date = self.env().block_timestamp() + duration;
            self.domains.insert(domain_key, &domain_data);

            // Refunds
            let refund_key = self.generate_key(&domain);
            let refund_data: RefundData = RefundData { expiration_date: domain_data.expiration_date, paid_price: domain_cost };
            self.refunds.insert(refund_key, &refund_data);

            let lock_balance: u128 = self.locked_balance.get(self.env().caller()).unwrap_or_default();
            self.locked_balance.insert(self.env().caller(), &(lock_balance+domain_cost));

            let refund_amount: u128 = self.env().transferred_value() - domain_cost;
            if self.env().transfer(self.env().caller(), refund_amount).is_err() {
                panic!("Transfer failed")
            }

            self.env().emit_event(DomainRenewed {
                domain: domain,
                owner: self.env().caller(),
                expiration_data: self.env().block_timestamp() + duration,
            });

            Ok(())
        }

        /**
         * @dev Request the refund of a expired domain
         * @param domain desired domain
         */
        #[ink(message, payable)]
        pub fn refund_domain(&mut self, domain: String) -> Result<()> {
            let refund_key = self.generate_key(&domain);
            let refund_data: RefundData = self.refunds.get(refund_key).unwrap();
            if refund_data.expiration_date >= self.env().block_timestamp() {
                return Err(Error::DomainNotExpired);
            }

            let amount: u128 = refund_data.paid_price; 
            if amount > 0 {
                let locke_balance: u128 = self.locked_balance.get(self.env().caller()).unwrap_or_default();
                self.locked_balance.insert(self.env().caller(), &(locke_balance-amount));
                if self.env().transfer(self.env().caller(), amount).is_err() {
                    panic!("Transfer failed")
                }    
            }

            Ok(())
        }

        /**
         * @dev Returns information related with the domain
         * @return DomainData 
         * owner address owner of the domain
         * expirationDate timeStamp of the renting expiration
         * metaData other information realted with the domain
         * availability boolean indication is the domain can be rented
         */
        #[ink(message)]
        pub fn get_domain_data(&self, domain: String) -> DomainData {
            if let Some(domain_data) = self.domains.get(domain) {
                return domain_data;
            } else {
                return DomainData {
                        owner: self.zero_address(),
                        expiration_date: u64::default(),
                        metadata: String::default(),
                };
            }
        }

        /**
         * Contract account id function needed by the test
         */
        #[ink(message)]
        pub fn get_id(&self) -> AccountId {
            self.env().account_id()
        }

        // Internal functions
        fn generate_key(&mut self, domain: &String) -> [u8; 32] {
            return self.generate_hash(domain, Hash::default(), self.env().caller())
        }

        fn generate_secret_internal(&mut self, domain: &String, salt: Hash) -> [u8; 32] {
            return self.generate_hash(domain, salt, self.zero_address())
        }

        fn generate_hash(&mut self, domain: &String, salt: Hash, caller: AccountId) -> [u8; 32] {
            let mut hash;

            if salt != Hash::default() {
                let encodable = (domain, salt);
                hash =
                    <ink::env::hash::Sha2x256 as ink::env::hash::HashOutput>::Type::default(); // 256-bit buffer
                ink::env::hash_encoded::<ink::env::hash::Sha2x256, _>(&encodable, &mut hash);
            } else {
                let encodable = (domain, caller);
                hash =
                    <ink::env::hash::Sha2x256 as ink::env::hash::HashOutput>::Type::default(); // 256-bit buffer
                ink::env::hash_encoded::<ink::env::hash::Sha2x256, _>(&encodable, &mut hash);
            }

            return hash
        }

        fn rent_price_internal(&mut self, domain: &String, duration: u64) -> Result<u128> {
            let domain_length = self.domain_length(&domain);
            if domain_length == 0 {
                return Err(Error::DomainLengthIsZero)
            }
            if duration < self.min_lock_time {
                return Err(Error::DurationIsNotEnough)
            }
            let duration: u128 = duration.into();
            return Ok(self.default_fee_by_letter * domain_length * duration);
        }


        fn domain_length(&mut self, domain: &String) -> u128 {
            return domain.len().try_into().unwrap();
        }

        fn zero_address(&self) -> AccountId {
            [0u8; 32].into()
        }
    }

    /// Unit tests
    #[cfg(test)]
    mod domain_registry_tests {
        use super::*;

        fn default_accounts(
        ) -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
            ink::env::test::default_accounts::<Environment>()
        }

        fn rent_domain_env(contract: AccountId, price: u128) {
            let new_block_timestamp: u64 = 1000;
            ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(new_block_timestamp);
            ink_env::test::set_callee::<ink_env::DefaultEnvironment>(contract);
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(default_accounts().alice);
            ink::env::test::set_value_transferred::<ink_env::DefaultEnvironment>(price);
        }

        #[ink::test]
        fn update_min_lock_time_works() {
            // given
            let mut domain_registry = DomainRegistry::new();
            let new_lock_time = 60 * 24 * 60 * 60; // 60 days

            // when
            domain_registry.update_min_lock_time(new_lock_time).unwrap();

            // then
            assert_eq!(domain_registry.get_min_lock_time(), new_lock_time);
        }

        #[ink::test]
        fn generate_secret_works() {
            // given
            let mut domain_registry = DomainRegistry::new();
            let domain = "mydomain".to_string();
            let salt = Hash::from([1u8; 32]);

            // when
            let secret = domain_registry.generate_secret(domain.clone(), salt);

            // then
            let encodable = (domain, salt);
            let mut hash =
                <ink::env::hash::Sha2x256 as ink::env::hash::HashOutput>::Type::default(); // 256-bit buffer
            ink::env::hash_encoded::<ink::env::hash::Sha2x256, _>(&encodable, &mut hash);
            assert_eq!(secret, hash.as_slice());
        }

        #[ink::test]
        fn rent_price_works() {
            // given
            let mut domain_registry = DomainRegistry::new();
            let domain = "mydomain".to_string();
            let duration = 30 * 24 * 60 * 60; // 30 days

            // when
            let price = domain_registry.rent_price(domain.clone(), duration);

            // then
            assert_eq!(price, domain_registry.default_fee_by_letter * domain.len() as u128 * duration as u128);
        }

        #[ink::test]
        fn request_domain_works() {
            // given
            let caller = default_accounts();
            let mut domain_registry = DomainRegistry::new();
            let domain = "mydomain".to_string();
            let salt = Hash::from([1u8; 32]);
            let secret = domain_registry.generate_secret(domain.clone(), salt);

            // when
            domain_registry.request_domain(secret).unwrap();

            // then
            assert_eq!(domain_registry.requested_domain.get(secret), Some(caller.alice));
        }
        
        #[ink::test]
        fn test_rent_domain() {
            // Initialize a contract instance
            let mut domain_registry = DomainRegistry::new();

            let owner = default_accounts().alice;
            let domain = "example.com".to_string();
            let duration = 60 * 60 * 24 * 31; // 1 year in seconds
            let secret = domain_registry.generate_secret(domain.clone(), Hash::default());
            let price = domain_registry.rent_price(domain.clone(), duration);
            
            domain_registry.request_domain(secret).unwrap();
            rent_domain_env(domain_registry.get_id(), price);

            // Call the rent_domain function
            let result = domain_registry.rent_domain(domain.clone(), Hash::default(), duration, "meta_data".to_string());

            // Check that the result is Ok
            assert_eq!(result, Ok(()));

            // Check that the domain data was stored correctly
            let domain_data = domain_registry.domains.get(&domain).unwrap();
            assert_eq!(&domain_data.owner, &owner);
            assert_eq!(&domain_data.expiration_date, &(duration+1000));
        }

        #[ink::test]
        fn test_renew_domain() {
            // Initialize a contract instance
            let mut domain_registry = DomainRegistry::new();

            let owner = default_accounts().alice;
            let domain = "example.com".to_string();
            let duration = 60 * 60 * 24 * 31; // 1 year in seconds
            let secret = domain_registry.generate_secret(domain.clone(), Hash::default());
            let price = domain_registry.rent_price(domain.clone(), duration);
            
            // Rent the domain
            domain_registry.request_domain(secret).unwrap();
            rent_domain_env(domain_registry.get_id(), price);
            let _result = domain_registry.rent_domain(domain.clone(), Hash::default(), duration, "meta_data".to_string());
                  
            // Renew the domain
            ink_env::test::set_callee::<ink_env::DefaultEnvironment>(domain_registry.get_id());
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(default_accounts().alice);
            ink::env::test::set_value_transferred::<ink_env::DefaultEnvironment>(price*2);

            let new_expiration_date = 60 * 60 * 24 * 62;
            domain_registry.renew_domain(domain.clone(), new_expiration_date).unwrap();

            // Check that the domain was updated
            let updated_domain_data = domain_registry.domains.get(&domain).unwrap();
            assert_eq!(updated_domain_data.owner, owner);
            assert_eq!(updated_domain_data.expiration_date, new_expiration_date+1000);
            assert_eq!(updated_domain_data.metadata, "meta_data");
        }
    }
}
