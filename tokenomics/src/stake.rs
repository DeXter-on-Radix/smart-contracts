use scrypto::prelude::*;

#[derive(ScryptoSbor, PartialEq)]
pub enum Status {
    On,
    Off,
}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct NFTReceiptData {
    staking_token: ResourceAddress,
    #[mutable]
    amount_staked: Decimal,
    // Unstake period denominated in number of epochs
    #[mutable]
    unstake_period_end: u64,
    #[mutable]
    pending_rewards: Decimal,
}

#[blueprint]
mod stake {
    enable_method_auth! {
        roles {
            admin => updatable_by: [OWNER];
        },
        methods {
            stake => PUBLIC;
            unstake => PUBLIC;
            add_stake => PUBLIC;
            withdraw_stake => PUBLIC;
            update_unstake_period => restrict_to: [OWNER];
            emergency_switch => restrict_to: [OWNER];
            emergency_withdraw => PUBLIC;
            check_unstake_status => PUBLIC;

        }
    }

    struct Stake {
        // Define what resources and data will be managed by Stake components
        stake_vault: FungibleVault,
        staking_token: ResourceAddress,
        unstake_period: u64,
        nft_receipt_resource_manager: ResourceManager,
        owner_badge: ResourceAddress,
        admin_badge: ResourceAddress,
        contract_status: Status,
    }

    impl Stake {
        pub fn instantiate_stake(
            owner_badge: ResourceAddress,
            admin_badge: ResourceAddress,
            unstake_period: u64,
            staking_token: ResourceAddress,
        ) -> Global<Stake> {
            // Set Up Actor Virtual Badge
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Stake::blueprint_id());

            let nft_receipt = ResourceBuilder::new_ruid_non_fungible::<NFTReceiptData>(
                OwnerRole::Updatable(rule!(require(owner_badge))),
            )
            .metadata(metadata! {
             roles {
                metadata_setter => rule!(require(owner_badge) || require_amount(3, admin_badge));
                metadata_setter_updater => rule!(require(owner_badge));
                metadata_locker => rule!(require(owner_badge) || require_amount(3, admin_badge));
                metadata_locker_updater => rule!(require(owner_badge));
                 },
                 init {
                     "name" => "Stake Receipt".to_string(), updatable;
                     "symbol" => "SR".to_string(), updatable;
                     "description" => "Stake Receipt".to_string(), updatable;
                 }
            })
            .mint_roles(mint_roles! {
                minter => rule!(require(global_caller(component_address)));
                minter_updater => rule!(deny_all);
            })
            .burn_roles(burn_roles! {
                burner => rule!(require(global_caller(component_address)));
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

            let component = Self {
                stake_vault: FungibleVault::new(staking_token),
                staking_token,
                unstake_period,
                nft_receipt_resource_manager: nft_receipt,
                owner_badge,
                admin_badge,
                contract_status: Status::On,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge))))
            .with_address(address_reservation)
            .roles(
               roles!(
                admin => rule!(require(admin_badge));
              )
            )
            .metadata(metadata! {
             roles {
                    metadata_setter => rule!(require(owner_badge) || require_amount(3, admin_badge));
                    metadata_setter_updater => rule!(require(owner_badge));
                    metadata_locker => rule!(require(owner_badge) || require_amount(3, admin_badge));
                    metadata_locker_updater => rule!(require(owner_badge));
                },
                 init {
                    "name" => "Staking Contract".to_string(), updatable;
                   "description" => "A contract that allows users to stake tokens.".to_string(), updatable;
                }
             })
            .globalize();

            return component;
        }

        pub fn stake(&mut self, stake_tokens: FungibleBucket) -> NonFungibleBucket {
            // Check if the contract is active
            assert!(self.contract_status == Status::On, "Contract is not active");
            // Check if the stake amount is greater than zero
            assert!(
                stake_tokens.amount() > Decimal::from(0),
                "Stake amount must be greater than zero"
            );
            // Check if the staking token matches
            assert!(
                stake_tokens.resource_address() == self.staking_token,
                "Invalid staking token"
            );
            // Put the stake in the vault
            let deposit_amount = stake_tokens.amount();
            self.stake_vault.put(stake_tokens);
            // Mint an NFT receipt
            let nft_receipt_data = NFTReceiptData {
                staking_token: self.staking_token,
                amount_staked: deposit_amount,
                unstake_period_end: 0,
                pending_rewards: Decimal::from(0),
            };
            let nft_receipt: NonFungibleBucket = self
                .nft_receipt_resource_manager
                .mint_ruid_non_fungible(nft_receipt_data)
                .as_non_fungible();
            // Return the NFT receipt
            return nft_receipt;
        }

        pub fn add_stake(
            &mut self,
            receipt: NonFungibleBucket,
            additional_stake_tokens: FungibleBucket,
        ) -> NonFungibleBucket {
            // Check if the contract is active
            assert!(self.contract_status == Status::On, "Contract is not active");
            // Check if the additional stake amount is greater than zero
            assert!(
                additional_stake_tokens.amount() > Decimal::from(0),
                "Additional stake amount must be greater than zero"
            );
            // Check if the staking token matches
            assert!(
                additional_stake_tokens.resource_address() == self.staking_token,
                "Invalid staking token"
            );
            // Check if the receipt is valid
            assert!(
                receipt.resource_address() == self.nft_receipt_resource_manager.address(),
                "Invalid NFT receipt"
            );
            assert!(
                receipt.amount() == Decimal::from(1),
                "Only one NFT receipt can be deposited at a time"
            );
            // Retrieve the NFT ID and data
            let nft_id: NonFungibleLocalId = receipt.non_fungible_local_id();
            let nft_receipt_data: NFTReceiptData = self
                .nft_receipt_resource_manager
                .get_non_fungible_data(&nft_id);
            // Update the staked amount
            let additional_stake_amount = additional_stake_tokens.amount();
            let new_stake_amount = nft_receipt_data.amount_staked + additional_stake_amount;
            // Put the additional stake in the vault
            self.stake_vault.put(additional_stake_tokens);
            // Update the NFT data
            self.nft_receipt_resource_manager.update_non_fungible_data(
                &nft_id,
                &"amount_staked",
                new_stake_amount,
            );
            // Return the updated receipt
            return receipt;
        }

        pub fn unstake(
            &mut self,
            amount: Decimal,
            receipt: NonFungibleBucket,
        ) -> NonFungibleBucket {
            // Check if the contract is active
            assert!(self.contract_status == Status::On, "Contract is not active");
            // Check if amount is greater than zero
            assert!(
                amount > Decimal::from(0),
                "Amount must be greater than zero"
            );
            // Check if the receipt is valid
            assert!(
                receipt.resource_address() == self.nft_receipt_resource_manager.address(),
                "Invalid NFT receipt"
            );
            assert!(
                receipt.amount() == Decimal::from(1),
                "Only one NFT receipt can be deposited at a time"
            );

            // Retrieve the NFT ID and data
            let nft_id: NonFungibleLocalId = receipt.non_fungible_local_id();
            let nft_receipt_data: NFTReceiptData = self
                .nft_receipt_resource_manager
                .get_non_fungible_data(&nft_id);
            // Check if amount is valid
            assert!(amount <= nft_receipt_data.amount_staked, "Invalid amount");
            // Check that unstake period is 0
            assert!(
                nft_receipt_data.unstake_period_end == 0,
                "Unstake period must be zero"
            );
            // Update the NFT data with the unstake_period_end
            let new_unstake_period_end = Runtime::current_epoch().number() + self.unstake_period;
            self.nft_receipt_resource_manager.update_non_fungible_data(
                &nft_id,
                &"unstake_period_end",
                new_unstake_period_end,
            );
            // Check that pending rewards are zero
            assert!(
                nft_receipt_data.pending_rewards == Decimal::from(0),
                "Pending rewards must be zero"
            );
            // Update the NFT data with pending rewards
            let new_pending_rewards = nft_receipt_data.pending_rewards + amount;
            self.nft_receipt_resource_manager.update_non_fungible_data(
                &nft_id,
                &"pending_rewards",
                new_pending_rewards,
            );
            // Update the NFT data with the new staked amount
            let new_stake_amount = nft_receipt_data.amount_staked - amount;
            self.nft_receipt_resource_manager.update_non_fungible_data(
                &nft_id,
                &"amount_staked",
                new_stake_amount,
            );
            // Return the nft receipt
            return receipt;
        }

        pub fn withdraw_stake(
            &mut self,
            receipt: NonFungibleBucket,
        ) -> (Option<NonFungibleBucket>, Option<FungibleBucket>) {
            // Check if the contract is active
            assert!(self.contract_status == Status::On, "Contract is not active");
            // Check if the receipt is valid
            assert!(
                receipt.resource_address() == self.nft_receipt_resource_manager.address(),
                "Invalid NFT receipt"
            );
            assert!(
                receipt.amount() == Decimal::from(1),
                "Only one NFT receipt can be deposited at a time"
            );
            // Retrieve the NFT ID and data
            let nft_id: NonFungibleLocalId = receipt.non_fungible_local_id();
            let mut nft_receipt_data: NFTReceiptData = self
                .nft_receipt_resource_manager
                .get_non_fungible_data(&nft_id);
            // Check if the unstake period has ended
            assert!(
                Runtime::current_epoch().number() >= nft_receipt_data.unstake_period_end,
                "Unstake period has not ended yet"
            );
            // Check if the unstake period is greater than zero
            assert!(
                nft_receipt_data.unstake_period_end > 0,
                "Unstake period must be greater than zero"
            );
            // Withdraw the staked amount
            let withdraw_amount = nft_receipt_data.pending_rewards;
            let withdraw_tokens: FungibleBucket = self.stake_vault.take(withdraw_amount);

            // Check the staked amount and take appropriate action
            if nft_receipt_data.amount_staked.is_zero() {
                // Burn the NFT receipt as there are no more staked tokens
                self.nft_receipt_resource_manager.burn(receipt);

                return (None, Some(withdraw_tokens));
            } else {
                // Update the NFT data for any remaining staked amount
                let reset_unstake_peirod_end = nft_receipt_data.unstake_period_end = 0;
                let reset_pending_rewards = nft_receipt_data.pending_rewards = Decimal::ZERO;
                self.nft_receipt_resource_manager.update_non_fungible_data(
                    &nft_id,
                    &"unstake_period_end",
                    &reset_unstake_peirod_end,
                );
                self.nft_receipt_resource_manager.update_non_fungible_data(
                    &nft_id,
                    &"pending_rewards",
                    &reset_pending_rewards,
                );

                return (Some(receipt), Some(withdraw_tokens));
            }
        }

        pub fn check_unstake_status(&self, receipt_proof: NonFungibleProof) {
            // Check if the contract is active
            assert!(self.contract_status == Status::On, "Contract is not active");
            // Check if the receipt is valid
            assert!(
                receipt_proof.resource_address() == self.nft_receipt_resource_manager.address(),
                "Invalid NFT Resource Address"
            );
            // Check proof
            let checked_proof = receipt_proof.check_with_message(
                self.nft_receipt_resource_manager.address(),
                "Invalid NFT Proof",
            );
            // Retrieve the NFT ID and data
            let nft_id: NonFungibleLocalId = checked_proof.non_fungible_local_id();
            let nft_receipt_data: NFTReceiptData = self
                .nft_receipt_resource_manager
                .get_non_fungible_data(&nft_id);
            // Calculate the number of epochs left
            let current_epoch = Runtime::current_epoch().number();
            let epochs_left = if nft_receipt_data.unstake_period_end > current_epoch {
                nft_receipt_data.unstake_period_end - current_epoch
            } else {
                0
            };
            // Calculate the time left in hours or days
            let minutes_left = epochs_left * 5; // Each epoch lasts 5 minutes
            let hours_left = minutes_left / 60;
            let days_left = hours_left / 24;
            // Display the remaining time
            info!(
                "There are {} epochs left until you can unstake.",
                epochs_left
            );
            if days_left >= 3 {
                info!("This is approximately {} days.", days_left);
            } else if hours_left < 72 && hours_left >= 1 {
                info!("This is approximately {} hours.", hours_left);
            } else {
                info!("This is approximately {} minutes.", minutes_left);
            }
        }

        pub fn update_unstake_period(&mut self, new_unstake_period: u64) {
            // Check if the contract is active
            assert!(self.contract_status == Status::On, "Contract is not active");
            // Check if the new unstake period is greater than zero
            assert!(
                new_unstake_period > 0,
                "Unstake period must be greater than zero"
            );
            // Check if the new unstake period is different from the current unstake period
            assert!(
                new_unstake_period != self.unstake_period,
                "Unstake period is already set to the new value"
            );
            // Update unstake period
            self.unstake_period = new_unstake_period;
        }

        pub fn emergency_switch(&mut self) {
            match self.contract_status {
                Status::On => self.contract_status = Status::Off,
                Status::Off => self.contract_status = Status::On,
            }
        }
        pub fn emergency_withdraw(&mut self, receipt: NonFungibleBucket) -> FungibleBucket {
            // Check if the contract is in emergency mode
            assert!(
                self.contract_status == Status::Off,
                "Contract is active, cannot withdraw in 'On' mode"
            );
            // Check if the receipt is valid
            assert!(
                receipt.resource_address() == self.nft_receipt_resource_manager.address(),
                "Invalid NFT receipt"
            );
            assert!(
                receipt.amount() == Decimal::from(1),
                "Only one NFT receipt can be deposited at a time"
            );
            // Retrieve the NFT ID and data
            let nft_id: NonFungibleLocalId = receipt.non_fungible_local_id();
            let nft_receipt_data: NFTReceiptData = self
                .nft_receipt_resource_manager
                .get_non_fungible_data(&nft_id);
            // Withdraw the staked amount and pending rewards
            let withdraw_tokens = self
                .stake_vault
                .take(nft_receipt_data.amount_staked + nft_receipt_data.pending_rewards);
            // Burn the NFT receipt
            self.nft_receipt_resource_manager.burn(receipt);
            // Return the withdrawn tokens
            return withdraw_tokens;
        }
    }
}
