use crate::contract as contract;
use crate::msg::{InitMsg, QueryMsg, HandleMsg, BridgeAction, SetPause, Sig, ValidatorInfo, SetGroupKey, WhitelistNft};
use borsh::BorshSerialize;
use rand_core::OsRng;
use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi, MockQuerier};
use cosmwasm_std::{to_binary, InitResponse, Env, Extern, MemoryStorage, CanonicalAddr, Binary};
use ed25519_dalek::{Keypair, PublicKey, ExpandedSecretKey};
use sha2::{Sha512, Digest};

const CHAIN_NONCE: u64 = 1;
const ACTION_ID: u128 = 1;

fn ed25519_kp() -> Keypair {
    Keypair::generate(&mut OsRng)
}

fn init_func(pubk: PublicKey, deps: &mut Extern<MemoryStorage, MockApi, MockQuerier>, env: Env) -> InitResponse {
    let msg = InitMsg { group_key: pubk.to_bytes(), chain_nonce: CHAIN_NONCE, whitelist: Vec::new() };

    let res = contract::init(deps, env, msg).unwrap();
    return res;
}

fn gen_sig(kp: &Keypair, env: &Env, action_id: u128, context: &[u8], inner: impl BorshSerialize) -> ValidatorInfo {
    let secret: ExpandedSecretKey = (&kp.secret).into();
    let act = BridgeAction::new(&env, action_id, CHAIN_NONCE, inner).unwrap();
    
    let mut hasher = Sha512::new();
    hasher.update(context);
    hasher.update(act.try_to_vec().unwrap());
    let dig = hasher.finalize();

    let sig = Sig(secret.sign(&dig, &kp.public).to_bytes());

    return ValidatorInfo {
        action_id,
        sig
    }
}

#[test]
fn proper_initialization() {
    let kp = ed25519_kp();
    let env = mock_env("creator", &[]);
    let mut deps = mock_dependencies(20, &[]);

    let res = init_func(kp.public.clone(), &mut deps, env);
    assert_eq!(0, res.messages.len());

    let res = contract::query(&deps, QueryMsg::GetPaused).unwrap();
    assert_eq!(res, to_binary(&false).unwrap());

    let res = contract::query(&deps, QueryMsg::GetGroupKey).unwrap();
    assert_eq!(res, to_binary(&kp.public.to_bytes()).unwrap())
}

#[test]
fn set_pause() {
    let kp = ed25519_kp();
    let env = mock_env("creator", &[]);
    let mut deps = mock_dependencies(20, &[]);

    init_func(kp.public.clone(), &mut deps, env.clone());

    let inner = SetPause(true);
    let info = gen_sig(&kp, &env, ACTION_ID, b"SetPause", inner.clone());
    contract::handle(&mut deps, env, HandleMsg::SetPause { info, inner }).unwrap();

    let res = contract::query(&deps, QueryMsg::GetPaused).unwrap();
    assert_eq!(res, to_binary(&true).unwrap());
}

#[test]
fn set_gk() {
    let kp = ed25519_kp();
    let env = mock_env("creator", &[]);
    let mut deps = mock_dependencies(20, &[]);

    init_func(kp.public.clone(), &mut deps, env.clone());

    let kp2 = ed25519_kp();
    let inner = SetGroupKey(kp2.public.clone().to_bytes());
    let info = gen_sig(&kp, &env, ACTION_ID, b"SetGroupKey", inner.clone());
    contract::handle(&mut deps, env, HandleMsg::SetGroupKey { info, inner }).unwrap();

    let res = contract::query(&deps, QueryMsg::GetGroupKey).unwrap();
    assert_eq!(res, to_binary(kp2.public.as_bytes()).unwrap());
}

#[test]
fn add_whitelist() {
    let kp = ed25519_kp();
    let env = mock_env("creator", &[]);
    let mut deps = mock_dependencies(20, &[]);

    init_func(kp.public.clone(), &mut deps, env.clone());

    let addr = CanonicalAddr(Binary(vec![1; 20]));
    let inner = WhitelistNft(addr.clone().0.0);
    let info = gen_sig(&kp, &env, ACTION_ID, b"WhitelistNft", inner.clone());
    contract::handle(&mut deps, env, HandleMsg::WhitelistNft { info, inner }).unwrap();

    let res = contract::query(&deps, QueryMsg::GetWhitelisted { addr }).unwrap();
    assert_eq!(res, to_binary(&true).unwrap());
}
