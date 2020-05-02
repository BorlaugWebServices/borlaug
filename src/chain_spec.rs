use grandpa_primitives::AuthorityId as GrandpaId;
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
use sp_runtime::traits::{IdentifyAccount, Verify};

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
    /// Whatever the current runtime is, with just Alice as an auth.
    Development,
    /// The Inca testnet.
    IncaTestnet,
    /// The development testnet.
    DevelopmentTestnet,
}

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

pub fn development_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Development,
        || {
            testnet_genesis(
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
    )
}
pub fn inca_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Borlaug Inca",
        "inca",
        ChainType::Live,
        || {
            testnet_genesis(
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
    )
}
pub fn maya_config() -> ChainSpec {
    ChainSpec::from_genesis(
        "Borlaug Maya",
        "maya",
        ChainType::Live,
        || {
            testnet_genesis(
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
    )
}

// impl Alternative {
//     /// Get an actual chain config from one of the alternatives.
//     pub(crate) fn load(self) -> Result<ChainSpec, String> {
//         Ok(match self {
//             Alternative::Development => ChainSpec::from_genesis(
//                 "Development",
//                 "dev",
//                 || {
//                     testnet_genesis(
//                         vec![get_authority_keys_from_seed("Alice")],
//                         get_account_id_from_seed::<sr25519::Public>("Alice"),
//                         vec![
//                             get_account_id_from_seed::<sr25519::Public>("Alice"),
//                             get_account_id_from_seed::<sr25519::Public>("Bob"),
//                             get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
//                             get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
//                             AccountPublic::from(
//                                 sp_core::sr25519::Public::from_ss58check(
//                                     "5EZ7gNcZidoanKK45JK4YVQNDpEScbcCNbV4BU7fJWJdAFsu",
//                                 )
//                                 .unwrap(),
//                             )
//                             .into_account(),
//                             AccountPublic::from(
//                                 sp_core::sr25519::Public::from_ss58check(
//                                     "5DkytoJY83z31QNKdgDitEc4K1ttLyWVW3NJfjyXqKy8DQcg",
//                                 )
//                                 .unwrap(),
//                             )
//                             .into_account(),
//                             AccountPublic::from(
//                                 sp_core::sr25519::Public::from_ss58check(
//                                     "5FX5WmY8WHXj7H9V7zSL3CSQ9JadBxEDsFuSGG7gUbgnm5EW",
//                                 )
//                                 .unwrap(),
//                             )
//                             .into_account(),
//                             AccountPublic::from(
//                                 sp_core::sr25519::Public::from_ss58check(
//                                     "5H9X5JSJTBAeUYxtMNsVSVMAyiNxyMBKqSGvgvjV4PMGgpDM",
//                                 )
//                                 .unwrap(),
//                             )
//                             .into_account(),
//                         ],
//                         true,
//                     )
//                 },
//                 vec![],
//                 None,
//                 Some("borlaug"),
//                 Some(
//                     json!({
//                         "tokenDecimals": 9,
//                         "tokenSymbol": "GRAM"
//                     })
//                     .as_object()
//                     .expect("Created an object")
//                     .clone(),
//                 ),
//                 None,
//             ),
//             Alternative::IncaTestnet => ChainSpec::from_genesis(
//                 "Borlaug Testnet Inca",
//                 "borlaug_testnet_inca",
//                 || {
//                     testnet_genesis(
//                         vec![
//                             // get_authority_keys_from_seed("Alice"),
//                             // get_authority_keys_from_seed("Bob"),
//                             // get_authority_keys_from_seed("Charlie")
//                             (
//                                 AuraId::from_ss58check(
//                                     "5G3WSp2yNJgRZxXvndY3qQ4VhM4mofpzpiVUuWQRVdFvDNzU",
//                                 )
//                                 .unwrap(),
//                                 GrandpaId::from_ss58check(
//                                     "5GnYdMRexbUBoP1WpbmHgvsCw1fRSH5Em44xHMqQsYHk4cRK",
//                                 )
//                                 .unwrap(),
//                             ),
//                             (
//                                 AuraId::from_ss58check(
//                                     "5Fej3rJdS3w2f7jkufxrNyhBMoy5zNvGVBCtRWGms7r4zsJU",
//                                 )
//                                 .unwrap(),
//                                 GrandpaId::from_ss58check(
//                                     "5FwYgvMWN1oBF4tWcCQWYZxBda17d3GvxL596A7APXMwgSdb",
//                                 )
//                                 .unwrap(),
//                             ),
//                             (
//                                 AuraId::from_ss58check(
//                                     "5Hive2LzHTqobHaDhJLs2PuDw6a1AV5eyyYX6fmu1RfwdQwT",
//                                 )
//                                 .unwrap(),
//                                 GrandpaId::from_ss58check(
//                                     "5DkBNwcyqZufh68Rz6Vg3C5Uqw12HpuUmeoMwTpBGD64ntMg",
//                                 )
//                                 .unwrap(),
//                             ),
//                         ],
//                         AccountPublic::from(
//                             sp_core::sr25519::Public::from_ss58check(
//                                 "5DDR8KcLFHFDthLnDXyEgc53r8pgT1LqcWrk7jA8PWwjow29",
//                             )
//                             .unwrap(),
//                         )
//                         .into_account(),
//                         vec![
//                             AccountPublic::from(
//                                 sp_core::sr25519::Public::from_ss58check(
//                                     "5EZ7gNcZidoanKK45JK4YVQNDpEScbcCNbV4BU7fJWJdAFsu",
//                                 )
//                                 .unwrap(),
//                             )
//                             .into_account(),
//                             AccountPublic::from(
//                                 sp_core::sr25519::Public::from_ss58check(
//                                     "5DkytoJY83z31QNKdgDitEc4K1ttLyWVW3NJfjyXqKy8DQcg",
//                                 )
//                                 .unwrap(),
//                             )
//                             .into_account(),
//                             AccountPublic::from(
//                                 sp_core::sr25519::Public::from_ss58check(
//                                     "5FX5WmY8WHXj7H9V7zSL3CSQ9JadBxEDsFuSGG7gUbgnm5EW",
//                                 )
//                                 .unwrap(),
//                             )
//                             .into_account(),
//                             AccountPublic::from(
//                                 sp_core::sr25519::Public::from_ss58check(
//                                     "5H9X5JSJTBAeUYxtMNsVSVMAyiNxyMBKqSGvgvjV4PMGgpDM",
//                                 )
//                                 .unwrap(),
//                             )
//                             .into_account(),
//                         ],
//                         true,
//                     )
//                 },
//                 vec![
//                     String::from("/ip4/3.6.94.87/tcp/30333/p2p/QmPZHibpY11toY1nQcArpryGyJxYFXBWWotg3TKLpBT7ug"), //Mumbai
//                     String::from("/ip4/3.1.196.206/tcp/30333/p2p/QmPZrx6TXbMcrgMgMqYMd6rhjbLXEYFYW2rV6i5UF7DnTP"), //Singapore
//                     String::from("/ip4/3.124.8.22/tcp/30333/p2p/QmWDUmXLxynkSyBzGGDEwnbW6i3o7qnsupadRLCmMtbMNk"), //Frankfurt
//                 ],
//                 None,
//                 Some("borlaug"),
//                 Some(
//                     json!({
//                         "tokenDecimals": 9,
//                         "tokenSymbol": "GRAM"
//                     })
//                     .as_object()
//                     .expect("Created an object")
//                     .clone(),
//                 ),
//                 None,
//             ),

//             Alternative::DevelopmentTestnet => ChainSpec::from_genesis(
//                 "Borlaug Testnet Development",
//                 "borlaug_testnet_development",
//                 || {
//                     testnet_genesis(
//                         vec![
//                             (
//                                 AuraId::from_ss58check(
//                                     "5G3WSp2yNJgRZxXvndY3qQ4VhM4mofpzpiVUuWQRVdFvDNzU",
//                                 )
//                                 .unwrap(),
//                                 GrandpaId::from_ss58check(
//                                     "5GnYdMRexbUBoP1WpbmHgvsCw1fRSH5Em44xHMqQsYHk4cRK",
//                                 )
//                                 .unwrap(),
//                             ),
//                             (
//                                 AuraId::from_ss58check(
//                                     "5Fej3rJdS3w2f7jkufxrNyhBMoy5zNvGVBCtRWGms7r4zsJU",
//                                 )
//                                 .unwrap(),
//                                 GrandpaId::from_ss58check(
//                                     "5FwYgvMWN1oBF4tWcCQWYZxBda17d3GvxL596A7APXMwgSdb",
//                                 )
//                                 .unwrap(),
//                             ),
//                             (
//                                 AuraId::from_ss58check(
//                                     "5Hive2LzHTqobHaDhJLs2PuDw6a1AV5eyyYX6fmu1RfwdQwT",
//                                 )
//                                 .unwrap(),
//                                 GrandpaId::from_ss58check(
//                                     "5DkBNwcyqZufh68Rz6Vg3C5Uqw12HpuUmeoMwTpBGD64ntMg",
//                                 )
//                                 .unwrap(),
//                             ),
//                         ],
//                         AccountPublic::from(
//                             sp_core::sr25519::Public::from_ss58check(
//                                 "5DDR8KcLFHFDthLnDXyEgc53r8pgT1LqcWrk7jA8PWwjow29",
//                             )
//                             .unwrap(),
//                         )
//                         .into_account(),
//                         vec![
//                             AccountPublic::from(
//                                 sp_core::sr25519::Public::from_ss58check(
//                                     "5EZ7gNcZidoanKK45JK4YVQNDpEScbcCNbV4BU7fJWJdAFsu",
//                                 )
//                                 .unwrap(),
//                             )
//                             .into_account(),
//                             AccountPublic::from(
//                                 sp_core::sr25519::Public::from_ss58check(
//                                     "5DkytoJY83z31QNKdgDitEc4K1ttLyWVW3NJfjyXqKy8DQcg",
//                                 )
//                                 .unwrap(),
//                             )
//                             .into_account(),
//                             AccountPublic::from(
//                                 sp_core::sr25519::Public::from_ss58check(
//                                     "5FX5WmY8WHXj7H9V7zSL3CSQ9JadBxEDsFuSGG7gUbgnm5EW",
//                                 )
//                                 .unwrap(),
//                             )
//                             .into_account(),
//                             AccountPublic::from(
//                                 sp_core::sr25519::Public::from_ss58check(
//                                     "5H9X5JSJTBAeUYxtMNsVSVMAyiNxyMBKqSGvgvjV4PMGgpDM",
//                                 )
//                                 .unwrap(),
//                             )
//                             .into_account(),
//                         ],
//                         true,
//                     )
//                 },
//                 vec![
//                     String::from("/ip4/10.0.0.23/tcp/30333/p2p/QmQKY7Lfe5aHfhppg7YtsnUwbEJnUfgcvPQoZX45f2FM5g"),
//                     String::from("/ip4/10.0.0.253/tcp/30333/p2p/QmRe6NHXio99XN5MDAQbJsqPi52FWmshYq3ALVADMPrKNZ"),
//                     String::from("/ip4/10.0.0.71/tcp/30333/p2p/QmdYsNn9gRPpXpQSdJXw9dh1wPdxqJnsxoZVzWeUu8LD2Q"),
//                 ],
//                 None,
//                 Some("borlaug"),
//                 Some(
//                     json!({
//                         "tokenDecimals": 9,
//                         "tokenSymbol": "GRAM"
//                     })
//                     .as_object()
//                     .expect("Created an object")
//                     .clone(),
//                 ),
//                 None,
//             ),
//         })
//     }

//     pub(crate) fn from(s: &str) -> Option<Self> {
//         match s {
//             "dev" => Some(Alternative::Development),
//             "inca" => Some(Alternative::IncaTestnet),
//             "" | "development" => Some(Alternative::DevelopmentTestnet),
//             _ => None,
//         }
//     }
// }

fn testnet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    GenesisConfig {
        system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
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

// pub fn load_spec(id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
//     Ok(match Alternative::from(id) {
//         Some(spec) => Box::new(spec.load()?),
//         None => Box::new(ChainSpec::from_json_file(std::path::PathBuf::from(id))?),
//     })
// }
