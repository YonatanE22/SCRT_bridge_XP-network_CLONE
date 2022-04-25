use borsh::BorshSerialize;
use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdError, StdResult, Storage, BankMsg, LogAttribute, HumanAddr, Uint128, CanonicalAddr,
};
use cosmwasm_storage::Singleton;
use secret_toolkit::snip721::{nft_dossier_query, transfer_nft_msg, mint_nft_msg, Metadata, burn_nft_msg, Transfer, batch_transfer_nft_msg, Burn, batch_burn_nft_msg};
use sha2::{Sha512, Digest};

use crate::events::{BridgeEventInfo, TransferSnip721, UnfreezeSnip721, TransferInfo, TransferSnip721Batch, UnfreezeSnip721Batch};
use crate::msg::{HandleMsg, InitMsg, QueryMsg, BridgeAction, ValidatorInfo};
use crate::state::{config, config_read, State, action_read, action_config_read, action, action_config, whitelisted_read, whitelisted};
use ed25519_compact::{PublicKey, Signature};

// TODO: confirm if this value is correct
const BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        group_key: msg.group_key,
        chain_nonce: msg.chain_nonce,
        event_cnt: Uint128(0),
        paused: false
    };

    config(&mut deps.storage).save(&state)?;

    for contract in msg.whitelist {
        whitelisted(&mut deps.storage, contract).save(&true)?;
    }

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse::default())
}

fn require_sig_i<S: Storage>(
    mut store: Singleton<S, bool>,
    env: &Env,
    state: State,
    info: ValidatorInfo,
    context: &[u8],
    inner: impl BorshSerialize,
) -> StdResult<()> {
    if store.load().unwrap_or(false) {
        return Err(StdError::generic_err("duplicate action"));
    }
    store.save(&true)?;

    let action = BridgeAction::new(env, info.action_id, state.chain_nonce, inner)?;
    let raw_act = action.try_to_vec().map_err(|e|
        StdError::serialize_err("borsh", e.to_string())
    )?;

    let mut hasher = Sha512::new();
    hasher.update(context);
    hasher.update(raw_act);
    let hash = hasher.finalize();

    let sig = Signature::new(info.sig.0);
    let key = PublicKey::new(state.group_key);
    key.verify(hash, &sig).map_err(|_| StdError::unauthorized())?;

    Ok(())
}

fn require_sig<S: Storage>(
    storage: &mut S,
    env: &Env,
    state: State,
    info: ValidatorInfo,
    context: &[u8],
    inner: impl BorshSerialize,
) -> StdResult<()> {
    require_sig_i(action(storage, info.action_id), env, state, info, context, inner)
}

fn require_sig_config<S: Storage>(
    storage: &mut S,
    env: &Env,
    state: State,
    info: ValidatorInfo,
    context: &[u8],
    inner: impl BorshSerialize,
) -> StdResult<()> {
    require_sig_i(action_config(storage, info.action_id), env, state, info, context, inner)
}

fn action_id<S: Storage>(
    store: &mut Singleton<S, State>,
    state: &mut State
) -> StdResult<u128> {
    let cnt = state.event_cnt.0;
    let ret = Ok(cnt);
    state.event_cnt = Uint128(cnt+1);
    store.save(&state)?;

    return ret;
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    let Extern { storage, api, querier } = deps;
    let mut store = config(storage);
    let mut state = store.load()?;

    let require_unpause = |state: &State| if state.paused {
        return Err(StdError::unauthorized())
    } else {
        Ok(())
    };

    let tx_fee = |env: &Env| {
        env.message.sent_funds
            .iter()
            .find(|c| c.denom == "SCRT")
            .map(|c| c.amount.u128())
            .ok_or_else(|| StdError::generic_err("TX Fees required!"))
    };

    let require_whitelist = |store: &S,addr: &HumanAddr| {
        if !whitelisted_read(store, api.canonical_address(addr)?).load().unwrap_or(false) {
            Err(StdError::unauthorized())
        } else {
            Ok(())
        }
    };

    match msg {
        HandleMsg::SetPause { info, inner } => {
            state.paused = inner.0;
            store.save(&state)?;

            require_sig_config(storage, &env, state, info, b"SetPause", inner)?;
        },
        HandleMsg::SetGroupKey { info, inner } => {
            require_unpause(&state)?;
            let old_state = state.clone();
            state.group_key = inner.0;

            require_sig_config(storage, &env, old_state, info, b"SetGroupKey", inner)?;
            let mut store = config(storage);
            store.save(&state)?;
        },
        HandleMsg::WithdrawFees { info, inner } => {
            require_unpause(&state)?;
            let contract_addr = env.contract.address.clone();
            let bal = querier.query_balance(&contract_addr, "SCRT")?;

            let bank_msg = BankMsg::Send {
                from_address: contract_addr,
                to_address: HumanAddr(inner.0.clone()),
                amount: vec![bal]
            }; 

            require_sig_config(storage, &env, state, info, b"WithdrawFees", inner)?;

            return Ok(HandleResponse {
                messages: vec![bank_msg.into()],
                log: vec![],
                data: None
            });
        },
        HandleMsg::WhitelistNft { info, inner } => {
            require_unpause(&state)?;
            whitelisted(storage, CanonicalAddr(Binary(inner.0.clone()))).save(&true)?;
            require_sig_config(storage, &env, state, info, b"WhitelistNft", inner)?;
        }
        HandleMsg::ValidateUnfreezeNft { info, inner } => {
            require_unpause(&state)?;

            let transfer = transfer_nft_msg(
                HumanAddr(inner.to.clone()),
                inner.unfreeze_args.token_id.clone(),
                None,
                None,
                BLOCK_SIZE,
                inner.unfreeze_args.contract_hash.clone(),
                HumanAddr(inner.unfreeze_args.contract.clone())
            )?;

            require_sig(storage, &env, state, info, b"ValidateUnfreezeNft", inner)?;

            return Ok(HandleResponse {
                messages: vec![transfer],
                log: vec![],
                data: None
            });
        }
        HandleMsg::ValidateUnfreezeNftBatch { info, inner } => {
            require_unpause(&state)?;

            let messages = inner.unfreeze_args.clone().into_iter().map(|a| transfer_nft_msg(
                HumanAddr(inner.to.clone()),
                a.token_id,
                None,
                None,
                BLOCK_SIZE,
                a.contract_hash,
                HumanAddr(a.contract)
            )).collect::<Result<Vec<_>, _>>()?;

            require_sig(storage, &env, state, info, b"ValidateUnfreezeNftBatch", inner)?;

            return Ok(HandleResponse {
                messages,
                log: vec![],
                data: None
            });
        }
        HandleMsg::ValidateTransferNft { info, inner } => {
            require_unpause(&state)?;

            let mint = mint_nft_msg(
                Some(inner.mint_args.token_id.clone()),
                Some(HumanAddr(inner.to.clone())),
                Some(Metadata {
                    token_uri: Some(inner.mint_args.token_uri.clone()),
                    extension: None
                }),
                None,
                None,
                None,
                BLOCK_SIZE,
                inner.mint_args.minter_hash.clone(),
                HumanAddr(inner.mint_args.minter.clone())
            )?;

            require_sig(storage, &env, state, info, b"ValidateTransferNft", inner)?;

            return Ok(HandleResponse {
                messages: vec![mint],
                log: vec![],
                data: None
            });
        }
        HandleMsg::ValidateTransferNftBatch { info, inner } => {
            require_unpause(&state)?;

            let messages = inner.mint_args.clone().into_iter().map(|a| mint_nft_msg(
                Some(a.token_id),
                Some(HumanAddr(inner.to.clone())),
                Some(Metadata {
                    token_uri: Some(a.token_uri),
                    extension: None
                }),
                None,
                None,
                None,
                BLOCK_SIZE,
                a.minter_hash,
                HumanAddr(a.minter)
            )).collect::<Result<Vec<_>, _>>()?;

            require_sig(storage, &env, state, info, b"ValidateTransferNftBatch", inner)?;

            return Ok(HandleResponse {
                messages,
                log: vec![],
                data: None
            });

        }
        HandleMsg::FreezeNft { contract, contract_hash, token_id, viewer, to, chain_nonce, minter } => {
            require_unpause(&state)?;
            let our_addr = env.contract.address.clone();
            let act_id = action_id(&mut store, &mut state)?;
            let fee = tx_fee(&env)?;

            require_whitelist(&storage, &contract)?;


            let nft_dat = nft_dossier_query(
                querier,
                token_id.clone(),
                viewer.clone(),
                None,
                BLOCK_SIZE,
                contract_hash.clone(),
                contract.clone()
            )?;

            let log: Vec<LogAttribute> = vec![
                BridgeEventInfo::new(act_id, chain_nonce, fee, to.clone())
                    .try_into()?,
                TransferSnip721 {
                    info: TransferInfo {
                        public_metadata: nft_dat.public_metadata,
                        private_metadata: nft_dat.private_metadata,
                        token_id: token_id.clone()
                    },
                    contract_addr: contract.clone(),
                    contract_hash: contract_hash.clone(),
                    mint_with: minter
                }.try_into()?
            ];

            let transfer = transfer_nft_msg(
                our_addr,
                token_id.clone(),
                None,
                None,
                BLOCK_SIZE,
                contract_hash.clone(),
                contract.clone()
            )?;

            return Ok(HandleResponse {
                messages: vec![transfer],
                log,
                data: None
            })
        },
        HandleMsg::FreezeNftBatch { contract, contract_hash, token_ids, viewer, to, chain_nonce, minter } => {
            require_unpause(&state)?;
            let our_addr = env.contract.address.clone();
            let act_id = action_id(&mut store, &mut state)?;
            let fee = tx_fee(&env)?;

            require_whitelist(&storage, &contract)?;

            let transfers = Transfer { token_ids: token_ids.clone(), memo: None, recipient: our_addr };
            let transfer_infos = token_ids.into_iter().map(|tok| {
                let nft_dat = nft_dossier_query(
                    querier,
                    tok.clone(),
                    viewer.clone(),
                    None,
                    BLOCK_SIZE,
                    contract_hash.clone(),
                    contract.clone()
                )?;

                return Ok(TransferInfo {
                    public_metadata: nft_dat.public_metadata,
                    private_metadata: nft_dat.private_metadata,
                    token_id: tok,
                });
            }).collect::<Result<Vec<_>, _>>()?;

            let transfer = batch_transfer_nft_msg(
                vec![transfers],
                None,
                BLOCK_SIZE,
                contract_hash.clone(),
                contract.clone()
            )?;

            let log: Vec<LogAttribute> = vec![
                BridgeEventInfo::new(act_id, chain_nonce, fee, to).try_into()?,
                TransferSnip721Batch {
                    infos: transfer_infos,
                    contract_addr: contract,
                    contract_hash: contract_hash,
                    mint_with: minter
                }.try_into()?
            ];

            return Ok(HandleResponse {
                messages: vec![transfer],
                log,
                data: None
            });
        }
        HandleMsg::WithdrawNft { burner, burner_hash, viewer, token_id, to, chain_nonce } => {
            require_unpause(&state)?;

            let act_id = action_id(&mut store, &mut state)?;
            let fee = tx_fee(&env)?;

            let nft_dat = nft_dossier_query(
                querier,
                token_id.clone(),
                viewer.clone(), 
                None,
                BLOCK_SIZE,
                burner_hash.clone(),
                burner.clone()
            )?;
            let token_uri = nft_dat.public_metadata
                .map(|m| m.token_uri)
                .flatten()
                .ok_or_else(|| StdError::unauthorized())?;

            let log: Vec<LogAttribute> = vec![
                BridgeEventInfo::new(act_id, chain_nonce, fee, to.clone())
                    .try_into()?,
                UnfreezeSnip721 {
                    token_uri: token_uri,
                    burner: burner.clone()
                }.try_into()?
            ];

            let burn = burn_nft_msg(
                token_id.clone(),
                None,
                None,
                256,
                burner_hash.clone(),
                burner.clone()
            )?;

            return Ok(HandleResponse {
                messages: vec![burn],
                log,
                data: None
            })
        },
        HandleMsg::WithdrawNftBatch { burner, burner_hash, token_ids, viewer, to, chain_nonce } => {
            require_unpause(&state)?;

            let act_id = action_id(&mut store, &mut state)?;
            let fee = tx_fee(&env)?;

            let burns = Burn { token_ids: token_ids.clone(), memo: None };
            let token_uris = token_ids.into_iter().map(|tok| {
               let nft_dat = nft_dossier_query(
                    querier,
                    tok.clone(),
                    viewer.clone(),
                    None,
                    BLOCK_SIZE,
                    burner_hash.clone(),
                    burner.clone()
                )?;

                return nft_dat.public_metadata
                    .map(|m| m.token_uri)
                    .flatten()
                    .ok_or_else(|| StdError::unauthorized());
            }).collect::<Result<Vec<_>, _>>()?;

            let burn = batch_burn_nft_msg(
                vec![burns],
                None,
                BLOCK_SIZE,
                burner_hash.clone(),
                burner.clone()
            )?;

            let log: Vec<LogAttribute> = vec![
                BridgeEventInfo::new(act_id, chain_nonce, fee, to.clone())
                    .try_into()?,
                UnfreezeSnip721Batch {
                    token_uris,
                    burner
                }.try_into()?
            ];

            return Ok(HandleResponse {
                messages: vec![burn],
                log,
                data: None
            })
        }
    }

    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    let config = config_read(&deps.storage).load()?;

    return match msg {
        QueryMsg::GetPaused => { to_binary(&config.paused) },
        QueryMsg::GetGroupKey => { to_binary(&config.group_key) },
        QueryMsg::GetChainNonce => { to_binary(&config.chain_nonce) },
        QueryMsg::GetEventCnt => { to_binary(&config.event_cnt) },
        QueryMsg::GetWhitelisted { addr } => {
            to_binary(&whitelisted_read(&deps.storage, addr).load()?)
        }
        QueryMsg::GetActionConsumed { action } => {
            to_binary(&action_read(&deps.storage, action).load()?)
        },
        QueryMsg::GetActionConfigConsumed { action } => {
            to_binary(&action_config_read(&deps.storage, action).load()?)
        }
    };
}
