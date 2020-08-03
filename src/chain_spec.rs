use runtime::{
    primitives::{AccountId, Signature},
    AuraConfig, BalancesConfig, GeneralCouncilMembershipConfig, GenesisConfig, GrandpaConfig,
    SudoConfig, SystemConfig, WASM_BINARY,
};
use sc_service;
use sc_service::ChainType;
use serde_json::json;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::Ss58Codec, sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate an authority key for Aura
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                vec![authority_keys_from_seed("Alice")],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
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
                ],
                true,
            )
        },
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
        None,
    ))
}
pub fn inca_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        "Borlaug Inca",
        "borlaug_inca",
        ChainType::Live,
        move || {
            testnet_genesis(
                wasm_binary,
                vec![
                    // get_authority_keys_from_seed("Alice"),
                    // get_authority_keys_from_seed("Bob"),
                    // get_authority_keys_from_seed("Charlie")
                    (
                        AuraId::from_ss58check("5G3WSp2yNJgRZxXvndY3qQ4VhM4mofpzpiVUuWQRVdFvDNzU")
                            .unwrap(),
                        GrandpaId::from_ss58check(
                            "5GnYdMRexbUBoP1WpbmHgvsCw1fRSH5Em44xHMqQsYHk4cRK",
                        )
                        .unwrap(),
                    ),
                    (
                        AuraId::from_ss58check("5Fej3rJdS3w2f7jkufxrNyhBMoy5zNvGVBCtRWGms7r4zsJU")
                            .unwrap(),
                        GrandpaId::from_ss58check(
                            "5FwYgvMWN1oBF4tWcCQWYZxBda17d3GvxL596A7APXMwgSdb",
                        )
                        .unwrap(),
                    ),
                    (
                        AuraId::from_ss58check("5Hive2LzHTqobHaDhJLs2PuDw6a1AV5eyyYX6fmu1RfwdQwT")
                            .unwrap(),
                        GrandpaId::from_ss58check(
                            "5DkBNwcyqZufh68Rz6Vg3C5Uqw12HpuUmeoMwTpBGD64ntMg",
                        )
                        .unwrap(),
                    ),
                ],
                AccountPublic::from(
                    sp_core::sr25519::Public::from_ss58check(
                        "5DDR8KcLFHFDthLnDXyEgc53r8pgT1LqcWrk7jA8PWwjow29",
                    )
                    .unwrap(),
                )
                .into_account(),
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
                ],
                true,
            )
        },
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
        None,
    ))
}
pub fn maya_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        "Borlaug Maya",
        "borlaug_maya",
        ChainType::Live,
        move || {
            testnet_genesis(
                wasm_binary,
                vec![
                    (
                        AuraId::from_ss58check("5G3WSp2yNJgRZxXvndY3qQ4VhM4mofpzpiVUuWQRVdFvDNzU")
                            .unwrap(),
                        GrandpaId::from_ss58check(
                            "5GnYdMRexbUBoP1WpbmHgvsCw1fRSH5Em44xHMqQsYHk4cRK",
                        )
                        .unwrap(),
                    ),
                    (
                        AuraId::from_ss58check("5Fej3rJdS3w2f7jkufxrNyhBMoy5zNvGVBCtRWGms7r4zsJU")
                            .unwrap(),
                        GrandpaId::from_ss58check(
                            "5FwYgvMWN1oBF4tWcCQWYZxBda17d3GvxL596A7APXMwgSdb",
                        )
                        .unwrap(),
                    ),
                    (
                        AuraId::from_ss58check("5Hive2LzHTqobHaDhJLs2PuDw6a1AV5eyyYX6fmu1RfwdQwT")
                            .unwrap(),
                        GrandpaId::from_ss58check(
                            "5DkBNwcyqZufh68Rz6Vg3C5Uqw12HpuUmeoMwTpBGD64ntMg",
                        )
                        .unwrap(),
                    ),
                ],
                AccountPublic::from(
                    sp_core::sr25519::Public::from_ss58check(
                        "5DDR8KcLFHFDthLnDXyEgc53r8pgT1LqcWrk7jA8PWwjow29",
                    )
                    .unwrap(),
                )
                .into_account(),
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
                ],
                true,
            )
        },
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
        None,
    ))
}

fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    GenesisConfig {
        system: Some(SystemConfig {
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        }),
        // indices: Some(IndicesConfig { indices: vec![] }),
        balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        }),
        sudo: Some(SudoConfig {
            key: root_key.clone(),
        }),
        aura: Some(AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        }),
        grandpa: Some(GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        }),
        collective_Instance1: Some(Default::default()),
        membership_Instance1: Some(GeneralCouncilMembershipConfig {
            members: vec![root_key],
            phantom: Default::default(),
        }),
    }
}
