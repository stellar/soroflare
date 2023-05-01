// This file includes a slightly modified version of the soroban-cli invoke command
// (https://github.com/stellar/soroban-tools/blob/01cdac0a03fa04399c392f374f3dae0f91a86039/cmd/soroban-cli/src/commands/contract/invoke.rs)
use std::rc::Rc;

use hex::FromHexError;
use soroban_env_host::{
    budget::Budget,
    events::Events,
    storage::Storage,
    xdr::{
        AccountId, HostFunction, LedgerKey, LedgerKeyAccount, PublicKey, ScBytes,
        ScHostStorageErrorCode, ScSpecEntry, ScStatus, ScVal, ScVec, Uint256,
    },
    Host, HostError,
};
use soroban_spec::read::FromWasmError;

use crate::soroban_cli::{self, strval::Spec};

// https://github.com/stellar/soroban-tools/blob/01cdac0a03fa04399c392f374f3dae0f91a86039/cmd/soroban-cli/src/commands/contract/invoke.rs#L457-L474
pub fn deploy(
    src: &[u8],
    contract_id: &[u8; 32],
    state: &mut soroban_ledger_snapshot::LedgerSnapshot,
) -> Result<(), soroban_env_host::xdr::Error> {
    let wasm_hash = soroban_cli::utils::add_contract_code_to_ledger_entries(
        &mut state.ledger_entries,
        src.to_vec(),
    )?
    .0;

    soroban_cli::utils::add_contract_to_ledger_entries(
        &mut state.ledger_entries,
        *contract_id,
        wasm_hash,
    );

    Ok(())
}

pub fn invoke(
    contract_id: &[u8; 32],
    fn_name: &str,
    args: &Vec<ScVal>,
    state: &mut soroban_ledger_snapshot::LedgerSnapshot,
) -> Result<(ScVal, (Storage, Budget, Events)), Error> {
    invoke_with_budget(contract_id, fn_name, args, state, None)
}

/// "basically" https://github.com/stellar/soroban-tools/blob/01cdac0a03fa04399c392f374f3dae0f91a86039/cmd/soroban-cli/src/commands/contract/invoke.rs#L339-L455
pub fn invoke_with_budget(
    contract_id: &[u8; 32],
    fn_name: &str,
    args: &Vec<ScVal>,
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
            Box::new(soroban_cli::utils::default_account_ledger_entry(
                source_account.clone(),
            )),
        ));
    }

    let snap = Rc::new(state.clone());
    let mut storage = Storage::with_recording_footprint(snap);
    let spec_entries =
        soroban_cli::utils::get_contract_spec_from_storage(&mut storage, *contract_id)
            .map_err(Error::CannotParseContractSpec)?;

    let h = Host::with_storage_and_budget(storage, budget);
    h.set_source_account(source_account);

    //TODO auth_next

    let mut ledger_info = state.ledger_info();
    ledger_info.sequence_number += 1;
    ledger_info.timestamp += 5;
    h.set_ledger_info(ledger_info);

    // // build_host_function_parameters
    // let mut complete_args = vec![
    //     ScVal::Object(Some(ScObject::Bytes(contract_id.try_into().unwrap()))),
    //     ScVal::Symbol(fn_name.try_into().unwrap()),
    // ];

    // complete_args.append(&mut args.to_vec());

    let (_, _, host_function_params) =
        build_host_function_parameters(*contract_id, &spec_entries, fn_name, args)?;

    // let host_function_params: ScVec = complete_args.try_into().unwrap();

    let res = h.invoke_function(HostFunction::InvokeContract(host_function_params))?;

    state.update(&h);

    let (storage, budget, events) = h.try_finish().map_err(|_h| {
        HostError::from(ScStatus::HostStorageError(
            ScHostStorageErrorCode::UnknownError,
        ))
    })?;

    Ok((res, (storage, budget, events)))
}

// https://github.com/stellar/soroban-tools/blob/01cdac0a03fa04399c392f374f3dae0f91a86039/cmd/soroban-cli/src/commands/contract/invoke.rs#LL162C5-L221C6
fn build_host_function_parameters(
    contract_id: [u8; 32],
    spec_entries: &[ScSpecEntry],
    fn_name: &str,
    parsed_args: &Vec<ScVal>,
) -> Result<(String, Spec, ScVec), Error> {
    let spec = Spec(Some(spec_entries.to_vec()));

    // Add the contract ID and the function name to the arguments
    let mut complete_args = vec![
        ScVal::Bytes(ScBytes(contract_id.try_into().unwrap())),
        ScVal::Symbol(
            fn_name
                .try_into()
                .map_err(|_| Error::FunctionNameTooLong(fn_name.to_string()))?,
        ),
    ];
    complete_args.extend_from_slice(parsed_args.as_slice());
    let complete_args_len = complete_args.len();

    Ok((
        fn_name.to_string(),
        spec,
        complete_args
            .try_into()
            .map_err(|_| Error::MaxNumberOfArgumentsReached {
                current: complete_args_len,
                maximum: ScVec::default().max_len(),
            })?,
    ))
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Host(#[from] HostError),
    #[error("cannot parse contract ID {contract_id}: {error}")]
    CannotParseContractId {
        contract_id: String,
        error: FromHexError,
    },
    #[error("function {0} was not found in the contract")]
    FunctionNotFoundInContractSpec(String),
    #[error("parsing contract spec: {0}")]
    CannotParseContractSpec(FromWasmError),
    #[error(transparent)]
    StrVal(#[from] soroban_cli::strval::Error),
    #[error("function name {0} is too long")]
    FunctionNameTooLong(String),
    #[error("argument count ({current}) surpasses maximum allowed count ({maximum})")]
    MaxNumberOfArgumentsReached { current: usize, maximum: usize },
}
