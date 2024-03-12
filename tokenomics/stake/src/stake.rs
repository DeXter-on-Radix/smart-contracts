use scrypto::prelude::*;

#[derive(ScryptoSbor, PartialEq, Debug)]
pub enum Status {
    On,
    Off,
}

#[derive(ScryptoSbor, NonFungibleData, Debug)]
pub struct NFTClaimReceiptData {
    stake_token_actual: ResourceAddress,
    stake_pool_synth_token: ResourceAddress,
    // Unstake period denominated in number of epochs
    #[mutable]
    unstake_period_end: u64,
    #[mutable]
    pool_units: Decimal,
    #[mutable]
    pending_unstake_amount: Decimal,
}

#[blueprint]
mod stake {

    enable_method_auth! {
        methods {
            stake => PUBLIC;
            unstake => PUBLIC;
            withdraw_stake => PUBLIC;
            show_redemption_value => PUBLIC;
            show_vault_amount => PUBLIC;
            check_unstake_status => PUBLIC;
            deposit => restrict_to: [OWNER];
            update_unstake_period => restrict_to: [OWNER];
            emergency_switch => restrict_to: [OWNER];
            emergency_withdraw_with_pool_units => PUBLIC;
            emergency_withdraw_with_nft_claim_receipt => PUBLIC;
            emergency_withdraw_pool_bypass_with_pool_units => PUBLIC;
            emergency_withdraw_pool_bypass_with_nft_claim_receipt => PUBLIC;
        }
    }
    #[derive(Debug)]
    // Define what resources and data will be managed by the Stake component
    struct Stake {
        // Component staking
        stake_vault_actual: FungibleVault,
        stake_token_actual: ResourceAddress,
        stake_vault_lp_token: Vault,
        // Define the unstake period in epochs
        unstake_period: u64,
        // Component staking NFT receipt
        nft_claim_receipt_resource_manager: ResourceManager,
        // Native account blueprint
        dapp_definition_account: Global<Account>,
        dapp_definition_address: GlobalAddress,
        // Native OneResourcePool blueprint
        stake_pool_synth: Global<OneResourcePool>,
        stake_pool_synth_token_manager: ResourceManager,
        stake_pool_synth_name: String,
        stake_pool_synth_token_symbol: String,
        stake_pool_lp_token_name: String,
        stake_pool_lp_token_manager: ResourceManager,
        stake_pool_lp_token_symbol: String,
        stake_pool_synth_description: String,
        // Owner badge
        owner_badge: ResourceAddress,
        // Contract status
        contract_status: Status,
    }

    impl Stake {
        pub fn instantiate_stake(
            stake_token_actual: ResourceAddress,
            unstake_period: u64,
            stake_pool_synth_name: String,
            stake_pool_synth_token_symbol: String,
            stake_pool_lp_token_name: String,
            stake_pool_lp_token_symbol: String,
            stake_pool_synth_description: String,
            owner_badge: Bucket,
        ) -> (Global<Stake>, Bucket) {
            // Set up actor virtual badge
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Stake::blueprint_id());

            let global_component_caller_badge =
                NonFungibleGlobalId::global_caller_badge(component_address);

            let owner_role =
                OwnerRole::Updatable(rule!(require(owner_badge.resource_address().clone())));

            // Create the NFT claim receipt for the staking component
            let nft_claim_receipt = ResourceBuilder::new_ruid_non_fungible::<NFTClaimReceiptData>(
                OwnerRole::Updatable(rule!(require(owner_badge.resource_address().clone()))),
            )
            .metadata(metadata! {
             roles {
                metadata_setter => rule!(require(owner_badge.resource_address().clone()));
                metadata_setter_updater => rule!(require(owner_badge.resource_address().clone()));
                metadata_locker => rule!(require(owner_badge.resource_address().clone()));
                metadata_locker_updater => rule!(require(owner_badge.resource_address().clone()));
                 },
                 init {
                     "name" => "Stake Claim Receipt".to_string(), updatable;
                     "symbol" => "SCR".to_string(), updatable;
                     "description" => "Stake Claim Receipt".to_string(), updatable;
                 }
            })
            .mint_roles(mint_roles! {
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(deny_all);
            })
            .burn_roles(burn_roles! {
                burner => rule!(allow_all);
                burner_updater => rule!(deny_all);
            })
            .withdraw_roles(withdraw_roles! {
                withdrawer => rule!(allow_all);
                withdrawer_updater => rule!(deny_all);
            })
            .deposit_roles(deposit_roles! {
                depositor => rule!(allow_all);
                depositor_updater => rule!(deny_all);
            })
            .freeze_roles(freeze_roles! {
                freezer => rule!(deny_all);
                freezer_updater => rule!(deny_all);
            })
            .recall_roles(recall_roles! {
                recaller => rule!(deny_all);
                recaller_updater => rule!(deny_all);
            })
            .non_fungible_data_update_roles(non_fungible_data_update_roles! {
                non_fungible_data_updater => rule!(require(global_caller(component_address)));
                non_fungible_data_updater_updater => rule!(deny_all);
            })
            .create_with_no_initial_supply();

            // Create staked version of the token to be used in the pool
            let stake_pool_synth_token_manager: ResourceManager = ResourceBuilder::new_fungible(
                OwnerRole::Updatable(rule!(require(owner_badge.resource_address().clone()))),
            )
            .divisibility(DIVISIBILITY_MAXIMUM)
            .metadata(metadata!(
                init {
                    "name" => stake_pool_synth_name.to_string(), updatable;
                    "symbol" => stake_pool_synth_token_symbol.to_string(), updatable;
                    "description" => stake_pool_synth_description.to_string(), updatable;
            }))
            // The below roles will be reset to only allow the component and the linked pool
            .withdraw_roles(withdraw_roles! {
              withdrawer => rule!(allow_all);
              withdrawer_updater => rule!(allow_all);
            })
            .deposit_roles(
                // The below roles will be reset to only allow the component and the linked pool
                deposit_roles! {
                  depositor => rule!(allow_all);
                  depositor_updater => rule!(allow_all);
                },
            )
            .mint_roles(mint_roles! {
                minter => rule!(require(global_component_caller_badge.resource_address().clone()));
                minter_updater => rule!(deny_all);
            })
            .burn_roles(burn_roles! {
              burner => rule!(require(global_component_caller_badge.resource_address().clone()));
              burner_updater => rule!(deny_all);
            })
            .create_with_no_initial_supply();

            // Instantiate the OneResourcePool blueprint
            let stake_pool_synth = Blueprint::<OneResourcePool>::instantiate(
                owner_role.clone(),
                rule!(require(global_component_caller_badge.clone())),
                stake_pool_synth_token_manager.address(),
                None,
            );

            // Instantiate the native account blueprint and define the dApp definition
            let dapp_definition_account =
                Blueprint::<Account>::create_advanced(OwnerRole::Updatable(rule!(allow_all)), None);

            let dapp_definition_address = GlobalAddress::from(dapp_definition_account.address());

            // Get the address of the pool's lp token
            let stake_pool_lp_token_address = ResourceAddress::try_from(
                stake_pool_synth
                    .get_metadata::<String, GlobalAddress>(String::from("pool_unit"))
                    .expect("No 'pool_unit' metadata field found in associated one-resource pool.")
                    .expect(
                        "Problem getting 'pool_unit' metadata for associated one-resource pool.",
                    ),
            )
            .expect("Could not convert pool token GlobalAddress to ResourceAddress.");

            // Get the resource manager for the pool's lp token
            let stake_pool_lp_token_manager = ResourceManager::from(stake_pool_lp_token_address);

            // Set the remaining permissions and metadata
            dapp_definition_account
                .set_owner_role(rule!(require(owner_badge.resource_address().clone())));

            owner_badge.authorize_with_all(|| {
                // Set the pool token name, symbol and dapp definition
                stake_pool_lp_token_manager
                    .set_metadata("name", format!("{} token", stake_pool_lp_token_name));
                stake_pool_lp_token_manager
                    .set_metadata("symbol", stake_pool_lp_token_symbol.clone());
                stake_pool_lp_token_manager
                    .set_metadata("dapp_definitions", vec![dapp_definition_address.clone()]);
                // Only allow the component and the pool to deposit and withdraw the synthetic token
                stake_pool_synth_token_manager.set_depositable(rule!(require_any_of(vec![
                    global_caller(component_address),
                    global_caller(stake_pool_synth.address())
                ])));
                stake_pool_synth_token_manager.lock_depositable();
                stake_pool_synth_token_manager.set_withdrawable(rule!(require_any_of(vec![
                    global_caller(component_address),
                    global_caller(stake_pool_synth.address())
                ])));
                stake_pool_synth_token_manager.lock_withdrawable();
            });

            let component = Self {
                stake_vault_actual: FungibleVault::new(stake_token_actual),
                stake_token_actual,
                stake_vault_lp_token: Vault::new(stake_pool_lp_token_manager.address()),
                unstake_period,
                nft_claim_receipt_resource_manager: nft_claim_receipt,
                dapp_definition_account,
                dapp_definition_address,
                stake_pool_synth,
                stake_pool_synth_token_manager,
                stake_pool_synth_name,
                stake_pool_synth_token_symbol,
                stake_pool_synth_description,
                stake_pool_lp_token_name,
                stake_pool_lp_token_manager,
                stake_pool_lp_token_symbol,
                owner_badge: owner_badge.resource_address(),
                contract_status: Status::On,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge.resource_address().clone()))))
            .with_address(address_reservation)
            .metadata(metadata! {
             roles {
                    metadata_setter => rule!(require(owner_badge.resource_address().clone()));
                    metadata_setter_updater => rule!(require(owner_badge.resource_address().clone()));
                    metadata_locker => rule!(require(owner_badge.resource_address().clone()));
                    metadata_locker_updater => rule!(require(owner_badge.resource_address().clone()));
                },
                 init {
                    "name" => "Staking Contract".to_string(), updatable;
                   "description" => "A contract that allows users to stake tokens.".to_string(), updatable;
                }
             })
            .globalize();

            return (component, owner_badge);
        }

        pub fn stake(&mut self, stake_tokens_actual: FungibleBucket) -> Bucket {
            // Check if the contract is active
            assert!(
                self.contract_status == Status::On,
                "Contract is not active."
            );

            // Check if the stake amount is greater than zero
            assert!(
                stake_tokens_actual.amount() > Decimal::from(0),
                "Stake amount must be greater than zero."
            );

            // Check if the staking token matches
            assert!(
                stake_tokens_actual.resource_address() == self.stake_token_actual,
                "Invalid staking token."
            );

            // Put the real stake in the vault
            let deposit_amount = stake_tokens_actual.amount();
            self.stake_vault_actual.put(stake_tokens_actual);
            info!("Actual token vault: {:?}", self.stake_vault_actual);

            // Mint an equal amount of synthetic tokens
            let minted_tokens = self.stake_pool_synth_token_manager.mint(deposit_amount);
            info!("Minted synthetic tokens: {:?}", minted_tokens);

            // Stake the synthetic tokens and receive the pool units; returning them to the user
            let tokens = self.stake_pool_synth.contribute(minted_tokens);
            info!("Minted pool units: {:?}", tokens);

            return tokens;
        }

        pub fn unstake(&mut self, pool_units: Bucket) -> NonFungibleBucket {
            // Check if the contract is active
            assert!(
                self.contract_status == Status::On,
                "Contract is not active."
            );

            // Check if amount is greater than zero
            assert!(
                pool_units.resource_address() == self.stake_pool_lp_token_manager.address(),
                "Invalid pool units."
            );

            // Check there is at least one pool unit
            assert!(
                pool_units.amount().clone() > Decimal::from(0),
                "Pool units must be greater than zero."
            );

            // Get the redemption value of the pool units
            let redemption_value = self
                .stake_pool_synth
                .get_redemption_value(pool_units.amount().clone());
            info!("Redemption value: {:?}", redemption_value);

            // Get the current epoch
            let current_epoch = Runtime::current_epoch().number();

            // Set the unstake period end
            let unstake_period_end = current_epoch.checked_add(self.unstake_period).unwrap();

            // Set up clone
            let pool_units_amount_clone = pool_units.amount().clone();

            // Mint and receive the claim nft
            let nft_claim_receipt_data = NFTClaimReceiptData {
                stake_token_actual: self.stake_token_actual.clone(),
                stake_pool_synth_token: self.stake_pool_synth_token_manager.address().clone(),
                unstake_period_end: unstake_period_end.clone(),
                pool_units: pool_units_amount_clone.clone(),
                pending_unstake_amount: redemption_value.clone(),
            };
            info!("NFT claim receipt data: {:?}", nft_claim_receipt_data);
            let nft_claim_receipt: NonFungibleBucket = self
                .nft_claim_receipt_resource_manager
                .mint_ruid_non_fungible(nft_claim_receipt_data)
                .as_non_fungible();

            // Store pool units inside the component
            self.stake_vault_lp_token.put(pool_units);
            info!("Pool units vault: {:?}", self.stake_vault_lp_token);

            // Return the NFT receipt
            return nft_claim_receipt;
        }

        pub fn withdraw_stake(&mut self, nft_claim_receipt: NonFungibleBucket) -> FungibleBucket {
            // Check if the contract is active
            assert!(
                self.contract_status == Status::On,
                "Contract is not active."
            );

            // Check the nft claim receipt
            assert!(
                nft_claim_receipt.resource_address()
                    == self.nft_claim_receipt_resource_manager.address(),
                "Invalid NFT claim receipt."
            );

            // Check it is only 1 nft claim receipt
            assert!(
                nft_claim_receipt.amount() == Decimal::from(1),
                "Only 1 NFT claim receipt is allowed."
            );

            // Retrieve the NFT id and data
            let nft_id: NonFungibleLocalId = nft_claim_receipt.non_fungible_local_id();
            let nft_claim_receipt_data: NFTClaimReceiptData = self
                .nft_claim_receipt_resource_manager
                .get_non_fungible_data(&nft_id);
            info!("NFT claim receipt data: {:?}", nft_claim_receipt_data);

            // Check if the unstake period has ended
            assert!(
                Runtime::current_epoch().number() >= nft_claim_receipt_data.unstake_period_end,
                "Unstake period has not ended yet."
            );

            // Check if the unstake period is greater than zero
            assert!(
                nft_claim_receipt_data.unstake_period_end > 0,
                "Unstake period must be greater than zero."
            );

            // Retrieve the amount of pool units written inside the NFT claim receipt from the component vault
            let take_pool_units = self
                .stake_vault_lp_token
                .take(nft_claim_receipt_data.pool_units);
            info!("Pool units: {:?}", take_pool_units);

            // Redeem the pool units in return for the synthetic staking tokens
            let synth_tokens = self.stake_pool_synth.redeem(take_pool_units);
            info!("Synthetic tokens: {:?}", synth_tokens);

            // Withdraw the same amount of actual staking tokens from the vault
            let withdrawn_tokens = self.stake_vault_actual.take(synth_tokens.amount());
            info!("Withdrawn actual tokens: {:?}", withdrawn_tokens);

            // Burn the synthetic staking tokens
            info!(
                "Burned synthetic tokens: {:?}",
                synth_tokens.amount().clone()
            );
            self.stake_pool_synth_token_manager.burn(synth_tokens);

            // Return the actual tokens to the user
            return withdrawn_tokens;
        }

        pub fn show_redemption_value(&mut self, pool_units: Decimal) {
            // Check if amount is greater than zero
            assert!(
                pool_units > Decimal::from(0),
                "Pool units must be greater than zero."
            );

            // Get the redemption value of the pool units
            info!(
                "Redeeming {:?} pool units will return {:?} tokens.",
                pool_units,
                self.stake_pool_synth.get_redemption_value(pool_units)
            );
        }

        pub fn show_vault_amount(&mut self) {
            // Get the amount of tokens in the vault
            info!(
                "There are currently {:?} tokens in the vault.",
                self.stake_pool_synth.get_vault_amount()
            );
        }

        pub fn check_unstake_status(&self, nft_claim_receipt_proof: NonFungibleProof) {
            // Check if the receipt is valid
            assert!(
                nft_claim_receipt_proof.resource_address()
                    == self.nft_claim_receipt_resource_manager.address(),
                "Invalid NFT claim receipt."
            );
            // Check proof
            let checked_proof = nft_claim_receipt_proof.check_with_message(
                self.nft_claim_receipt_resource_manager.address(),
                "Invalid NFT claim receipt proof.",
            );
            // Retrieve the NFT id and data
            let nft_id: NonFungibleLocalId = checked_proof.non_fungible_local_id();
            let nft_claim_receipt_data: NFTClaimReceiptData = self
                .nft_claim_receipt_resource_manager
                .get_non_fungible_data(&nft_id);
            info!("NFT claim receipt data: {:?}", nft_claim_receipt_data);

            // Calculate the number of epochs left
            let current_epoch = Runtime::current_epoch().number();

            // Calculate the epochs left
            let epochs_left = nft_claim_receipt_data
                .unstake_period_end
                .checked_sub(current_epoch)
                .unwrap_or(0);

            // Calculate the time left in hours or days
            let minutes_left: u64 = epochs_left.clone().checked_mul(5).unwrap_or(0); // Each epoch lasts 5 minutes
            let hours_left = minutes_left.checked_div(60).unwrap_or(0);
            let days_left = hours_left.checked_div(24).unwrap_or(0);

            // Display the remaining time
            if epochs_left.clone() > 0 {
                info!(
                    "There are {} epochs left until you can withdraw your stake.",
                    epochs_left
                );
                if days_left >= 3 {
                    info!("This is approximately {} days.", days_left);
                } else if hours_left < 72 && hours_left >= 1 {
                    info!("This is approximately {} hours.", hours_left);
                } else {
                    info!("This is approximately {} minutes.", minutes_left);
                }
            } else {
                info!("You can now withdraw your stake.");
            }
        }

        pub fn deposit(&mut self, stake_tokens_actual: FungibleBucket) {
            // Check if the contract is active
            assert!(
                self.contract_status == Status::On,
                "Contract is not active."
            );

            // Check if the stake amount is greater than zero
            assert!(
                stake_tokens_actual.amount() > Decimal::from(0),
                "Stake amount must be greater than zero."
            );

            // Check if the staking token matches
            assert!(
                stake_tokens_actual.resource_address() == self.stake_token_actual,
                "Invalid staking token."
            );

            // Put the real stake in the vault
            let deposit_amount = stake_tokens_actual.amount();
            self.stake_vault_actual.put(stake_tokens_actual);
            info!("Actual token vault: {:?}", self.stake_vault_actual);

            // Mint an equal amount of synthetic tokens
            let minted_tokens = self.stake_pool_synth_token_manager.mint(deposit_amount);
            info!("Minted synthetic tokens: {:?}", minted_tokens);

            // Deposit tokens into the stake pool
            self.stake_pool_synth.protected_deposit(minted_tokens);
            info!(
                "Synthetic tokens vault: {:?}",
                self.stake_pool_synth.get_vault_amount()
            );
        }

        pub fn update_unstake_period(&mut self, new_unstake_period: u64) {
            // Check if the contract is active
            assert!(self.contract_status == Status::On, "Contract is not active");

            // Check if the new unstake period is greater than zero
            assert!(
                new_unstake_period > 0,
                "Unstake period must be greater than zero."
            );

            // Check if the new unstake period is different from the current unstake period
            assert!(
                new_unstake_period != self.unstake_period,
                "Unstake period is already set to the new value."
            );

            // Update unstake period
            self.unstake_period = new_unstake_period;
            info!(
                "Unstake period has been updated to {} epochs.",
                self.unstake_period
            );
        }

        pub fn emergency_switch(&mut self) {
            // Toggle the contract status
            match self.contract_status {
                Status::On => self.contract_status = Status::Off,
                Status::Off => self.contract_status = Status::On,
            }
            info!(
                "Contract status has been switched to {:?}.",
                self.contract_status
            );
        }

        pub fn emergency_withdraw_with_pool_units(&mut self, pool_units: Bucket) -> FungibleBucket {
            // Check if the contract is in emergency mode
            assert!(
                self.contract_status == Status::Off,
                "Contract is active, cannot withdraw in 'On' mode."
            );

            // Check if amount is greater than zero
            assert!(
                pool_units.amount() > Decimal::from(0),
                "Withdraw amount must be greater than zero."
            );

            // Check if the staking token matches
            assert!(
                pool_units.resource_address() == self.stake_pool_lp_token_manager.address(),
                "Invalid pool units."
            );

            // Withdraw the actual tokens from the vault; ignoring the unstake period only

            // Redeem the pool units in return for the synthetic staking tokens
            let tokens = self.stake_pool_synth.redeem(pool_units);
            info!("Redeemed synthetic tokens: {:?}", tokens);

            // Withdraw the same amount of actual staking tokens from the vault
            let withdraw_tokens = self.stake_vault_actual.take(tokens.amount());
            info!("Withdrawn actual tokens: {:?}", withdraw_tokens);

            // Burn the synthetic staking tokens
            info!("Burned synthetic tokens: {:?}", tokens.amount().clone());
            self.stake_pool_synth_token_manager.burn(tokens);

            // Return the withdrawn tokens
            return withdraw_tokens;
        }

        pub fn emergency_withdraw_with_nft_claim_receipt(
            &mut self,
            nft_claim_receipt: NonFungibleBucket,
        ) -> FungibleBucket {
            // Check if the contract is in emergency mode
            assert!(
                self.contract_status == Status::Off,
                "Contract is active, cannot withdraw in 'On' mode."
            );

            // Check the nft claim receipt
            assert!(
                nft_claim_receipt.resource_address()
                    == self.nft_claim_receipt_resource_manager.address(),
                "Invalid NFT claim receipt."
            );

            // Check it is only 1 nft claim receipt
            assert!(
                nft_claim_receipt.amount() == Decimal::from(1),
                "Only 1 NFT claim receipt is allowed."
            );

            // Retrieve the NFT id and data
            let nft_id: NonFungibleLocalId = nft_claim_receipt.non_fungible_local_id();
            let nft_claim_receipt_data: NFTClaimReceiptData = self
                .nft_claim_receipt_resource_manager
                .get_non_fungible_data(&nft_id);
            info!("NFT claim receipt data: {:?}", nft_claim_receipt_data);

            // Withdraw the actual tokens from the vault; ignoring the unstake period only

            // Take the pool units from the component vault and redeem them for the user
            let take_pool_units = self
                .stake_vault_lp_token
                .take(nft_claim_receipt_data.pool_units);
            info!("Pool units: {:?}", take_pool_units);

            // Redeem the pool units in return for the synthetic staking tokens
            let tokens = self.stake_pool_synth.redeem(take_pool_units);
            info!("Redeemed synthetic tokens: {:?}", tokens);

            // Withdraw the same amount of actual staking tokens from the vault
            let withdraw_tokens = self.stake_vault_actual.take(tokens.amount());
            info!("Withdrawn actual tokens: {:?}", withdraw_tokens);

            // Burn the synthetic staking tokens
            info!("Burned synthetic tokens: {:?}", tokens.amount().clone());
            self.stake_pool_synth_token_manager.burn(tokens);

            // Return the withdrawn tokens
            return withdraw_tokens;
        }

        pub fn emergency_withdraw_pool_bypass_with_pool_units(
            &mut self,
            pool_units: Bucket,
        ) -> FungibleBucket {
            // Check if the contract is in emergency mode
            assert!(
                self.contract_status == Status::Off,
                "Contract is active, cannot withdraw in 'On' mode."
            );

            // Check if amount is greater than zero
            assert!(
                pool_units.amount() > Decimal::from(0),
                "Withdraw amount must be greater than zero."
            );

            // Check if the staking token matches
            assert!(
                pool_units.resource_address() == self.stake_pool_lp_token_manager.address(),
                "Invalid pool units."
            );

            // Define the redemption value
            let redemption_value = self
                .stake_pool_synth
                .get_redemption_value(pool_units.amount());
            info!("Redemption value: {:?}", redemption_value);

            // Withdraw the actual tokens from the vault; ignoring the unstake period and resource pool
            let withdraw_tokens = self.stake_vault_actual.take(redemption_value.clone());
            info!("Withdrawn actual tokens: {:?}", withdraw_tokens);

            // Burn the pool units
            info!("Burned pool units: {:?}", pool_units.amount().clone());
            self.stake_pool_lp_token_manager.burn(pool_units);

            // Return the withdrawn tokens
            return withdraw_tokens;
        }

        pub fn emergency_withdraw_pool_bypass_with_nft_claim_receipt(
            &mut self,
            nft_claim_receipt: NonFungibleBucket,
        ) -> FungibleBucket {
            // Check if the contract is in emergency mode
            assert!(
                self.contract_status == Status::Off,
                "Contract is active, cannot withdraw in 'On' mode."
            );

            // Check the nft claim receipt
            assert!(
                nft_claim_receipt.resource_address()
                    == self.nft_claim_receipt_resource_manager.address(),
                "Invalid NFT claim receipt."
            );

            // Check it is only 1 nft claim receipt
            assert!(
                nft_claim_receipt.amount() == Decimal::from(1),
                "Only 1 NFT claim receipt is allowed."
            );

            // Retrieve the NFT ID and data
            let nft_id: NonFungibleLocalId = nft_claim_receipt.non_fungible_local_id();
            let nft_claim_receipt_data: NFTClaimReceiptData = self
                .nft_claim_receipt_resource_manager
                .get_non_fungible_data(&nft_id);
            info!("NFT claim receipt data: {:?}", nft_claim_receipt_data);

            // Define the redemption value
            let redemption_value = nft_claim_receipt_data.pending_unstake_amount;
            info!("Redemption value: {:?}", redemption_value);

            // Withdraw the actual tokens from the vault; ignoring the unstake period and resource pool
            let withdraw_tokens = self.stake_vault_actual.take(redemption_value.clone());
            info!("Withdrawn actual tokens: {:?}", withdraw_tokens);

            // Burn the NFT receipt
            self.nft_claim_receipt_resource_manager
                .burn(nft_claim_receipt);
            info!("Burned NFT claim receipt: {:?}", nft_claim_receipt_data);

            // Return the withdrawn tokens
            return withdraw_tokens;
        }
    }
}
