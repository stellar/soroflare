// This file includes a slightly modified version of the soroban-cli invoke command
// (https://github.com/stellar/soroban-tools/blob/638fa06de23fcf7e0cab3eb8ed0dcf22e3d6d367/cmd/soroban-cli/src/contract/invoke.rs)
use std::rc::Rc;

use soroban_env_host::{
    budget::Budget,
    events::Events,
    storage::Storage,
    xdr::{
        AccountId, HostFunction, LedgerKey, LedgerKeyAccount, PublicKey, ScHostStorageErrorCode,
        ScObject, ScStatus, ScVal, ScVec, Uint256,
    },
    Host, HostError,
};

use super::soroban_env_utils;

// https://github.com/stellar/soroban-tools/blob/638fa06de23fcf7e0cab3eb8ed0dcf22e3d6d367/cmd/soroban-cli/src/contract/invoke.rs#L429-L446
pub fn deploy(
    src: &[u8],
    contract_id: &[u8; 32],
    state: &mut soroban_ledger_snapshot::LedgerSnapshot,
) -> Result<(), soroban_env_host::xdr::Error> {
    let wasm_hash = soroban_env_utils::add_contract_code_to_ledger_entries(
        &mut state.ledger_entries,
        src.to_vec(),
    )?
    .0;

    soroban_env_utils::add_contract_to_ledger_entries(
        &mut state.ledger_entries,
        *contract_id,
        wasm_hash,
    );

    Ok(())
}

pub fn invoke(
    contract_id: &[u8; 32],
    fn_name: &str,
    args: &[ScVal],
    state: &mut soroban_ledger_snapshot::LedgerSnapshot,
) -> Result<(ScVal, (Storage, Budget, Events)), Error> {
    invoke_with_budget(contract_id, fn_name, args, state, None)
}

/// "basically" https://github.com/stellar/soroban-tools/blob/638fa06de23fcf7e0cab3eb8ed0dcf22e3d6d367/cmd/soroban-cli/src/contract/invoke.rs#L311-L427
pub fn invoke_with_budget(
    contract_id: &[u8; 32],
    fn_name: &str,
    args: &[ScVal],
    state: &mut soroban_ledger_snapshot::LedgerSnapshot,
    budget: Option<Budget>,
) -> Result<(ScVal, (Storage, Budget, Events)), Error> {
    let budget = budget.unwrap_or_default();

    // Create source account, adding it to the ledger if not already present.
    let source_account = AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(
        stellar_strkey::ed25519::PublicKey::from_string(
            "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
        )
        .unwrap()
        .0,
    )));
    let source_account_ledger_key = LedgerKey::Account(LedgerKeyAccount {
        account_id: source_account.clone(),
    });
    if !state
        .ledger_entries
        .iter()
        .any(|(k, _)| **k == source_account_ledger_key)
    {
        state.ledger_entries.push((
            Box::new(source_account_ledger_key),
            Box::new(soroban_env_utils::default_account_ledger_entry(source_account.clone())),
        ));
    }

    let snap = Rc::new(state.clone());
    let storage = Storage::with_recording_footprint(snap);

    let h = Host::with_storage_and_budget(storage, budget);
    h.set_source_account(source_account);

    let mut ledger_info = state.ledger_info();
    ledger_info.sequence_number += 1;
    ledger_info.timestamp += 5;
    h.set_ledger_info(ledger_info);

    let mut complete_args = vec![
        ScVal::Object(Some(ScObject::Bytes(contract_id.try_into().unwrap()))),
        ScVal::Symbol(fn_name.try_into().unwrap()),
    ];

    complete_args.append(&mut args.to_vec());

    let host_function_params: ScVec = complete_args.try_into().unwrap();

    let res = h.invoke_function(HostFunction::InvokeContract(host_function_params))?;

    state.update(&h);

    let (storage, budget, events) = h.try_finish().map_err(|_h| {
        HostError::from(ScStatus::HostStorageError(
            ScHostStorageErrorCode::UnknownError,
        ))
    })?;

    Ok((res, (storage, budget, events)))
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Host(#[from] HostError),
}
