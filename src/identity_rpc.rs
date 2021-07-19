use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use codec::{Codec, Decode, Encode};
use identity_runtime_api::IdentityApi as IdentityRuntimeApi;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_primitives::{Attestation, DidDocument, DidProperty, Fact, Statement};
use serde::{
    Deserialize, Serialize,
    {
        de::{self, Deserializer, Visitor},
        ser::Serializer,
    },
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::convert::TryFrom;
use std::fmt;
use std::sync::Arc;

#[rpc]
pub trait IdentityApi<BlockHash, AccountId, CatalogId, ClaimId, Moment> {
    #[rpc(name = "get_catalogs")]
    fn get_catalogs(
        &self,
        account: AccountId,
        at: Option<BlockHash>,
    ) -> Result<Vec<CatalogResponse<CatalogId>>>;

    #[rpc(name = "get_catalog")]
    fn get_catalog(
        &self,
        catalog_id: CatalogId,
        at: Option<BlockHash>,
    ) -> Result<CatalogResponse<CatalogId>>;

    #[rpc(name = "get_dids_in_catalog")]
    fn get_dids_in_catalog(
        &self,
        catalog_id: CatalogId,
        at: Option<BlockHash>,
    ) -> Result<Vec<DidDocumentBasicResponse>>;

    #[rpc(name = "get_did_in_catalog")]
    fn get_did_in_catalog(
        &self,
        catalog_id: CatalogId,
        did: Did,
        at: Option<BlockHash>,
    ) -> Result<DidDocumentResponse<AccountId>>;

    #[rpc(name = "get_did")]
    fn get_did(&self, did: Did, at: Option<BlockHash>) -> Result<DidDocumentResponse<AccountId>>;

    #[rpc(name = "get_dids_by_subject")]
    fn get_dids_by_subject(
        &self,
        subject: AccountId,
        at: Option<BlockHash>,
    ) -> Result<Vec<DidDocumentBasicResponse>>;

    #[rpc(name = "get_dids_by_controller")]
    fn get_dids_by_controller(
        &self,
        controller: AccountId,
        at: Option<BlockHash>,
    ) -> Result<Vec<DidDocumentBasicResponse>>;

    #[rpc(name = "get_claims")]
    fn get_claims(
        &self,
        did: Did,
        at: Option<BlockHash>,
    ) -> Result<Vec<ClaimResponse<ClaimId, AccountId, Moment>>>;
}

#[derive(Encode, Default, Decode, Debug, Clone)]
pub struct Did {
    pub id: [u8; 32],
}

impl From<Did> for pallet_primitives::Did {
    fn from(did: Did) -> pallet_primitives::Did {
        pallet_primitives::Did { id: did.id }
    }
}

impl<'de> Deserialize<'de> for Did {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DidVisitor;

        impl<'de> Visitor<'de> for DidVisitor {
            type Value = Did;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("String")
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Did, E>
            where
                E: de::Error,
            {
                if value.len() != 66 {
                    return Err(E::custom("Invalid DID".to_string()));
                }
                let mut array: [u8; 32] = Default::default();
                //should be safe since chars are hex only
                let hex_only = &value[2..];
                let bytes: &[u8] =
                    &hex::decode(hex_only).map_err(|e| E::custom(format!("Invalid DID: {}", e)))?;
                array.copy_from_slice(&bytes[0..32]);
                Ok(Did { id: array })
            }
        }

        deserializer.deserialize_identifier(DidVisitor)
    }
}

impl From<Did> for std::string::String {
    fn from(did: Did) -> std::string::String {
        format!("0x{}", hex::encode(did.id))
    }
}

impl From<pallet_primitives::Did> for Did {
    fn from(did: pallet_primitives::Did) -> Did {
        Did { id: did.id }
    }
}

impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.id))
    }
}

#[derive(Debug)]
pub struct DidParseError(String);

impl TryFrom<String> for Did {
    type Error = DidParseError;
    fn try_from(str: String) -> std::result::Result<Did, DidParseError> {
        if str.len() != 66 {
            return Err(DidParseError("Invalid DID".to_string()));
        }
        let mut array: [u8; 32] = Default::default();
        //should be safe since chars are hex only
        let hex_only = &str[2..];
        let bytes = match hex::decode(hex_only) {
            Ok(bytes) => bytes,
            Err(err) => return Err(DidParseError(format!("{}", err))),
        };

        array.copy_from_slice(&bytes.as_slice()[0..32]);
        Ok(Did { id: array })
    }
}

impl From<&[u8]> for Did {
    fn from(bytes: &[u8]) -> Did {
        let mut array: [u8; 32] = Default::default();
        array.copy_from_slice(&bytes[0..32]);
        Did { id: array }
    }
}

impl Serialize for Did {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("0x{}", hex::encode(self.id)))
    }
}

#[derive(Serialize, Deserialize)]
pub struct CatalogResponse<CatalogId> {
    pub catalog_id: CatalogId,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct DidDocumentBasicResponse {
    pub did: String,
    pub short_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DidDocumentResponse<AccountId> {
    pub short_name: Option<String>,
    pub subject: AccountId,
    pub controllers: Vec<AccountId>,
    pub properties: Vec<DidPropertyResponse>,
}

impl<BoundedString> From<(pallet_primitives::Did, Option<BoundedString>)>
    for DidDocumentBasicResponse
where
    BoundedString: Into<Vec<u8>>,
{
    fn from((did, short_name): (pallet_primitives::Did, Option<BoundedString>)) -> Self {
        let did: Did = did.into();
        DidDocumentBasicResponse {
            did: did.to_string(),
            short_name: short_name
                .map(|short_name| String::from_utf8_lossy(&short_name.into()).to_string()),
        }
    }
}

impl<ClaimId, AccountId, Moment, BoundedStringName, BoundedStringFact>
    From<(
        ClaimId,
        pallet_primitives::Claim<AccountId, Moment, BoundedStringName, BoundedStringFact>,
    )> for ClaimResponse<ClaimId, AccountId, Moment>
where
    BoundedStringName: Into<Vec<u8>>,
    BoundedStringFact: Into<Vec<u8>>,
{
    fn from(
        (claim_id, claim): (
            ClaimId,
            pallet_primitives::Claim<AccountId, Moment, BoundedStringName, BoundedStringFact>,
        ),
    ) -> Self {
        ClaimResponse {
            claim_id,
            description: String::from_utf8_lossy(&claim.description.into()).to_string(),
            statements: claim.statements.into_iter().map(|s| s.into()).collect(),
            created_by: claim.created_by,
            attestation: claim.attestation.map(|a| a.into()),
        }
    }
}

impl<AccountId, BoundedStringName, BoundedStringFact>
    From<(
        Option<BoundedStringName>,
        DidDocument<AccountId, BoundedStringName>,
        Vec<DidProperty<BoundedStringName, BoundedStringFact>>,
        Vec<AccountId>,
    )> for DidDocumentResponse<AccountId>
where
    BoundedStringName: Clone + Into<Vec<u8>>,
    BoundedStringFact: Into<Vec<u8>>,
{
    fn from(
        (short_name, did_document, properties, controllers): (
            Option<BoundedStringName>,
            DidDocument<AccountId, BoundedStringName>,
            Vec<DidProperty<BoundedStringName, BoundedStringFact>>,
            Vec<AccountId>,
        ),
    ) -> Self {
        DidDocumentResponse {
            short_name: short_name
                .or_else(|| did_document.short_name.clone())
                .map(|short_name| String::from_utf8_lossy(&short_name.into()).to_string()),
            subject: did_document.subject,
            controllers,
            properties: properties.into_iter().map(|p| p.into()).collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DidPropertyResponse {
    pub name: String,
    pub fact: FactResponse,
}

impl<BoundedStringName, BoundedStringFact> From<DidProperty<BoundedStringName, BoundedStringFact>>
    for DidPropertyResponse
where
    BoundedStringName: Into<Vec<u8>>,
    BoundedStringFact: Into<Vec<u8>>,
{
    fn from(property: DidProperty<BoundedStringName, BoundedStringFact>) -> Self {
        DidPropertyResponse {
            name: String::from_utf8_lossy(&property.name.into()).to_string(),
            fact: property.fact.into(),
        }
    }
}

impl<AccountId, Moment> From<Attestation<AccountId, Moment>>
    for AttestationResponse<AccountId, Moment>
{
    fn from(attestation: Attestation<AccountId, Moment>) -> Self {
        AttestationResponse {
            attested_by: attestation.attested_by,
            valid_until: attestation.valid_until,
        }
    }
}

impl<BoundedStringName, BoundedStringFact> From<Statement<BoundedStringName, BoundedStringFact>>
    for StatementResponse
where
    BoundedStringName: Into<Vec<u8>>,
    BoundedStringFact: Into<Vec<u8>>,
{
    fn from(statement: Statement<BoundedStringName, BoundedStringFact>) -> Self {
        StatementResponse {
            name: String::from_utf8_lossy(&statement.name.into()).to_string(),
            fact: statement.fact.into(),
            for_issuer: statement.for_issuer,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct FactResponse {
    #[serde(rename = "type")]
    pub data_type: String,
    pub value: String,
}

impl<BoundedString> From<Fact<BoundedString>> for FactResponse
where
    BoundedString: Into<Vec<u8>>,
{
    fn from(fact: Fact<BoundedString>) -> Self {
        match fact {
            Fact::Bool(value) => FactResponse {
                data_type: String::from("Bool"),
                value: value.to_string(),
            },
            Fact::Text(value) => FactResponse {
                data_type: String::from("Text"),
                value: String::from_utf8_lossy(&value.into()).to_string(),
            },
            Fact::U8(value) => FactResponse {
                data_type: String::from("U8"),
                value: value.to_string(),
            },
            Fact::U16(value) => FactResponse {
                data_type: String::from("U16"),
                value: value.to_string(),
            },
            Fact::U32(value) => FactResponse {
                data_type: String::from("U32"),
                value: value.to_string(),
            },
            Fact::U128(value) => FactResponse {
                data_type: String::from("U128"),
                value: value.to_string(),
            },
            Fact::Date(year, month, day) => {
                let date = NaiveDate::from_ymd(i32::from(year), u32::from(month), u32::from(day));
                FactResponse {
                    data_type: String::from("Date"),
                    value: date.to_string(),
                }
            }
            //TODO: check that this conversion is correct
            Fact::Iso8601(year, month, day, hour, minute, second, timezone) => {
                let timezone = String::from_utf8_lossy(&timezone).to_string();
                let date = NaiveDate::from_ymd(i32::from(year), u32::from(month), u32::from(day));
                let time =
                    NaiveTime::from_hms(u32::from(hour), u32::from(minute), u32::from(second));
                let dt = NaiveDateTime::new(date, time);
                FactResponse {
                    data_type: String::from("Iso8601"),
                    value: format!("{}{}", dt, timezone),
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct StatementResponse {
    pub name: String,
    pub fact: FactResponse,
    pub for_issuer: bool,
}

#[derive(Serialize, Deserialize)]
pub struct AttestationResponse<AccountId, Moment> {
    pub attested_by: AccountId,
    pub valid_until: Moment,
}

#[derive(Serialize, Deserialize)]
pub struct ClaimResponse<ClaimId, AccountId, Moment> {
    pub claim_id: ClaimId,
    pub description: String,
    pub statements: Vec<StatementResponse>,
    pub created_by: AccountId,
    pub attestation: Option<AttestationResponse<AccountId, Moment>>,
}

pub struct Identity<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Identity<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

macro_rules! convert_error {
    () => {{
        |e| RpcError {
            code: ErrorCode::ServerError(1),
            message: "Error in Identity API".into(),
            data: Some(format!("{:?}", e).into()),
        }
    }};
}

macro_rules! not_found_error {
    () => {{
        RpcError {
            code: ErrorCode::ServerError(404),
            message: "Entity not found".into(),
            data: Some("Entity not found".into()),
        }
    }};
}

impl<C, Block, AccountId, CatalogId, ClaimId, Moment, BoundedStringName, BoundedStringFact>
    IdentityApi<<Block as BlockT>::Hash, AccountId, CatalogId, ClaimId, Moment>
    for Identity<
        C,
        (
            Block,
            AccountId,
            CatalogId,
            ClaimId,
            BoundedStringName,
            BoundedStringFact,
        ),
    >
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: IdentityRuntimeApi<
        Block,
        AccountId,
        CatalogId,
        ClaimId,
        Moment,
        BoundedStringName,
        BoundedStringFact,
    >,
    AccountId: Codec + Send + Sync + 'static,
    CatalogId: Codec + Copy + Send + Sync + 'static,
    ClaimId: Codec + Copy + Send + Sync + 'static,
    Moment: Codec + Copy + Send + Sync + 'static,
    BoundedStringName: Codec + Clone + Send + Sync + 'static + Into<Vec<u8>>,
    BoundedStringFact: Codec + Clone + Send + Sync + 'static + Into<Vec<u8>>,
{
    fn get_catalogs(
        &self,
        account: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<CatalogResponse<CatalogId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let catalogs = api.get_catalogs(&at, account).map_err(convert_error!())?;
        Ok(catalogs
            .into_iter()
            .map(|(catalog_id, catalog)| CatalogResponse::<CatalogId> {
                catalog_id,
                name: String::from_utf8_lossy(&catalog.name.into()).to_string(),
            })
            .collect())
    }

    fn get_catalog(
        &self,
        catalog_id: CatalogId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<CatalogResponse<CatalogId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let catalog = api
            .get_catalog(&at, catalog_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;

        Ok(CatalogResponse::<CatalogId> {
            catalog_id,
            name: String::from_utf8_lossy(&catalog.name.into()).to_string(),
        })
    }

    fn get_dids_in_catalog(
        &self,
        catalog_id: CatalogId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<DidDocumentBasicResponse>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let dids = api
            .get_dids_in_catalog(&at, catalog_id)
            .map_err(convert_error!())?;
        Ok(dids
            .into_iter()
            .map(|(did, name)| (did, Some(name)).into())
            .collect())
    }

    fn get_did_in_catalog(
        &self,
        catalog_id: CatalogId,
        did: Did,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<DidDocumentResponse<AccountId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let (name, did_document, properties, controllers) = api
            .get_did_in_catalog(&at, catalog_id, did.clone().into())
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;
        Ok((Some(name), did_document, properties, controllers).into())
    }

    fn get_did(
        &self,
        did: Did,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<DidDocumentResponse<AccountId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let (did_document, properties, controllers) = api
            .get_did(&at, did.clone().into())
            .map_err(convert_error!())?
            .ok_or(not_found_error!())?;
        Ok((None, did_document, properties, controllers).into())
    }

    fn get_dids_by_subject(
        &self,
        subject: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<DidDocumentBasicResponse>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let dids = api
            .get_dids_by_subject(&at, subject)
            .map_err(convert_error!())?;
        Ok(dids
            .into_iter()
            .map(|(did, maybe_short_name)| (did, maybe_short_name).into())
            .collect())
    }

    fn get_dids_by_controller(
        &self,
        controller: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<DidDocumentBasicResponse>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let dids = api
            .get_dids_by_controller(&at, controller)
            .map_err(convert_error!())?;
        Ok(dids
            .into_iter()
            .map(|(did, maybe_short_name)| (did, maybe_short_name).into())
            .collect())
    }

    fn get_claims(
        &self,
        did: Did,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<ClaimResponse<ClaimId, AccountId, Moment>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let claims = api
            .get_claims(&at, did.clone().into())
            .map_err(convert_error!())?;
        Ok(claims
            .into_iter()
            .map(|(claim_id, claim)| (claim_id, claim).into())
            .collect())
    }
}
