use radix_engine_interface::prelude::FungibleBucket;
use radix_engine_interface::prelude::*;
use scrypto::*;
use scrypto_test::prelude::*;
use tokenomics_working_file3::dexter_stake::test_bindings::*;

#[test]
fn dexter_stake_stake_test() -> Result<(), RuntimeError> {
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

    // Assert
    assert_eq!(NativeBucket::amount(&pool_units, &mut env)?, dec!("1000"));
    println!("Pool Units: {:?}", pool_units.amount(&mut env)?);

    Ok(())
}
