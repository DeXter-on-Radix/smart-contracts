# DeXter On Radix Tokenomics - Staking

These are the staking smart contracts for DeXter On Radix, a decentralized exchange on the [Radix network](https://github.com/radixdlt) powered by [AlphaDEX](https://alphadex.net/). Check out our [linkr.ee](https://linktr.ee/dexteronradix).

## Getting Started

Regarding the initial setup, all instructions may be found [here](https://docs.radixdlt.com/docs/getting-rust-scrypto).

To run the blueprints locally using resim:

```bash
resim reset
resim new-account
resim publish .
resim call-function <PACKAGE_ADDRESS> Stake instantiate_stake <CONTRACT_NAME> <CONTRACT_DESCRIPTION> <CONTRACT_TAGS> <DAPP_DEFINITION_ACCOUNT_NAME> <DAPP_DEFINITION_ACCOUNT_DESCRIPTION> <DAPP_DEFINITION_ACCOUNT_ICON_URL> <STAKE_TOKEN_ACTUAL_ADDRESS> <UNSTAKE_PERIOD> <NFT_CLAIM_RECEIPT_NAME> <NFT_CLAIM_RECEIPT_SYMBOL> <NFT_CLAIM_RECEIPT_DESCRIPTION> <STAKE_POOL_SYNTHETIC_NAME> <STAKE_POOL_SYNTHETIC_TOKEN_SYMBOL> <STAKE_POOL_SYNTHETIC_DESCRIPTION> <STAKE_POOL_LP_TOKEN_NAME> <STAKE_POOL_LP_TOKEN_DESCRIPTION> <OWNER_BADGE> <SUPER_ADMIN_BADGE_ADDRESS> <ADMIN_BADGE_ADDRESS>
resim run manifest/${manifest_name}.rtm
```

## Contributing

We welcome contributions to this project. Please read our [contributing guidelines](../unit-tests/CONTRIBUTING.md) before submitting a pull request.

## Disclaimer

This code is provided "as-is" for educational and community engagement purposes. It has not undergone any formal auditing and may contain vulnerabilities or bugs. Users should exercise due diligence and caution. Usage of this code is solely at the user's own risk, including but not limited to ensuring compliance with applicable laws and regulations.

The creators and contributors of this project expressly disclaim any liability for misuse or damages that may arise from the use of this code. Please be aware that the code may be incomplete and is subject to future modifications, particularly when intended for use in production environments or in high-risk applications.

For more information about your rights and obligations when using or contributing to this project, please refer to the Apache 2.0 license located [here](../unit-tests/LICENSE).