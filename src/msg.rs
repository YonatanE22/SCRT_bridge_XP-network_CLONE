use schemars::{JsonSchema, schema::{SchemaObject, InstanceType, ArrayValidation}};
use secret_toolkit::snip721::ViewerInfo;
use serde::{Deserialize, Serialize};
use borsh::{BorshSerialize, BorshDeserialize};
use cosmwasm_std::{CanonicalAddr, HumanAddr, StdResult, Env};
use serde_big_array::BigArray;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub group_key: [u8; 32],
    pub chain_nonce: u64,
    pub whitelist: Vec<CanonicalAddr>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Sig(
    #[serde(with = "BigArray")]
    pub [u8; 64]
);

impl JsonSchema for Sig {
    fn is_referenceable() -> bool {
        return false;
    }

    fn schema_name() -> String {
        "Array_size_64_of_uint8".into()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            array: Some(Box::new(ArrayValidation {
                items: Some(gen.subschema_for::<u8>().into()),
                max_items: Some(64),
                min_items: Some(64),
                ..Default::default()
            })),
            ..Default::default()
        }.into()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValidatorInfo {
    pub action_id: u128,
    pub sig: Sig
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SetPause(pub bool);

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawFees(pub String);

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SetGroupKey(pub [u8; 32]);

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistNft(pub Vec<u8>);

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintArgs {
    pub minter: String,
    pub minter_hash: String,
    pub token_uri: String,
    pub token_id: String
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValidateTransferNft {
    pub mint_args: MintArgs,
    pub to: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValidateTransferNftBatch {
    pub mint_args: Vec<MintArgs>,
    pub to: String
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnfreezeArgs {
    pub contract: String,
    pub contract_hash: String,
    pub token_id: String
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValidateUnfreezeNft {
    pub unfreeze_args: UnfreezeArgs,
    pub to: String
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ValidateUnfreezeNftBatch {
    pub unfreeze_args: Vec<UnfreezeArgs>,
    pub to: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    SetPause { info: ValidatorInfo, inner: SetPause },
    WithdrawFees { info: ValidatorInfo, inner: WithdrawFees },
    SetGroupKey { info: ValidatorInfo, inner: SetGroupKey },
    WhitelistNft { info: ValidatorInfo, inner: WhitelistNft },
    ValidateTransferNft { info: ValidatorInfo, inner: ValidateTransferNft },
    ValidateTransferNftBatch { info: ValidatorInfo, inner: ValidateTransferNftBatch },
    ValidateUnfreezeNft { info: ValidatorInfo, inner: ValidateUnfreezeNft },
    ValidateUnfreezeNftBatch { info: ValidatorInfo, inner: ValidateUnfreezeNftBatch  },
    FreezeNft { contract: HumanAddr, contract_hash: String, token_id: String, viewer: Option<ViewerInfo>, to: String, chain_nonce: u64, minter: String },
    FreezeNftBatch { contract: HumanAddr, contract_hash: String, token_ids: Vec<String>, viewer: Option<ViewerInfo>, to: String, chain_nonce: u64, minter: String },
    WithdrawNft { burner: HumanAddr, burner_hash: String, token_id: String, viewer: Option<ViewerInfo>, to: String, chain_nonce: u64 },
    WithdrawNftBatch { burner: HumanAddr, burner_hash: String, token_ids: Vec<String>, viewer: Option<ViewerInfo>, to: String, chain_nonce: u64 }
}

#[derive(BorshSerialize, Clone, Debug)]
pub struct BridgeAction<T: BorshSerialize> {
    pub chain_nonce: u64,
    pub sc_addr: String,
    pub action_id: u128,
    pub inner: T,
}

impl<T: BorshSerialize> BridgeAction<T> {
    pub fn new(env: &Env, action_id: u128, chain_nonce: u64, inner: T) -> StdResult<Self> {
        Ok(BridgeAction {
            chain_nonce,
            action_id,
            inner,
            sc_addr: env.contract.address.0.clone()
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetGroupKey,
    GetPaused,
    GetChainNonce,
    GetEventCnt,
    GetWhitelisted { addr: CanonicalAddr },
    GetActionConsumed { action: u128 },
    GetActionConfigConsumed { action: u128 }
}
