use runtime::{
    constants::currency::*,
    primitives::{AccountId, Balance, Signature},
    wasm_binary_unwrap, BalancesConfig, Block, CouncilConfig, GenesisConfig, SudoConfig,
    SystemConfig,
};
use runtime::{AuraConfig, GrandpaConfig, SettingsConfig};
use sc_chain_spec::ChainSpecExtension;
use sc_network::config::MultiaddrWithPeerId;
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{crypto::Ss58Codec, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::marker::PhantomData;

type AccountPublic = <Signature as Verify>::Signer;

static ORG_ADMIN: &str = "5EZ7gNcZidoanKK45JK4YVQNDpEScbcCNbV4BU7fJWJdAFsu";
static ATTESTER_1: &str = "5GnGy76zKS2Yy77vvvwyhzBDxPku6yZ3Y8cBB9eiZKpQ7rUW";
static ATTESTER_2: &str = "5HDfARwo5GGHTr7E7vDuwKkJKt31xoJUCUFWRdzkifDQW5HK";

static ALPHA: &str = "5G3WSp2yNJgRZxXvndY3qQ4VhM4mofpzpiVUuWQRVdFvDNzU";
static ALPHA_ED: &str = "5GnYdMRexbUBoP1WpbmHgvsCw1fRSH5Em44xHMqQsYHk4cRK";

static BETA: &str = "5Fej3rJdS3w2f7jkufxrNyhBMoy5zNvGVBCtRWGms7r4zsJU";
static BETA_ED: &str = "5FwYgvMWN1oBF4tWcCQWYZxBda17d3GvxL596A7APXMwgSdb";

static GAMA: &str = "5Hive2LzHTqobHaDhJLs2PuDw6a1AV5eyyYX6fmu1RfwdQwT";
static GAMA_ED: &str = "5DkBNwcyqZufh68Rz6Vg3C5Uqw12HpuUmeoMwTpBGD64ntMg";

static COUNCIL: &str = "5CD9YDBg4nohwKQJ3CzwjZZsy7yka3srB6oxmuHn9rNJPx68";
static SUDO: &str = "5DDR8KcLFHFDthLnDXyEgc53r8pgT1LqcWrk7jA8PWwjow29";

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
        ORG_ADMIN, ATTESTER_1, ATTESTER_2, COUNCIL, ALPHA, BETA, GAMA, SUDO,
    ]
    .into_iter()
    .map(|public_key| {
        AccountPublic::from(sp_core::sr25519::Public::from_ss58check(public_key).unwrap())
            .into_account()
    })
    .collect()
}

fn get_initial_council() -> Vec<AccountId> {
    vec![COUNCIL]
        .into_iter()
        .map(|public_key| {
            AccountPublic::from(sp_core::sr25519::Public::from_ss58check(public_key).unwrap())
                .into_account()
        })
        .collect()
}

fn create_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
    const ENDOWMENT: Balance = 10_000_000_000_000 * GRAM;
    GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary_unwrap().to_vec(),
        },
        balances: BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, ENDOWMENT))
                .collect(),
        },
        aura: AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        grandpa: GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        },
        sudo: SudoConfig {
            // Assign network admin rights.
            key: Some(root_key),
        },
        transaction_payment: Default::default(),
        settings: SettingsConfig {
            transaction_byte_fee: 10_000u32.into(),
            fee_split_ratio: 80,
            extrinisic_extra: vec![(3, vec![(1, 100_000)])],
        },
        treasury: Default::default(),
        council: CouncilConfig {
            members: get_initial_council(),
            phantom: PhantomData,
        },
        // membership_Instance1: Some(GeneralCouncilMembershipConfig {
        //     members: vec![root_key],
        //     phantom: Default::default(),
        // }),
        audits: Default::default(),
        groups: Default::default(),
        identity: Default::default(),
        provenance: Default::default(),
    }
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(seed: &str) -> (AuraId, GrandpaId) {
    (
        get_from_seed::<AuraId>(seed),
        get_from_seed::<GrandpaId>(seed),
    )
}

pub fn development_config_genesis() -> GenesisConfig {
    let endowed_accounts = get_endowed_accounts();

    create_genesis(
        vec![authority_keys_from_seed("Alice")],
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
        Some("aztec"),
        Some(
            json!({
                "tokenDecimals": 6,
                "tokenSymbol": "GRAM"
            })
            .as_object()
            .expect("Created an object")
            .clone(),
        ),
        Default::default(),
    )
}
/// Helper function to generate stash, controller and session key from public key
pub fn authority_keys_from_public_keys(public_keys: Vec<(&str, &str)>) -> Vec<(AuraId, GrandpaId)> {
    public_keys
        .into_iter()
        .map(|(aura, grandpa)| {
            let aura = sp_core::sr25519::Public::from_ss58check(aura).unwrap();
            let grandpa = sp_core::ed25519::Public::from_ss58check(grandpa).unwrap();
            (aura.into(), grandpa.into())
        })
        .collect()
}

pub fn aztec_config_genesis() -> GenesisConfig {
    //TODO: set up initial authorities

    let initial_authorities =
        authority_keys_from_public_keys(vec![(ALPHA, ALPHA_ED), (BETA, BETA_ED), (GAMA, GAMA_ED)]);

    let root_key =
        AccountPublic::from(sp_core::sr25519::Public::from_ss58check(SUDO).unwrap()).into_account();

    let endowed_accounts = get_endowed_accounts();

    create_genesis(initial_authorities, root_key, endowed_accounts)
}

/// Aztec chainspec
pub fn aztec_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Borlaug Aztec",
        "borlaug_aztec",
        ChainType::Live,
        aztec_config_genesis,
        vec![],
        None,
        Some("borlaug"),
        None,
        Some(
            json!({
                "tokenDecimals": 6,
                "tokenSymbol": "GRAM"
            })
            .as_object()
            .expect("Created an object")
            .clone(),
        ),
        Default::default(),
    )
}
/// chainspec for testing locally
pub fn local_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Borlaug Aztec",
        "borlaug_aztec",
        ChainType::Live,
        aztec_config_genesis,
        vec![
            "/ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEUomfFERvd2NVqs1hon8nbt2DXwaN9VmkCaZvLHQeUD3"
                .parse::<MultiaddrWithPeerId>()
                .unwrap(),
            "/ip4/127.0.0.1/tcp/30334/p2p/12D3KooWBKtjoJC64dibA6ezcEvwt7ne1aBxWHMbx3dmXr5bsGPn"
                .parse::<MultiaddrWithPeerId>()
                .unwrap(),
            "/ip4/127.0.0.1/tcp/30335/p2p/12D3KooWK11nqRKB794ZpfDNG4coJRFy5iT7Ma9NzXpoKJNHd5jr"
                .parse::<MultiaddrWithPeerId>()
                .unwrap(),
        ],
        None,
        Some("borlaug"),
        Some("aztec"),
        Some(
            json!({
                "tokenDecimals": 6,
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
