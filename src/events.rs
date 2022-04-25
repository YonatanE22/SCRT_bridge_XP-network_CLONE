use std::convert::TryInto;

use cosmwasm_std::{HumanAddr, LogAttribute, log, StdError, StdResult};
use secret_toolkit::snip721::Metadata;
use serde::Serialize;

fn to_log_attr<T: Serialize>(name: &str, e: &T) -> StdResult<LogAttribute> {
    let ser = serde_json_wasm::to_string(e)
        .map_err(|e| StdError::serialize_err("serde-json-wasm", e))?;

    Ok(log(
            name,
            ser
    ))
}

macro_rules! bridge_event {
    ($ev:ident) => {
        impl TryInto<LogAttribute> for $ev {
            type Error = StdError;

            fn try_into(self) -> StdResult<LogAttribute> {
                to_log_attr(stringify!($struct), &self)
            }
        }
    };
}

#[derive(Debug, Serialize)]
pub struct BridgeEventInfo {
    pub action_id: u128,
    pub chain_nonce: u64,
    pub tx_fees: u128,
    pub to: String
}
bridge_event!(BridgeEventInfo);

impl BridgeEventInfo {
    pub fn new(action_id: u128, chain_nonce: u64, tx_fees: u128, to: String) -> Self {
        Self {
            action_id,
            chain_nonce,
            tx_fees,
            to
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TransferSnip721 {
    pub contract_addr: HumanAddr,
    pub contract_hash: String,
    pub mint_with: String,
    pub info: TransferInfo
}
bridge_event!(TransferSnip721);

#[derive(Debug, Serialize)]
pub struct TransferInfo {
    pub public_metadata: Option<Metadata>,
    pub private_metadata: Option<Metadata>,
    pub token_id: String
}

#[derive(Debug, Serialize)]
pub struct TransferSnip721Batch {
   pub infos: Vec<TransferInfo>,
   pub contract_addr: HumanAddr,
   pub contract_hash: String,
   pub mint_with: String
}
bridge_event!(TransferSnip721Batch);

#[derive(Debug, Serialize)]
pub struct UnfreezeSnip721 {
    pub token_uri: String,
    pub burner: HumanAddr
}
bridge_event!(UnfreezeSnip721);

#[derive(Debug, Serialize)]
pub struct UnfreezeSnip721Batch {
    pub token_uris: Vec<String>,
    pub burner: HumanAddr
}
bridge_event!(UnfreezeSnip721Batch);
