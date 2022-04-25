use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Storage, CanonicalAddr, Uint128};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub group_key: [u8; 32],
    pub event_cnt: Uint128,
    pub paused: bool,
    pub chain_nonce: u64
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn action<S: Storage>(storage: &mut S, action: u128) -> Singleton<S, bool> { 
    singleton(storage, &action.to_be_bytes())
}

pub fn action_read<S: Storage>(storage: &S, action: u128) -> ReadonlySingleton<S, bool> {
    singleton_read(storage, &action.to_be_bytes())
}

pub fn action_config<S: Storage>(storage: &mut S, action: u128) -> Singleton<S, bool> {
    singleton(storage, &[CONFIG_KEY, &action.to_be_bytes()].concat())
}

pub fn action_config_read<S: Storage>(storage: &S, action: u128) -> ReadonlySingleton<S, bool> {
    singleton_read(storage, &[CONFIG_KEY, &action.to_be_bytes()].concat())
}

pub fn whitelisted<S: Storage>(storage: &mut S, address: CanonicalAddr) -> Singleton<S, bool> {
    singleton(storage, address.as_slice())
}

pub fn whitelisted_read<S: Storage>(storage: &S, address: CanonicalAddr) -> ReadonlySingleton<S, bool> {
    singleton_read(storage, address.as_slice())
}
