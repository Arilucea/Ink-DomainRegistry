#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod domain_registry {
    
    extern crate alloc;
    
    use ink::{storage::Mapping};
    use alloc::{string::{String}};
    use sha3::{Digest};

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

        /**
         * @dev Return how much cost to rent a specific domain in a period
         * @param domain desired domain
         * @param duration how long is the domain rent (in seconds)
         */
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

        /**
         * @dev Reserve a domain using the secret generated with the function above
         * @param secret combination of domain and salt 
         */
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

        /**
         * @dev Confirm a domain reserve, transaction must be send with enough ether to pay for the duration of the rent
         * @param domain desired domain
         * @param salt random information
         * @param duration how long is the domain rent (in seconds)
         * @param metadata other information realted with the domain
         */
        #[ink(message, payable)]
        pub fn rent_domain(&mut self, domain: String, salt: Hash, duration: u64, metadata: String) -> Result<()> {
            let domain_slice: &str = &domain;
            let secret: &str = &(self.generate_secret(domain_slice.to_string(), salt));
            
            let requester: AccountId = self.requested_domain.get(secret).unwrap(); 
            if requester != self.env().caller() {
                return Err(Error::NotTheOriginalRequester);
            }
            let reserve_time: u64 = self.reserve_time.get(secret).unwrap(); 
            if reserve_time < self.env().block_timestamp() {
                return Err(Error::RentCannotBeDoneInSameBlock);
            } 

            let domain_cost: u128 = self.rent_price(domain_slice.to_string(), duration).unwrap();
            if self.env().transferred_value() < domain_cost {
                return Err(Error::SentValueIsNotEnough);
            }

            let domain_key: &str = &domain;

            if self.domains.get(domain_key) != None {
                let domain_data: DomainData = self.domains.get(domain_key).unwrap();
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
            let refund_key = self.create_key(domain_slice.to_string());
            let refund_data: RefundData = RefundData { expiration_date: domain_data.expiration_date, paid_price: domain_cost };
            self.refunds.insert(refund_key, &refund_data);

            let lock_balance: u128 = self.locked_balance.get(requester).unwrap_or_default();
            self.locked_balance.insert(requester, &(lock_balance+domain_cost));

            let refund_amount: u128 = self.env().transferred_value() - domain_cost;            
            if self.env().transfer(self.env().caller(), refund_amount).is_err() {
                panic!("Transfer failed")
            }

            self.env().emit_event(DomainRegistered {
                domain: domain_slice.to_string(),
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
            let domain_slice: &str = &domain;

            let domain_cost: u128 = self.rent_price(domain_slice.to_string(), duration).unwrap();
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
            let refund_key = self.create_key(domain_slice.to_string());
            let refund_data: RefundData = RefundData { expiration_date: domain_data.expiration_date, paid_price: domain_cost };
            self.refunds.insert(refund_key, &refund_data);

            let lock_balance: u128 = self.locked_balance.get(self.env().caller()).unwrap_or_default();
            self.locked_balance.insert(self.env().caller(), &(lock_balance+domain_cost));

            let refund_amount: u128 = self.env().transferred_value() - domain_cost;
            if self.env().transfer(self.env().caller(), refund_amount).is_err() {
                panic!("Transfer failed")
            }

            self.env().emit_event(DomainRenewed {
                domain: domain_slice.to_string(),
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
            let refund_key = self.create_key(domain);
            let refund_data: RefundData = self.refunds.get(refund_key).unwrap();
            if refund_data.expiration_date <= self.env().block_timestamp() {
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
            let domain_data: DomainData = self.domains.get(domain).unwrap();
            return domain_data;
        }

        // Internal functions
        fn create_key(&mut self, domain: String) -> String {
            let mut hasher = sha3::Keccak256::new();
            hasher.update(domain);
            hasher.update(self.env().caller());
            let secret: String = format!("{:X}", hasher.finalize());

            return secret;
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

            let hash = domain_registry.rent_price("aaaaaaaaa".to_string(), 10000000000000);
            println!("{}", hash.unwrap());

            let hash = domain_registry.request_domain("aaaaaaa".to_string());
            println!("{:?}", hash);

            assert_eq!(domain_registry.domain_length("casa".to_string()), 4);
        }

    }
}
