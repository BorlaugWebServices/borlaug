use runtime::{
    constants::currency::*,
    primitives::{AccountId, Balance, Signature},
    wasm_binary_unwrap, BalancesConfig, Block, CouncilConfig, GenesisConfig, SettingsConfig,
    SudoConfig, SystemConfig,
};
use sc_chain_spec::ChainSpecExtension;
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sp_core::{crypto::Ss58Codec, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::marker::PhantomData;

type AccountPublic = <Signature as Verify>::Signer;

static ORG_ADMIN: &str = "5EZ7gNcZidoanKK45JK4YVQNDpEScbcCNbV4BU7fJWJdAFsu";
static ATTESTER_1: &str = "5GnGy76zKS2Yy77vvvwyhzBDxPku6yZ3Y8cBB9eiZKpQ7rUW";
static ATTESTER_2: &str = "5HDfARwo5GGHTr7E7vDuwKkJKt31xoJUCUFWRdzkifDQW5HK";

static COUNCIL: &str = "5CD9YDBg4nohwKQJ3CzwjZZsy7yka3srB6oxmuHn9rNJPx68";

// Note this is the URL for the telemetry server
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// Block numbers with known hashes.
    pub fork_blocks: sc_client_api::ForkBlocks<Block>,
    /// Known bad block hashes.
    pub bad_blocks: sc_client_api::BadBlocks<Block>,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

fn get_endowed_accounts() -> Vec<AccountId> {
    vec![
        AccountPublic::from(sp_core::sr25519::Public::from_ss58check(ORG_ADMIN).unwrap())
            .into_account(),
        AccountPublic::from(sp_core::sr25519::Public::from_ss58check(ATTESTER_1).unwrap())
            .into_account(),
        AccountPublic::from(sp_core::sr25519::Public::from_ss58check(ATTESTER_2).unwrap())
            .into_account(),
        AccountPublic::from(sp_core::sr25519::Public::from_ss58check(COUNCIL).unwrap())
            .into_account(),
    ]
}

fn get_initial_council() -> Vec<AccountId> {
    vec![
        AccountPublic::from(sp_core::sr25519::Public::from_ss58check(COUNCIL).unwrap())
            .into_account(),
    ]
}

fn create_genesis(root_key: AccountId, endowed_accounts: Vec<AccountId>) -> GenesisConfig {
    const ENDOWMENT: Balance = 10_000_000 * GRAM;
    GenesisConfig {
        frame_system: Some(SystemConfig {
            code: wasm_binary_unwrap().to_vec(),
            changes_trie_config: Default::default(),
        }),
        // indices: Some(IndicesConfig { indices: vec![] }),
        pallet_balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|x| (x, ENDOWMENT))
                .collect(),
        }),
        pallet_sudo: Some(SudoConfig { key: root_key }),
        pallet_collective_Instance1: Some(CouncilConfig {
            members: get_initial_council(),
            phantom: PhantomData,
        }),
        settings: Some(SettingsConfig {
            transaction_byte_fee: 10_000u32.into(),
            fee_split_ratio: 80,
            extrinisic_extra: vec![(3, vec![(1, 100_000)])],
        }),
        pallet_treasury: Some(Default::default()),
    }
}

pub fn development_config_genesis() -> GenesisConfig {
    let endowed_accounts = get_endowed_accounts();
    create_genesis(
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        endowed_accounts,
    )
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Development,
        development_config_genesis,
        vec![],
        None,
        Some("borlaug"),
        Some(
            json!({
                "tokenDecimals": 9,
                "tokenSymbol": "GRAM"
            })
            .as_object()
            .expect("Created an object")
            .clone(),
        ),
        Default::default(),
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use sp_runtime::BuildStorage;

    #[test]
    fn test_create_development_chain_spec() {
        development_config().build_storage().unwrap();
    }
}
