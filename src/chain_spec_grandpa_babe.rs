use runtime::{
    constants::currency::*,
    primitives::{AccountId, Balance, Signature},
    wasm_binary_unwrap, BalancesConfig, Block, CouncilConfig, GenesisConfig, SudoConfig,
    SystemConfig,
};
use runtime::{
    AuthorityDiscoveryConfig, BabeConfig, GrandpaConfig, ImOnlineConfig, SessionConfig,
    SessionKeys, SettingsConfig, StakerStatus, StakingConfig,
};
// GeneralCouncilMembershipConfig,
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::Ss58Codec, sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    Perbill,
};

type AccountPublic = <Signature as Verify>::Signer;

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

fn session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
    SessionKeys {
        grandpa,
        babe,
        im_online,
        authority_discovery,
    }
}

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

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(
    seed: &str,
) -> (
    AccountId,
    AccountId,
    GrandpaId,
    BabeId,
    ImOnlineId,
    AuthorityDiscoveryId,
) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<GrandpaId>(seed),
        get_from_seed::<BabeId>(seed),
        get_from_seed::<ImOnlineId>(seed),
        get_from_seed::<AuthorityDiscoveryId>(seed),
    )
}

fn get_endowed_accounts() -> Vec<AccountId> {
    // vec![
    //     get_account_id_from_seed::<sr25519::Public>("Alice"),
    //     get_account_id_from_seed::<sr25519::Public>("Bob"),
    //     get_account_id_from_seed::<sr25519::Public>("Charlie"),
    //     get_account_id_from_seed::<sr25519::Public>("Dave"),
    //     get_account_id_from_seed::<sr25519::Public>("Eve"),
    //     get_account_id_from_seed::<sr25519::Public>("Ferdie"),
    //     get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
    //     get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
    //     get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
    //     get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
    //     get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
    //     get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
    // ]

    vec![
        AccountPublic::from(
            sp_core::sr25519::Public::from_ss58check(
                "5EZ7gNcZidoanKK45JK4YVQNDpEScbcCNbV4BU7fJWJdAFsu",
            )
            .unwrap(),
        )
        .into_account(),
        AccountPublic::from(
            sp_core::sr25519::Public::from_ss58check(
                "5DkytoJY83z31QNKdgDitEc4K1ttLyWVW3NJfjyXqKy8DQcg",
            )
            .unwrap(),
        )
        .into_account(),
        AccountPublic::from(
            sp_core::sr25519::Public::from_ss58check(
                "5FX5WmY8WHXj7H9V7zSL3CSQ9JadBxEDsFuSGG7gUbgnm5EW",
            )
            .unwrap(),
        )
        .into_account(),
        AccountPublic::from(
            sp_core::sr25519::Public::from_ss58check(
                "5H9X5JSJTBAeUYxtMNsVSVMAyiNxyMBKqSGvgvjV4PMGgpDM",
            )
            .unwrap(),
        )
        .into_account(),
        AccountPublic::from(
            sp_core::sr25519::Public::from_ss58check(
                "5CfMkF8xrakzXaA4dW4S5iEG9PgrSbs8BkE3ooHYn9fckrQS",
            )
            .unwrap(),
        )
        .into_account(),
        AccountPublic::from(
            sp_core::sr25519::Public::from_ss58check(
                "5EbzuvEYgSgcmDZNsEdCCMwCw4mrzCTNNUk7dAAog9WwotS7",
            )
            .unwrap(),
        )
        .into_account(),
        AccountPublic::from(
            sp_core::sr25519::Public::from_ss58check(
                "5HDfARwo5GGHTr7E7vDuwKkJKt31xoJUCUFWRdzkifDQW5HK",
            )
            .unwrap(),
        )
        .into_account(),
        AccountPublic::from(
            sp_core::sr25519::Public::from_ss58check(
                "5GnGy76zKS2Yy77vvvwyhzBDxPku6yZ3Y8cBB9eiZKpQ7rUW",
            )
            .unwrap(),
        )
        .into_account(),
        AccountPublic::from(
            sp_core::sr25519::Public::from_ss58check(
                "5DDR8KcLFHFDthLnDXyEgc53r8pgT1LqcWrk7jA8PWwjow29",
            )
            .unwrap(),
        )
        .into_account(),
    ]
}

fn create_genesis(
    initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )>,
    root_key: AccountId,
    mut endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
    initial_authorities.iter().for_each(|x| {
        if !endowed_accounts.contains(&x.0) {
            endowed_accounts.push(x.0.clone())
        }
    });

    const ENDOWMENT: Balance = 10_000_000 * GRAM;
    const STASH: Balance = ENDOWMENT / 1000;
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
        pallet_session: Some(SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        }),
        pallet_sudo: Some(SudoConfig { key: root_key }),
        pallet_staking: Some(StakingConfig {
            validator_count: initial_authorities.len() as u32 * 2,
            minimum_validator_count: initial_authorities.len() as u32,
            stakers: initial_authorities
                .iter()
                .map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
                .collect(),
            invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
            slash_reward_fraction: Perbill::from_percent(10),
            ..Default::default()
        }),
        pallet_collective_Instance1: Some(CouncilConfig::default()),
        settings: Some(SettingsConfig {
            fee_split_ratio: 80,
            extrinisic_extra: vec![(3, vec![(1, 100_000)])],
        }),

        pallet_babe: Some(BabeConfig {
            authorities: vec![],
        }),
        pallet_im_online: Some(ImOnlineConfig { keys: vec![] }),
        pallet_authority_discovery: Some(AuthorityDiscoveryConfig { keys: vec![] }),

        pallet_grandpa: Some(GrandpaConfig {
            authorities: vec![],
        }),
        pallet_treasury: Some(Default::default()),
        // collective_Instance1: Some(Default::default()),
        // membership_Instance1: Some(GeneralCouncilMembershipConfig {
        //     members: vec![root_key],
        //     phantom: Default::default(),
        // }),
    }
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

pub fn aztec_config_genesis() -> GenesisConfig {
    //TODO: set up initial authorities
    let initial_authorities: Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        AuthorityDiscoveryId,
    )> = vec![];

    let root_key = AccountPublic::from(
        sp_core::sr25519::Public::from_ss58check(
            "5DDR8KcLFHFDthLnDXyEgc53r8pgT1LqcWrk7jA8PWwjow29",
        )
        .unwrap(),
    )
    .into_account();

    let endowed_accounts = get_endowed_accounts();

    create_genesis(initial_authorities, root_key, endowed_accounts)
}

/// Aztec chainspec
pub fn aztec_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Borlaug Aztec",
        "borlaug_aztec",
        ChainType::Local,
        aztec_config_genesis,
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
