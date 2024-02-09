use std::collections::HashMap;

use crate::{
    response::{BasicJsonResponse, JsonResponse},
    TaskRegistry,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use soroban_env_host::{
    budget::Budget,
    xdr::{
        BytesM, ContractCodeEntry, ContractDataDurability, ContractDataEntry, ContractExecutable,
        ExtensionPoint, Hash, LedgerEntry, LedgerEntryData, LedgerEntryExt, LedgerKey,
        LedgerKeyContractCode, LedgerKeyContractData, Limits, ReadXdr, ScAddress,
        ScContractInstance, ScSymbol, ScVal, ScVec, VecM, WriteXdr,
    },
};
use soroflare_vm::{
    contract_id, soroban_vm,
    soroflare_utils::{self, EntryWithLifetime},
};
use worker::{console_log, kv::KvStore, Request, Response, RouteContext};

#[derive(Serialize)]
pub struct ExecutionResult {
    cpu: u64,
    mem: u64,
    result: String,
}

#[derive(Deserialize, Serialize)]
pub struct WithSnapshotInput {
    ledger_sequence: u32,
    keys: Vec<LedgerKey>,
    vals: Vec<EntryWithLifetime>,
    contract_id: [u8; 32],
    fname: String,
    params: Vec<ScVal>,
}

pub struct Generic;

impl Generic {
    async fn run_with_snapshot(
        req: &mut Request,
        modules: KvStore,
    ) -> Result<ExecutionResult, Result<Response, worker::Error>> {
        let WithSnapshotInput {
            ledger_sequence,
            keys,
            vals,
            contract_id,
            fname,
            params,
        } = req.json().await.unwrap();

        // todo: group all simulation-related errors in a specific errors enum and implement conversions
        // to the RPC API for it.
        let mut wasm_hashes = Vec::new();
        for val in &vals {
            if !val.is_live(ledger_sequence) {
                return Err(JsonResponse::new("Entry is expired", 400).with_opt(val).into())
            }
            if let LedgerEntryData::ContractData(contract_data) = &val.entry.data {
                if let ScVal::ContractInstance(instance) = &contract_data.val {
                    if let ContractExecutable::Wasm(hash) = &instance.executable {
                        wasm_hashes.push(hash.0)
                    }
                }
            }
        }

        let mut inferred_keys = Vec::new();
        let mut inferred_vals = Vec::new();

        for hash in wasm_hashes {
            if !keys.iter().any(|key| {
                if let LedgerKey::ContractCode(code) = key {
                    code.hash.0 == hash
                } else {
                    false
                }
            }) {
                let hex_hash = hex::encode(hash);
                if let Ok(module) = modules.get(&hex_hash).text().await {
                    if let Some(module) = module {
                        let module = hex::decode(module).unwrap();

                        let key =
                            LedgerKey::ContractCode(LedgerKeyContractCode { hash: Hash(hash) });
                        let val = EntryWithLifetime {
                            entry: LedgerEntry {
                                last_modified_ledger_seq: 0,
                                data: LedgerEntryData::ContractCode(ContractCodeEntry {
                                    ext: ExtensionPoint::V0,
                                    hash: Hash(hash),
                                    code: BytesM::try_from(module).unwrap(),
                                }),
                                ext: LedgerEntryExt::V0,
                            },
                            live_until: Some(u32::MAX),
                        };

                        inferred_keys.push(key);
                        inferred_vals.push(val);
                    } else {
                        return Err(
                            JsonResponse::new("Wasm was not installed on soroflare", 400)
                                .with_opt(hex_hash)
                                .into(),
                        );
                    }
                } else {
                    return Err(
                        JsonResponse::new("Internal error when executing KV query", 400)
                            .with_opt(hex_hash)
                            .into(),
                    );
                };
            }
        }

        inferred_keys.extend(keys);
        inferred_vals.extend(vals);

        let mut state = soroflare_utils::ledger_snapshot_from_entries_and_ledger(
            ledger_sequence,
            inferred_keys,
            inferred_vals,
        )
        .map_err(|e: soroban_vm::Error| -> Result<Response, worker::Error> {
            match e {
                soroflare_vm::soroban_vm::Error::InvalidSnapshot => {
                    JsonResponse::new("Invalid snapshot provided", 400)
                        .with_opt(e.to_string())
                        .into()
                }
                _ => JsonResponse::new("Unknown issue, please file a bug report.", 400)
                    .with_opt(e.to_string())
                    .into(),
            }
        })?;

        let advanced_budget = Budget::default();

        let execution_result = soroban_vm::invoke_with_budget(
            &contract_id,
            &fname,
            &params,
            &mut state,
            Some(advanced_budget),
        );

        if let Ok(res) = execution_result {
            let (scval_result, (_, user_solve_budget, _)) = res;
            let cpu = user_solve_budget.get_cpu_insns_consumed().unwrap();
            let mem = user_solve_budget.get_mem_bytes_consumed().unwrap();

            let result = scval_result.to_xdr_base64(Limits::none()).unwrap();

            Ok(ExecutionResult { cpu, mem, result })
        } else {
            return Err(JsonResponse::new("Failed to execute contract", 400)
                .with_opt(execution_result.err().unwrap().to_string())
                .into());
        }
    }

    fn run(
        raw_wasm: &[u8],
        req: &Request,
    ) -> Result<ExecutionResult, Result<Response, worker::Error>> {
        let get_query: HashMap<_, _> = req
            .url()
            .map_err(|err| return Err::<Response, worker::Error>(err.into()))?
            .query_pairs()
            .into_owned()
            .collect();

        let fname = if let Some(fname) = get_query.get("fname") {
            fname
        } else {
            return Err(BasicJsonResponse::new("No fname", 400).into());
        };
        let params = if let Some(xdr) = get_query.get("params") {
            if let Ok(ScVal::Vec(Some(vec))) = ScVal::from_xdr_base64(xdr, Limits::none()) {
                vec
            } else {
                return Err(BasicJsonResponse::new("Invalid params", 400).into());
            }
        } else {
            return Err(BasicJsonResponse::new("No params", 400).into());
        };

        let mut state = soroflare_utils::empty_ledger_snapshot();
        let contract_id = contract_id!(0);

        if let Err(e) = soroban_vm::deploy(raw_wasm, &mut state, &contract_id) {
            return Err(JsonResponse::new("Failed to deploy user contract", 500)
                .with_opt(e.to_string())
                .into());
        }

        //let advanced_budget = Budget::try_from_configs(u64::MAX, u64::MAX, ContractCostParams::default(), ContractCostParams::default()).unwrap();
        let advanced_budget = Budget::default();

        let execution_result = soroban_vm::invoke_with_budget(
            &contract_id,
            fname,
            &params,
            &mut state,
            Some(advanced_budget),
        );

        if let Ok(res) = execution_result {
            let (scval_result, (_, user_solve_budget, _)) = res;
            let cpu = user_solve_budget.get_cpu_insns_consumed().unwrap();
            let mem = user_solve_budget.get_mem_bytes_consumed().unwrap();

            let result = scval_result.to_xdr_base64(Limits::none()).unwrap();

            Ok(ExecutionResult { cpu, mem, result })
        } else {
            return Err(JsonResponse::new("Failed to execute contract", 400)
                .with_opt(execution_result.err().unwrap().to_string())
                .into());
        }
    }
}

pub async fn handle(
    mut req: Request,
    _: RouteContext<TaskRegistry<'_>>,
) -> Result<Response, worker::Error> {
    let data = if let Ok(raw) = req.bytes().await {
        raw
    } else {
        return BasicJsonResponse::new("Error reading submitted data in body", 400).into();
    };

    // validate WASM magic word
    if data.len() <= 4
        || !(data[0] == 0x00 && data[1] == 0x61 && data[2] == 0x73 && data[3] == 0x6d)
    {
        return BasicJsonResponse::new("Submitted data does not contain valid WASM code", 400)
            .into();
    }

    let result = Generic::run(&data, &req);

    if let Err(err) = result {
        return err;
    }

    if let Ok(execution) = result {
        JsonResponse::new("Successful execution", 200)
            .with_opt(execution)
            .into()
    } else {
        result.err().unwrap()
    }
}

pub async fn handle_upload(
    mut req: Request,
    ctx: RouteContext<TaskRegistry<'_>>,
) -> Result<Response, worker::Error> {
    let data = if let Ok(raw) = req.bytes().await {
        raw
    } else {
        return BasicJsonResponse::new("Error reading submitted data in body", 400).into();
    };

    // validate WASM magic word
    if data.len() <= 4
        || !(data[0] == 0x00 && data[1] == 0x61 && data[2] == 0x73 && data[3] == 0x6d)
    {
        return BasicJsonResponse::new("Submitted data does not contain valid WASM code", 400)
            .into();
    };

    let modules = ctx.kv("MODULES").unwrap();
    let hash: [u8; 32] = Sha256::digest(data.as_slice()).into();

    let _ = modules
        .put(&hex::encode(hash), hex::encode(data))
        .unwrap()
        .execute()
        .await
        .unwrap();

    JsonResponse::new("Successfully uploaded wasm", 200)
        .with_opt(hex::encode(hash))
        .into()
}

pub async fn handle_snapshot(
    mut req: Request,
    ctx: RouteContext<TaskRegistry<'_>>,
) -> Result<Response, worker::Error> {
    let modules = ctx.kv("MODULES").unwrap();

    let result = Generic::run_with_snapshot(&mut req, modules).await;

    if let Err(err) = result {
        return err;
    }

    if let Ok(execution) = result {
        JsonResponse::new("Successful execution", 200)
            .with_opt(execution)
            .into()
    } else {
        result.err().unwrap()
    }
}

#[test]
fn generate_snapshot_request() {
    let symbol = ScVal::Symbol(ScSymbol("tdep".to_string().try_into().unwrap()));

    let binary = hex::decode("0061736d01000000010f0360027e7e017e60017e017e60000002070101760167000003030201020405017001010105030100100619037f01418080c0000b7f00418080c0000b7f00418080c0000b073105066d656d6f727902000568656c6c6f0001015f00020a5f5f646174615f656e6403010b5f5f686561705f6261736503020ac80102c20101027f23808080800041206b2201248080808000024002402000a741ff01712202410e460d00200241ca00470d010b200120003703082001428ee8f1d8ba02370300410021020340024020024110470d00410021020240034020024110460d01200141106a20026a200120026a290300370300200241086a21020c000b0b200141106aad4220864204844284808080201080808080002100200141206a24808080800020000f0b200141106a20026a4202370300200241086a21020c000b0b00000b02000b00430e636f6e747261637473706563763000000000000000000000000568656c6c6f000000000000010000000000000002746f00000000001100000001000003ea00000011001e11636f6e7472616374656e766d657461763000000000000000140000000000770e636f6e74726163746d6574617630000000000000000572737665720000000000000e312e37362e302d6e696768746c7900000000000000000008727373646b7665720000002f32302e302e30233832326365366363336534363163636339323532373562343732643737623663613335623263643900").unwrap();
    println!("{:?}", binary);
    let hash = Hash(Sha256::digest([0; 32].as_slice()).into());
    let code_key = LedgerKey::ContractCode(LedgerKeyContractCode { hash: hash.clone() });
    let code_entry = LedgerEntry {
        last_modified_ledger_seq: 0,
        data: LedgerEntryData::ContractCode(ContractCodeEntry {
            ext: ExtensionPoint::V0,
            hash: hash.clone(),
            code: binary.try_into().unwrap(),
        }),
        ext: LedgerEntryExt::V0,
    };

    let contract_key = LedgerKey::ContractData(LedgerKeyContractData {
        contract: ScAddress::Contract([0; 32].into()),
        key: ScVal::LedgerKeyContractInstance,
        durability: ContractDataDurability::Persistent,
    });

    let contract_entry = LedgerEntry {
        last_modified_ledger_seq: 0,
        data: LedgerEntryData::ContractData(ContractDataEntry {
            contract: ScAddress::Contract([0; 32].into()),
            key: ScVal::LedgerKeyContractInstance,
            durability: ContractDataDurability::Persistent,
            val: ScVal::ContractInstance(ScContractInstance {
                executable: ContractExecutable::Wasm(hash),
                storage: None,
            }),
            ext: ExtensionPoint::V0,
        }),
        ext: LedgerEntryExt::V0,
    };

    let snapshot = WithSnapshotInput {
        ledger_sequence: 50,
        keys: vec![code_key, contract_key],
        vals: vec![
            EntryWithLifetime {
                entry: code_entry,
                live_until: Some(100),
            },
            EntryWithLifetime {
                entry: contract_entry,
                live_until: Some(100),
            },
        ],
        contract_id: [0; 32],
        fname: String::from("hello"),
        params: vec![symbol],
    };
    println!("{}", serde_json::json!(snapshot));
}

#[test]
fn generate_snapshot_request_no_code() {
    let symbol = ScVal::Symbol(ScSymbol("tdep".to_string().try_into().unwrap()));

    let contract_key = LedgerKey::ContractData(LedgerKeyContractData {
        contract: ScAddress::Contract([0; 32].into()),
        key: ScVal::LedgerKeyContractInstance,
        durability: ContractDataDurability::Persistent,
    });

    let contract_entry = LedgerEntry {
        last_modified_ledger_seq: 0,
        data: LedgerEntryData::ContractData(ContractDataEntry {
            contract: ScAddress::Contract([0; 32].into()),
            key: ScVal::LedgerKeyContractInstance,
            durability: ContractDataDurability::Persistent,
            val: ScVal::ContractInstance(ScContractInstance {
                executable: ContractExecutable::Wasm(Hash(
                    hex::decode("ea3eacfb7157ad4cee0f5c1ea548a98aa9d93ab080a9fd28d093967be6a67028")
                        .unwrap()
                        .try_into()
                        .unwrap(),
                )),
                storage: None,
            }),
            ext: ExtensionPoint::V0,
        }),
        ext: LedgerEntryExt::V0,
    };

    let snapshot = WithSnapshotInput {
        ledger_sequence: 50,
        keys: vec![contract_key],
        vals: vec![EntryWithLifetime {
            entry: contract_entry,
            live_until: Some(100),
        }],
        contract_id: [0; 32],
        fname: String::from("hello"),
        params: vec![symbol],
    };
    println!("{}", serde_json::json!(snapshot));
}
