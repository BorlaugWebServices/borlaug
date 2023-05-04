use codec::Codec;
use frame_support::dispatch::fmt::Display;
use groups_runtime_api::GroupsApi as GroupsRuntimeApi;
use jsonrpsee::{
    core::{RpcResult},
    proc_macros::rpc,
    types::error::{CallError,  ErrorObject},
};
use pallet_primitives::{Group, Votes};
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{

    traits::{AtLeast32BitUnsigned, Block as BlockT},
};
use std::sync::Arc;

#[rpc(client, server)]
pub trait GroupsApi<BlockHash, AccountId, GroupId, MemberCount, ProposalId, Hash> {
    #[method(name = "member_of")]
    fn member_of(
        &self,
        account_id: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<GroupResponse<GroupId, AccountId, MemberCount>>>;

    #[method(name = "is_member")]
    fn is_member(
        &self,
        group_id: GroupId,
        account_id: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<bool>;

    #[method(name = "get_group_by_account")]
    fn get_group_by_account(
        &self,
        account_id: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<GroupResponse<GroupId, AccountId, MemberCount>>;

    #[method(name = "get_group_account")]
    fn get_group_account(&self, group_id: GroupId, at: Option<BlockHash>) -> RpcResult<AccountId>;

    #[method(name = "get_group")]
    fn get_group(
        &self,
        group_id: GroupId,
        at: Option<BlockHash>,
    ) -> RpcResult<GroupResponse<GroupId, AccountId, MemberCount>>;

    #[method(name = "get_sub_groups")]
    fn get_sub_groups(
        &self,
        group_id: GroupId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<GroupResponse<GroupId, AccountId, MemberCount>>>;

    #[method(name = "get_proposal")]
    fn get_proposal(
        &self,
        proposal_id: ProposalId,
        at: Option<BlockHash>,
    ) -> RpcResult<ProposalResponse<ProposalId, GroupId, AccountId, MemberCount>>;

    #[method(name = "get_proposals_by_group")]
    fn get_proposals_by_group(
        &self,
        group_id: GroupId,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<ProposalResponse<ProposalId, GroupId, AccountId, MemberCount>>>;

    #[method(name = "get_proposals_by_account")]
    fn get_proposals_by_account(
        &self,
        account_id: AccountId,
        at: Option<BlockHash>,
    ) -> RpcResult<
        Vec<(
            GroupId,
            Vec<ProposalResponse<ProposalId, GroupId, AccountId, MemberCount>>,
        )>,
    >;
}

#[derive(Serialize, Deserialize)]
pub struct GroupResponse<GroupId, AccountId, MemberCount> {
    pub group_id: GroupId,
    pub name: String,
    pub total_vote_weight: MemberCount,
    pub members: Vec<(AccountId, MemberCount)>,
    pub threshold: MemberCount,
    pub anonymous_account: AccountId,
    //u64 instead of Balance due to bug in serde https://github.com/paritytech/substrate/issues/4641
    pub balance: u64,
    pub parent: Option<GroupId>,
}
impl<GroupId, AccountId, MemberCount, BoundedString, Balance>
    From<(
        GroupId,
        Group<GroupId, AccountId, MemberCount, BoundedString>,
        Vec<(AccountId, MemberCount)>,
        Balance,
    )> for GroupResponse<GroupId, AccountId, MemberCount>
where
    BoundedString: Into<Vec<u8>>,
    Balance: AtLeast32BitUnsigned,
{
    fn from(
        (group_id, group, members, balance): (
            GroupId,
            Group<GroupId, AccountId, MemberCount, BoundedString>,
            Vec<(AccountId, MemberCount)>,
            Balance,
        ),
    ) -> Self {
        GroupResponse {
            group_id,
            name: String::from_utf8_lossy(&group.name.into()).to_string(),
            total_vote_weight: group.total_vote_weight,
            members,
            threshold: group.threshold,
            anonymous_account: group.anonymous_account,
            balance: balance.unique_saturated_into(),
            parent: group.parent,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ProposalResponse<ProposalId, GroupId, AccountId, MemberCount> {
    pub proposal_id: ProposalId,
    pub group_id: GroupId,
    pub members: Vec<(AccountId, MemberCount)>,
    pub hash: Option<String>,
    pub proposal_len: Option<u32>,
    pub votes: VotesResponse<AccountId, MemberCount>,
}
impl<ProposalId, GroupId, Hash, AccountId, MemberCount>
    From<(
        ProposalId,
        GroupId,
        Vec<(AccountId, MemberCount)>,
        Option<(Hash, u32)>,
        Votes<AccountId, MemberCount>,
    )> for ProposalResponse<ProposalId, GroupId, AccountId, MemberCount>
where
    Hash: AsRef<[u8]>,
{
    fn from(
        (proposal_id, group_id, members, proposal, votes): (
            ProposalId,
            GroupId,
            Vec<(AccountId, MemberCount)>,
            Option<(Hash, u32)>,
            Votes<AccountId, MemberCount>,
        ),
    ) -> Self {
        ProposalResponse {
            proposal_id,
            group_id,
            members,
            hash: proposal
                .as_ref()
                .map(|(hash, _len)| String::from_utf8_lossy(hash.as_ref()).to_string()),
            proposal_len: proposal.map(|(_hash, len)| len),
            votes: votes.into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct VotesResponse<AccountId, MemberCount> {
    pub threshold: MemberCount,
    pub total_vote_weight: MemberCount,
    pub ayes: Vec<(AccountId, MemberCount)>,
    pub nays: Vec<(AccountId, MemberCount)>,
}
impl<AccountId, MemberCount> From<Votes<AccountId, MemberCount>>
    for VotesResponse<AccountId, MemberCount>
{
    fn from(votes: Votes<AccountId, MemberCount>) -> Self {
        VotesResponse {
            threshold: votes.threshold,
            total_vote_weight: votes.total_vote_weight,
            ayes: votes.ayes,
            nays: votes.nays,
        }
    }
}

pub struct Groups<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Groups<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

/// Error type of this RPC api.
pub enum Error {
    /// The transaction was not decodable.
    DecodeError,
    /// The call to runtime failed.
    RuntimeError,
    NotFoundError,
}

impl From<Error> for i32 {
    fn from(e: Error) -> i32 {
        match e {
            Error::RuntimeError => 1,
            Error::DecodeError => 2,
            Error::NotFoundError => 404,
        }
    }
}

static RPC_MODULE: &str = "Groups API";

macro_rules! convert_error {
    () => {{
        |e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                format!("Runtime Error in {}", RPC_MODULE),
                Some(format!("{:?}", e)),
            ))
        }
    }};
}

// macro_rules! decode_error {
//     () => {{
//         |e| {
//             CallError::Custom(ErrorObject::owned(
//                 Error::DecodeError.into(),
//                 format!("Decode Error in {}", RPC_MODULE),
//                 Some(format!("{:?}", e)),
//             ))
//         }
//     }};
// }

macro_rules! not_found_error {
    ($id:expr) => {{
        {
            CallError::Custom(ErrorObject::owned(
                Error::NotFoundError.into(),
                format!("Entity not found Error in {}", RPC_MODULE),
                Some(format!("{}", $id)),
            ))
        }
    }};
}

impl<C, Block, AccountId, GroupId, MemberCount, ProposalId, Hash, BoundedString, Balance>
    GroupsApiServer<<Block as BlockT>::Hash, AccountId, GroupId, MemberCount, ProposalId, Hash>
    for Groups<
        C,
        (
            Block,
            AccountId,
            GroupId,
            MemberCount,
            ProposalId,
            Hash,
            BoundedString,
            Balance,
        ),
    >
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: GroupsRuntimeApi<
        Block,
        AccountId,
        GroupId,
        MemberCount,
        ProposalId,
        Hash,
        BoundedString,
        Balance,
    >,
    GroupId: Codec + Copy + Send + Sync + 'static + Display,
    MemberCount: Codec + Copy + Send + Sync + 'static + Display,
    AccountId: Codec + Send + Sync + 'static + Display + Clone,
    ProposalId: Codec + Copy + Send + Sync + 'static + Display,
    Hash: Codec + Clone + Send + Sync + 'static + AsRef<[u8]>,
    BoundedString: Codec + Clone + Send + Sync + 'static + Into<Vec<u8>>,
    Balance: Codec + Copy + Send + Sync + AtLeast32BitUnsigned + 'static,
{
    fn member_of(
        &self,
        account_id: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<GroupResponse<GroupId, AccountId, MemberCount>>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let groups = api.member_of(at, account_id).map_err(convert_error!())?;
        Ok(groups.into_iter().map(|g| g.into()).collect())
    }

    fn is_member(
        &self,
        group_id: GroupId,
        account_id: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<bool> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let is_member = api
            .is_member(at, group_id, account_id)
            .map_err(convert_error!())?;
        Ok(is_member)
    }

    fn get_group_by_account(
        &self,
        account_id: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<GroupResponse<GroupId, AccountId, MemberCount>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let (group_id, group, members, balance) = api
            .get_group_by_account(at, account_id.clone())
            .map_err(convert_error!())?
            .ok_or(not_found_error!(account_id))?;
        Ok((group_id, group, members, balance).into())
    }

    fn get_group_account(
        &self,
        group_id: GroupId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<AccountId> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let account_id = api
            .get_group_account(at, group_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!(group_id))?;
        Ok(account_id)
    }

    fn get_group(
        &self,
        group_id: GroupId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<GroupResponse<GroupId, AccountId, MemberCount>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let (group, members, balance) = api
            .get_group(at, group_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!(group_id))?;
        Ok((group_id, group, members, balance).into())
    }

    fn get_sub_groups(
        &self,
        group_id: GroupId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<GroupResponse<GroupId, AccountId, MemberCount>>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let groups = api.get_sub_groups(at, group_id).map_err(convert_error!())?;

        Ok(groups
            .into_iter()
            .map(|(sub_group_id, group, members, balance)| {
                (sub_group_id, group, members, balance).into()
            })
            .collect())
    }

    fn get_proposal(
        &self,
        proposal_id: ProposalId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<ProposalResponse<ProposalId, GroupId, AccountId, MemberCount>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let (proposal_id, group_id, members, proposal, votes) = api
            .get_proposal(at, proposal_id)
            .map_err(convert_error!())?
            .ok_or(not_found_error!(proposal_id))?;

        Ok((proposal_id, group_id, members, proposal, votes).into())
    }

    fn get_proposals_by_group(
        &self,
        group_id: GroupId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<ProposalResponse<ProposalId, GroupId, AccountId, MemberCount>>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let proposals = api
            .get_proposals_by_group(at, group_id)
            .map_err(convert_error!())?;

        Ok(proposals
            .into_iter()
            .map(|(proposal_id, group_id, members, proposal, votes)| {
                (proposal_id, group_id, members, proposal, votes).into()
            })
            .collect())
    }

    fn get_proposals_by_account(
        &self,
        account_id: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<
        Vec<(
            GroupId,
            Vec<ProposalResponse<ProposalId, GroupId, AccountId, MemberCount>>,
        )>,
    > {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        let proposals = api
            .get_proposals_by_account(at, account_id)
            .map_err(convert_error!())?;

        Ok(proposals
            .into_iter()
            .map(|(group_id, proposals)| {
                (
                    group_id,
                    proposals
                        .into_iter()
                        .map(|(proposal_id, group_id, members, proposal, votes)| {
                            (proposal_id, group_id, members, proposal, votes).into()
                        })
                        .collect(),
                )
            })
            .collect())
    }
}
