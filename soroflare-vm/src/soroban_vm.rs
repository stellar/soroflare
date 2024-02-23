// Update:
// Leaving the below comments though now with the updates the code is now quite different
// and doesn't have the same logic.

// This file includes a slightly modified version of the soroban-cli invoke command
// (https://github.com/stellar/soroban-tools/blob/35d33ee0c00e6b8bb49df534b9427ed45b080b48/cmd/soroban-cli/src/commands/contract/invoke.rs)

use std::rc::Rc;

use hex::FromHexError;
use soroban_env_host::{
    auth::RecordedAuthPayload, budget::Budget, events::Events, storage::Storage, xdr::{
        AccountId, ContractEvent, Error as XdrError, Hash, HostFunction, InvokeContractArgs, LedgerKey, LedgerKeyAccount, PublicKey, ScAddress, ScSpecEntry, ScSymbol, ScVal, ScVec, StringM, Uint256
    }, Host, HostError
};
use soroban_spec_tools::Spec;

use soroban_spec::read::FromWasmError;
use worker::console_log;
// use worker::console_log;

use crate::soroban_cli::{self};

pub fn deploy(
    src: &[u8],
    state: &mut soroban_ledger_snapshot::LedgerSnapshot,
    contract_id: &[u8; 32],
) -> Result<(), Error> {
    let wasm_hash = soroban_cli::utils::add_contract_code_to_ledger_entries(
        &mut state.ledger_entries,
        src.to_vec(),
        state.min_persistent_entry_ttl,
    )
    .map_err(Error::CannotAddContractToLedgerEntries)?
    .0;

    soroban_cli::utils::add_contract_to_ledger_entries(
        &mut state.ledger_entries,
        *contract_id,
        wasm_hash,
        state.min_persistent_entry_ttl,
    );

    Ok(())
}

pub fn invoke(
    contract_id: &[u8; 32],
    fn_name: &str,
    args: &Vec<ScVal>,
    state: &mut soroban_ledger_snapshot::LedgerSnapshot,
) -> Result<InvocationResult, Error> {
    invoke_with_budget(contract_id, fn_name, args, state, None)
}

pub fn invoke_with_budget(
    contract_id: &[u8; 32],
    fn_name: &str,
    args: &Vec<ScVal>,
    state: &mut soroban_ledger_snapshot::LedgerSnapshot,
    budget: Option<Budget>,
) -> Result<InvocationResult, Error> {
    let budget = budget.unwrap_or_default();

    // Create source account adding it to the ledger.
    // This is a default address, currently further customization is not needed.
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
            (
                Box::new(soroban_cli::utils::default_account_ledger_entry(
                    source_account.clone(),
                )),
                None,
            ),
        ));
    }

    let snap = Rc::new(state.clone());
    let storage = Storage::with_recording_footprint(snap);

    let spec_entries = soroban_cli::utils::get_contract_spec_from_state(&state, *contract_id)
        .map_err(Error::CannotParseContractSpec)?;

    let h = Host::with_storage_and_budget(storage, budget);

    h.set_source_account(source_account)?;
    h.set_base_prng_seed(rand::Rng::gen(&mut rand::thread_rng()))?;

    let mut ledger_info = state.ledger_info();
    ledger_info.sequence_number += 1;
    ledger_info.timestamp += 5;
    h.set_ledger_info(ledger_info.clone())?;

    let (spec, host_function_params) =
        build_host_function_parameters(*contract_id, spec_entries, fn_name, args)?;
    
    h.enable_debug().unwrap();

    // Currently, we rely solely on recording auths.
    h.switch_to_recording_auth(true).unwrap();

    let res = h
        .invoke_function(HostFunction::InvokeContract(host_function_params))
        .map_err(|host_error| {
            if let Some(spec) = spec {            
                if let Ok(error) = spec.find_error_type(host_error.error.get_code()) {
                    Error::ContractInvoke(
                        error.name.to_utf8_string_lossy(),
                        error.doc.to_utf8_string_lossy(),
                    )
                } else {
                    console_log!("{:?}", host_error);
                    host_error.into()
                }
            } else {
                console_log!("spec external {:?}", host_error);
                
                host_error.into()
            }
        })?;

    state.update(&h);
    
    // Note:
    // currently we don't need to deal with auth.

    let budget = h.budget_cloned();
    let auth_payloads = h.get_recorded_auth_payloads()?;
    let (storage, events) = h.try_finish()?;

    let events = events.0.iter().map(|e| {
        e.event.clone()
    }).collect::<Vec<ContractEvent>>();


    Ok(
        InvocationResult {
            result: res,
            storage,
            budget,
            events,
            auth_payloads
        }
    )
}

// a modified version of https://github.com/stellar/soroban-tools/blob/v0.8.0/cmd/soroban-cli/src/commands/contract/invoke.rs#L211-L233
// applied for the new InvokeContractArgs type.
fn build_host_function_parameters(
    contract_id: [u8; 32],
    spec_entries: Option<Vec<ScSpecEntry>>,
    fn_name: &str,
    parsed_args: &Vec<ScVal>,
) -> Result<(Option<Spec>, InvokeContractArgs), Error> {
    let spec = if let Some(spec_entries) = spec_entries {
        Some(Spec(Some(spec_entries)))
    } else {
        None
    };

    // Add the contract ID and the function name to the arguments
    let mut complete_args = vec![];
    complete_args.extend_from_slice(parsed_args.as_slice());
    let complete_args_len = complete_args.len();

    let invoke_args = InvokeContractArgs {
        contract_address: ScAddress::Contract(Hash(contract_id)),
        function_name: ScSymbol(<_ as TryInto<StringM<32>>>::try_into(fn_name).unwrap()),
        args: complete_args
            .try_into()
            .map_err(|_| Error::MaxNumberOfArgumentsReached {
                current: complete_args_len,
                maximum: ScVec::default().max_len(),
            })?,
    };

    Ok((spec, invoke_args))
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Host(#[from] HostError),
    #[error("cannot add contract to ledger entries: {0}")]
    CannotAddContractToLedgerEntries(XdrError),
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
    StrVal(#[from] soroban_spec_tools::Error),
    #[error("function name {0} is too long")]
    FunctionNameTooLong(String),
    #[error("argument count ({current}) surpasses maximum allowed count ({maximum})")]
    MaxNumberOfArgumentsReached { current: usize, maximum: usize },
    #[error("Contract Error\n{0}: {1}")]
    ContractInvoke(String, String),

    #[error("Invalid Snapshot provided")]
    InvalidSnapshot,
}

pub struct InvocationResult {
    pub result: ScVal,
    pub storage: Storage,
    pub budget: Budget,
    pub events: Vec<ContractEvent>,
    pub auth_payloads: Vec<RecordedAuthPayload>
}
