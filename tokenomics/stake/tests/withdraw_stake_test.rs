use radix_engine_interface::prelude::FungibleBucket;
use radix_engine_interface::prelude::*;
use scrypto::*;
use scrypto_test::prelude::*;
use tokenomics_working_file3::dexter_stake::test_bindings::*;
use tokenomics_working_file3::dexter_stake::Status;

#[test]
fn dexter_stake_withdraw_stake_contract_on_pool_on_test() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();
    let package_address = Package::compile_and_publish(this_package!(), &mut env)?;

    // Mint tokens
    let owner_badge = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(1, &mut env)?;
    let super_admin_badge = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(1, &mut env)?;
    let admin_badge = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(1, &mut env)?;
    let xrd_bucket = env
        .call_method_typed::<_, _, Bucket>(FAUCET, "free", &())
        .unwrap();

    // Define the tokens used
    let super_admin_address = NativeBucket::resource_address(&super_admin_badge, &mut env)?;
    let admin_address = NativeBucket::resource_address(&admin_badge, &mut env)?;
    let actual_token_address_xrd = NativeBucket::resource_address(&xrd_bucket, &mut env)?;

    let dexter_stake = DeXterStake::instantiate_stake(
        "contract name".to_string(),
        "contract description".to_string(),
        vec!["tag1".to_string(), "tag2".to_string()],
        "dapp definition account name".to_string(),
        "dapp definition account description".to_string(),
        "https://dexteronradix.com/".to_string(),
        actual_token_address_xrd,
        7,
        "nft claim receipt name".to_string(),
        "nft claim receipt symbol".to_string(),
        "nft claim receipt description".to_string(),
        "stake pool synth name".to_string(),
        "stake pool synth symbol".to_string(),
        "stake pool synth description".to_string(),
        "stake pool lp token name".to_string(),
        "stake pool lp token symbol".to_string(),
        owner_badge,
        super_admin_address,
        admin_address,
        package_address,
        &mut env,
    )?;

    // Disable auth model
    env.disable_auth_module();

    // Method Testing
    let (dexter_stake, _bucket) = dexter_stake;
    let mut dexter_stake_clone = dexter_stake.clone();

    // Stake
    // Take 1000 tokens from the bucket
    let xrd_bucket_stake: Bucket = xrd_bucket.take(dec!(1000), &mut env).unwrap();
    // Turn xrd bucket into a fungible bucket
    let xrd_bucket_stake: FungibleBucket = FungibleBucket(xrd_bucket_stake);

    // Act
    // Define the pool units
    let pool_units = dexter_stake_clone.stake(xrd_bucket_stake, &mut env)?;

    // Unstake
    // Act
    // Define the nft claim receipt
    // Define the nft claim receipt
    let nft_claim_receipt = dexter_stake_clone.unstake(pool_units, &mut env)?;

    // Withdraw Stake
    // Set the epoch to 15
    let set_epoch_to = Epoch::of(15);
    env.set_current_epoch(set_epoch_to);

    // Contract On, Pool On

    // Act
    // Define the xrd bucket
    let xrd_bucket = dexter_stake_clone.withdraw_stake(None, Some(nft_claim_receipt), &mut env)?;
    // Turn the xrd bucket into a regular bucket
    let xrd_bucket_regular = Bucket::from(xrd_bucket);

    // Assert
    assert_eq!(
        NativeBucket::amount(&xrd_bucket_regular, &mut env)?,
        dec!("1000")
    );
    println!(
        "Tokens Withdrawn: {:?} (Contract On, Pool On)",
        xrd_bucket_regular.amount(&mut env)?
    );

    Ok(())
}

#[test]
fn dexter_stake_withdraw_stake_contract_off_pool_on_test() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();
    let package_address = Package::compile_and_publish(this_package!(), &mut env)?;

    // Mint tokens
    let owner_badge = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(1, &mut env)?;
    let super_admin_badge = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(1, &mut env)?;
    let admin_badge = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(1, &mut env)?;
    let xrd_bucket = env
        .call_method_typed::<_, _, Bucket>(FAUCET, "free", &())
        .unwrap();

    // Define the tokens used
    let super_admin_address = NativeBucket::resource_address(&super_admin_badge, &mut env)?;
    let admin_address = NativeBucket::resource_address(&admin_badge, &mut env)?;
    let actual_token_address_xrd = NativeBucket::resource_address(&xrd_bucket, &mut env)?;

    let dexter_stake = DeXterStake::instantiate_stake(
        "contract name".to_string(),
        "contract description".to_string(),
        vec!["tag1".to_string(), "tag2".to_string()],
        "dapp definition account name".to_string(),
        "dapp definition account description".to_string(),
        "https://dexteronradix.com/".to_string(),
        actual_token_address_xrd,
        7,
        "nft claim receipt name".to_string(),
        "nft claim receipt symbol".to_string(),
        "nft claim receipt description".to_string(),
        "stake pool synth name".to_string(),
        "stake pool synth symbol".to_string(),
        "stake pool synth description".to_string(),
        "stake pool lp token name".to_string(),
        "stake pool lp token symbol".to_string(),
        owner_badge,
        super_admin_address,
        admin_address,
        package_address,
        &mut env,
    )?;

    // Disable auth model
    env.disable_auth_module();

    // Method Testing
    let (dexter_stake, _bucket) = dexter_stake;
    let mut dexter_stake_clone = dexter_stake.clone();

    // Stake
    // Take 1000 tokens from the bucket
    let xrd_bucket_stake1: Bucket = xrd_bucket.take(dec!(1000), &mut env).unwrap();
    // Take 1000 tokens from the bucket
    let xrd_bucket_stake2: Bucket = xrd_bucket.take(dec!(1000), &mut env).unwrap();
    // Turn xrd bucket into a fungible bucket
    let xrd_bucket_stake1: FungibleBucket = FungibleBucket(xrd_bucket_stake1);
    // Turn xrd bucket into a fungible bucket
    let xrd_bucket_stake2: FungibleBucket = FungibleBucket(xrd_bucket_stake2);

    // Act
    // Define the pool units
    let pool_units1 = dexter_stake_clone.stake(xrd_bucket_stake1, &mut env)?;
    // Define the pool units in a second instance
    let pool_units2 = dexter_stake_clone.stake(xrd_bucket_stake2, &mut env)?;

    // Unstake
    // Act
    // Define the nft claim receipt
    let nft_claim_receipt = dexter_stake_clone.unstake(pool_units1, &mut env)?;

    // Withdraw Stake
    // Set the epoch to 15
    let set_epoch_to = Epoch::of(15);
    env.set_current_epoch(set_epoch_to);

    // Contract Off, Pool On
    // Turning the contract off
    dexter_stake_clone.emergency_switch(Some(Status::Off), None, &mut env)?;

    // Act
    // Nft claim receipt
    // Define the xrd bucket
    let xrd_bucket1 = dexter_stake_clone.withdraw_stake(None, Some(nft_claim_receipt), &mut env)?;
    // Turn the xrd bucket into a regular bucket
    let xrd_bucket_regular1 = Bucket::from(xrd_bucket1);

    // Assert
    assert_eq!(
        NativeBucket::amount(&xrd_bucket_regular1, &mut env)?,
        dec!("1000")
    );
    println!(
        "Tokens Withdrawn: {:?} (Contract Off, Pool On)",
        xrd_bucket_regular1.amount(&mut env)?
    );

    // Pool units
    // Define the xrd bucket
    let xrd_bucket2 = dexter_stake_clone.withdraw_stake(Some(pool_units2), None, &mut env)?;
    // Turn the xrd bucket into a regular bucket
    let xrd_bucket_regular2 = Bucket::from(xrd_bucket2);

    // Assert
    assert_eq!(
        NativeBucket::amount(&xrd_bucket_regular2, &mut env)?,
        dec!("1000")
    );
    println!(
        "Tokens Withdrawn: {:?} (Contract Off, Pool On)",
        xrd_bucket_regular2.amount(&mut env)?
    );

    // Resetting the contract status
    dexter_stake_clone.emergency_switch(Some(Status::On), None, &mut env)?;

    Ok(())
}

#[test]
fn dexter_stake_withdraw_stake_contract_off_pool_off_test() -> Result<(), RuntimeError> {
    // Arrange
    let mut env = TestEnvironment::new();
    let package_address = Package::compile_and_publish(this_package!(), &mut env)?;

    // Mint tokens
    let owner_badge = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(1, &mut env)?;
    let super_admin_badge = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(1, &mut env)?;
    let admin_badge = ResourceBuilder::new_fungible(OwnerRole::None)
        .divisibility(18)
        .mint_initial_supply(1, &mut env)?;
    let xrd_bucket = env
        .call_method_typed::<_, _, Bucket>(FAUCET, "free", &())
        .unwrap();

    // Define the tokens used
    let super_admin_address = NativeBucket::resource_address(&super_admin_badge, &mut env)?;
    let admin_address = NativeBucket::resource_address(&admin_badge, &mut env)?;
    let actual_token_address_xrd = NativeBucket::resource_address(&xrd_bucket, &mut env)?;

    let dexter_stake = DeXterStake::instantiate_stake(
        "contract name".to_string(),
        "contract description".to_string(),
        vec!["tag1".to_string(), "tag2".to_string()],
        "dapp definition account name".to_string(),
        "dapp definition account description".to_string(),
        "https://dexteronradix.com/".to_string(),
        actual_token_address_xrd,
        7,
        "nft claim receipt name".to_string(),
        "nft claim receipt symbol".to_string(),
        "nft claim receipt description".to_string(),
        "stake pool synth name".to_string(),
        "stake pool synth symbol".to_string(),
        "stake pool synth description".to_string(),
        "stake pool lp token name".to_string(),
        "stake pool lp token symbol".to_string(),
        owner_badge,
        super_admin_address,
        admin_address,
        package_address,
        &mut env,
    )?;

    // Disable auth model
    env.disable_auth_module();

    // Method Testing
    let (dexter_stake, _bucket) = dexter_stake;
    let mut dexter_stake_clone = dexter_stake.clone();

    // Stake
    // Take 1000 tokens from the bucket
    let xrd_bucket_stake1: Bucket = xrd_bucket.take(dec!(1000), &mut env).unwrap();
    // Take 1000 tokens from the bucket
    let xrd_bucket_stake2: Bucket = xrd_bucket.take(dec!(1000), &mut env).unwrap();
    // Turn xrd bucket into a fungible bucket
    let xrd_bucket_stake1: FungibleBucket = FungibleBucket(xrd_bucket_stake1);
    // Turn xrd bucket into a fungible bucket
    let xrd_bucket_stake2: FungibleBucket = FungibleBucket(xrd_bucket_stake2);

    // Act
    // Define the pool units
    let pool_units1 = dexter_stake_clone.stake(xrd_bucket_stake1, &mut env)?;
    // Define the pool units in a second instance
    let pool_units2 = dexter_stake_clone.stake(xrd_bucket_stake2, &mut env)?;

    // Unstake

    // Act
    // Define the nft claim receipt
    let nft_claim_receipt = dexter_stake_clone.unstake(pool_units1, &mut env)?;

    // Withdraw Stake
    // Set the epoch to 15
    let set_epoch_to = Epoch::of(15);
    env.set_current_epoch(set_epoch_to);

    // Contract Off, Pool Off
    // Turning the contract and pool off
    dexter_stake_clone.emergency_switch(Some(Status::Off), Some(Status::Off), &mut env)?;

    // Act
    // Nft claim receipt
    // Define the xrd bucket
    let xrd_bucket1 = dexter_stake_clone.withdraw_stake(None, Some(nft_claim_receipt), &mut env)?;
    // Turn the xrd bucket into a regular bucket
    let xrd_bucket_regular1 = Bucket::from(xrd_bucket1);

    // Assert
    assert_eq!(
        NativeBucket::amount(&xrd_bucket_regular1, &mut env)?,
        dec!("1000")
    );
    println!(
        "Tokens Withdrawn: {:?} (Contract Off, Pool Off)",
        xrd_bucket_regular1.amount(&mut env)?
    );

    // Pool units
    // Define the xrd bucket
    let xrd_bucket2 = dexter_stake_clone.withdraw_stake(Some(pool_units2), None, &mut env)?;
    // Turn the xrd bucket into a regular bucket
    let xrd_bucket_regular2 = Bucket::from(xrd_bucket2);

    // Assert
    assert_eq!(
        NativeBucket::amount(&xrd_bucket_regular2, &mut env)?,
        dec!("1000")
    );
    println!(
        "Tokens Withdrawn: {:?} (Contract Off, Pool Off)",
        xrd_bucket_regular2.amount(&mut env)?
    );

    // Resetting the contract status
    dexter_stake_clone.emergency_switch(Some(Status::On), Some(Status::On), &mut env)?;
    Ok(())
}
