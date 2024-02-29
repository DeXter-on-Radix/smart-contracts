use scrypto::prelude::*;

#[derive(ScryptoSbor, PartialEq, Debug)]
pub enum Status {
    On,
    Off,
}

#[blueprint]
mod stake {
    enable_method_auth! {
        methods {
            stake => PUBLIC;
            unstake => PUBLIC;
            show_redemption_value => PUBLIC;
            show_vault_amount => PUBLIC;
            airdrop => restrict_to: [OWNER];
            emergency_switch => restrict_to: [OWNER];
        }
    }
    #[derive(Debug)]
    // Define what resources and data will be managed by the Stake component
    struct Stake {
        // Native OneResourcePool Blueprint
        stake_pool: Global<OneResourcePool>,
        stake_pool_token: ResourceAddress,
        claim_frequency: u64,
        owner_badge: ResourceAddress,
        platform_badge_vault: NonFungibleVault,
        rewards_pool: Vec<FungibleVault>,
        contract_status: Status,
    }

    impl Stake {
        pub fn instantiate_stake(
            stake_pool_token: ResourceAddress,
            owner_badge: ResourceAddress,
            platform_badge: NonFungibleBucket,
            claim_frequency: u64,
        ) -> Global<Stake> {
            // Check if the claim frequency is greater than zero
            assert!(
                claim_frequency > 0,
                "Claim frequency must be greater than zero"
            );
            // Check if the platform badge amount is exactly one
            assert!(
                platform_badge.amount() == Decimal::from(1),
                "Must deposit exactly one platform badge"
            );
            // Set Up Actor Virtual Badge
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Stake::blueprint_id());

            let global_component_caller_badge =
                NonFungibleGlobalId::global_caller_badge(component_address);

            let owner_role = OwnerRole::Updatable(rule!(require(owner_badge)));

            // Instantiate the OneResourcePool Blueprint
            let stake_pool = Blueprint::<OneResourcePool>::instantiate(
                owner_role.clone(),
                rule!(require(global_component_caller_badge)),
                stake_pool_token,
                None,
            );

            let component = Self {
                stake_pool,
                stake_pool_token,
                claim_frequency,
                owner_badge,
                platform_badge_vault: NonFungibleVault::with_bucket(platform_badge),
                rewards_pool: Vec::new(),
                contract_status: Status::On,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge))))
            .with_address(address_reservation)
            .metadata(metadata! {
             roles {
                    metadata_setter => rule!(require(owner_badge));
                    metadata_setter_updater => rule!(require(owner_badge));
                    metadata_locker => rule!(require(owner_badge));
                    metadata_locker_updater => rule!(require(owner_badge));
                },
                 init {
                    "name" => "Staking Contract".to_string(), updatable;
                   "description" => "A contract that allows users to stake tokens and claim rewards.".to_string(), updatable;
                }
             })
            .globalize();

            return component;
        }

        pub fn stake(&mut self, stake_tokens: Bucket) -> Bucket {
            // Check if the contract is active
            assert!(self.contract_status == Status::On, "Contract is not active");
            // Check if the stake amount is greater than zero
            assert!(
                stake_tokens.amount() > Decimal::from(0),
                "Stake amount must be greater than zero"
            );
            // Check if the staking token matches
            assert!(
                stake_tokens.resource_address() == self.stake_pool_token,
                "Invalid staking token"
            );
            // Stake the tokens and receive the pool units
            let tokens = self.stake_pool.contribute(stake_tokens);

            return tokens;
        }

        pub fn unstake(&mut self, pool_units: Bucket) -> Bucket {
            // Check if the contract is active
            assert!(self.contract_status == Status::On, "Contract is not active");
            // Check if amount is greater than zero
            assert!(
                pool_units.amount() > Decimal::from(0),
                "Unstake amount must be greater than zero"
            );
            // Deposit the pool units in return for the staking tokens
            let tokens = self.stake_pool.redeem(pool_units);

            return tokens;
        }

        pub fn show_redemption_value(&mut self, pool_units: Decimal) {
            // Check if the contract is active
            assert!(self.contract_status == Status::On, "Contract is not active");
            // Check if amount is greater than zero
            assert!(
                pool_units > Decimal::from(0),
                "Unstake amount must be greater than zero"
            );
            // Get the redemption value of the pool units
            self.stake_pool.get_redemption_value(pool_units);
        }

        pub fn show_vault_amount(&mut self) {
            // Check if the contract is active
            assert!(self.contract_status == Status::On, "Contract is not active");
            // Get the amount of tokens in the vault
            self.stake_pool.get_vault_amount();
        }

        pub fn airdrop(&mut self, stake_tokens: Bucket) {
            // Check if the contract is active
            assert!(self.contract_status == Status::On, "Contract is not active");
            // Check if the stake amount is greater than zero
            assert!(
                stake_tokens.amount() > Decimal::from(0),
                "Stake amount must be greater than zero"
            );
            // Check if the staking token matches
            assert!(
                stake_tokens.resource_address() == self.stake_pool_token,
                "Invalid staking token"
            );
            // Deposit tokens into the stake pool
            self.stake_pool.protected_deposit(stake_tokens);
        }

        pub fn emergency_switch(&mut self) {
            // Toggle the contract status
            match self.contract_status {
                Status::On => self.contract_status = Status::Off,
                Status::Off => self.contract_status = Status::On,
            }
            info!(
                "Contract status has been switched to {:?}",
                self.contract_status
            );
        }
    }
}
