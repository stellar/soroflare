use std::{collections::HashMap, rc::Rc};

use crate::{
    response::{BasicJsonResponse, JsonResponse},
    State,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use soroban_env_host::{
    auth::RecordedAuthPayload,
    budget::Budget,
    events::Events,
    xdr::{
        AccountId, BytesM, ContractCodeEntry, ContractDataDurability, ContractDataEntry,
        ContractEvent, ContractExecutable, ExtensionPoint, Hash, HostFunction, LedgerEntry,
        LedgerEntryData, LedgerEntryExt, LedgerFootprint, LedgerKey, LedgerKeyContractCode,
        LedgerKeyContractData, Limits, PublicKey, ReadXdr, ScAddress, ScContractInstance, ScSymbol,
        ScVal, ScVec, SorobanAddressCredentials, SorobanAuthorizationEntry, SorobanCredentials,
        SorobanResources, SorobanTransactionData, Uint256, VecM, WriteXdr,
    },
};
use soroban_simulation::{
    simulation::{InvokeHostFunctionSimulationResult, SimulationAdjustmentConfig},
    NetworkConfig,
};
use soroflare_vm::{
    contract_id, soroban_cli,
    soroban_vm::{self, build_host_function_parameters, InvocationResult},
    soroflare_utils::{self, EntryWithLifetime},
};
use worker::{console_log, kv::KvStore, Request, Response, RouteContext};

// TODO: wait on clarification about the preamble.
/// Instructions for the client to restore any potentially expired
/// ledger entries
#[derive(Serialize, Default)]
pub struct RestorePreamble {
    min_resource_fee: String,
    transaction_data: String,
}

#[derive(Deserialize, Serialize)]
pub struct WithSnapshotInput {
    ledger_sequence: u32,
    //keys: Vec<LedgerKey>, // keys of vals need to have the same index within the array.
    //vals: Vec<EntryWithLifetime>,
    ledger_entries: Vec<(LedgerKey, (LedgerEntry, Option<u32>))>,
    contract_id: [u8; 32],
    source: [u8; 32],
    fname: String,
    params: Vec<ScVal>,
    network: Option<String>,
    adjustment_config: SimulationAdjustmentConfig,
    network_config: Option<NetworkConfig>,
}

pub struct Generic;

impl Generic {
    async fn run_with_snapshot(
        req: &mut Request,
        modules: KvStore,
    ) -> Result<InvokeHostFunctionSimulationResult, Result<Response, worker::Error>> {
        //let mut api_result = ExecutionResult::default();

        let WithSnapshotInput {
            ledger_sequence,
            ledger_entries,
            contract_id,
            source,
            fname,
            params,
            network,
            adjustment_config,
            network_config,
        } = req.json().await.unwrap();

        // todo: probably remove this and adapt rest of the code.
        let mut keys = vec![];
        let mut vals = vec![];

        for (key, entry_with_ttl) in ledger_entries {
            keys.push(key);
            vals.push(EntryWithLifetime { entry: entry_with_ttl.0, live_until: entry_with_ttl.1 })
        }

        // TODO: wait on clarification about the preamble.
        //let mut expired_entries = Vec::new();
        //let mut expired_keys = Vec::new();

        // todo: group all simulation-related errors in a specific errors enum and implement conversions
        // to the RPC API for it.
        let mut wasm_hashes = Vec::new();
        for val in vals.iter() {
            //if !val.is_live(ledger_sequence) {
            //expired_entries.push(val);
            //expired_keys.push(keys[idx].clone());
            //}
            if let LedgerEntryData::ContractData(contract_data) = &val.entry.data {
                if let ScVal::ContractInstance(instance) = &contract_data.val {
                    if let ContractExecutable::Wasm(hash) = &instance.executable {
                        wasm_hashes.push(hash.0)
                    }
                }
            }
        }

        // TODO: wait on clarification about the preamble.
        /*
        if !expired_entries.is_empty() {
            let txdata = SorobanTransactionData {
                ext: ExtensionPoint::V0,
                resource_fee: 0,
                resources: SorobanResources {
                    footprint: LedgerFootprint {
                        read_only: VecM::default(),
                        read_write: expired_keys.try_into().unwrap()
                    },
                    instructions: 0,
                    read_bytes: 0,
                    write_bytes: 0
                }
            };

            api_result.restorePreamble = Some(RestorePreamble {
                transaction_data: txdata.to_xdr_base64(Limits::none()).unwrap(),
                min_resource_fee: "0".into()
            });
        }*/

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

        let state = soroflare_utils::ledger_snapshot_from_entries_and_ledger(
            ledger_sequence,
            inferred_keys,
            inferred_vals,
            network.as_deref(),
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

        //let advanced_budget = Budget::default();
        let spec_entries =
            soroban_cli::utils::get_contract_spec_from_state(&state, contract_id).unwrap();
        let (_, host_function_params) =
            build_host_function_parameters(contract_id, spec_entries, &fname, &params).unwrap();
        let host_fn = HostFunction::InvokeContract(host_function_params);
        let source_account = AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(source)));

        let rc_state = Rc::new(state.clone());
        let simulation = soroban_simulation::simulation::simulate_invoke_host_function_op(
            rc_state,
            network_config,
            &adjustment_config,
            &state.ledger_info(),
            host_fn,
            None,
            &source_account,
            [0; 32],
            true,
        )
        .unwrap();

        Ok(simulation)
    }
}

pub async fn handle_upload(
    mut req: Request,
    ctx: RouteContext<State>,
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
    ctx: RouteContext<State>,
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

mod test {
    use soroban_env_host::xdr::{
        AccountEntry, AccountEntryExt, AccountId, Int128Parts, LedgerKeyAccount, PublicKey,
        ScBytes, ScMap, ScMapEntry, ScString, SequenceNumber, String32, Thresholds, Uint256,
    };

    use super::*;
    /*
        #[test]
        fn generate_snapshot_request() {
            let symbol = ScVal::Symbol(ScSymbol("tdep".to_string().try_into().unwrap()));

            let binary = hex::decode("0061736d01000000010f0360027e7e017e60017e017e60000002070101760167000003030201020405017001010105030100100619037f01418080c0000b7f00418080c0000b7f00418080c0000b073105066d656d6f727902000568656c6c6f0001015f00020a5f5f646174615f656e6403010b5f5f686561705f6261736503020ac80102c20101027f23808080800041206b2201248080808000024002402000a741ff01712202410e460d00200241ca00470d010b200120003703082001428ee8f1d8ba02370300410021020340024020024110470d00410021020240034020024110460d01200141106a20026a200120026a290300370300200241086a21020c000b0b200141106aad4220864204844284808080201080808080002100200141206a24808080800020000f0b200141106a20026a4202370300200241086a21020c000b0b00000b02000b00430e636f6e747261637473706563763000000000000000000000000568656c6c6f000000000000010000000000000002746f00000000001100000001000003ea00000011001e11636f6e7472616374656e766d657461763000000000000000140000000000770e636f6e74726163746d6574617630000000000000000572737665720000000000000e312e37362e302d6e696768746c7900000000000000000008727373646b7665720000002f32302e302e30233832326365366363336534363163636339323532373562343732643737623663613335623263643900").unwrap();

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
                network: None,
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
                            hex::decode(
                                "ea3eacfb7157ad4cee0f5c1ea548a98aa9d93ab080a9fd28d093967be6a67028",
                            )
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
                network: None,
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
    */
    mod sac_snapshot_and_request {
        use soroban_env_host::{
            fees::{FeeConfiguration, RentFeeConfiguration},
            xdr::{ContractCostParamEntry, ContractCostParams},
            Env, Host, TryFromVal, Val,
        };

        use super::*;

        // NOTE: currently the source account is inferred within the vm wrapper itself and already has
        // a xlm balance of 10000 XLM strooops by default.
        #[test]
        fn gentest() {
            let contract_a = stellar_strkey::Contract::from_string(
                "CBKMUZNFQIAL775XBB2W2GP5CNHBM5YGH6C3XB7AY6SUVO2IBU3VYK2V",
            ).unwrap().0.into();
            let contract_b = stellar_strkey::Contract::from_string(
                "CBRIAA73VOIKPZYM5G3LGPF3NGCFXLR3IW22MKEYJAB3QBOMTUTRCASK",
            ).unwrap().0.into();

            let snapshot = WithSnapshotInput {
                adjustment_config: SimulationAdjustmentConfig::default_adjustment(),
                network_config: None,
                network: Some("Test SDF Network ; September 2015".into()),
                ledger_sequence: 0,
                source: [0; 32],
                ledger_entries: vec![],
                contract_id: contract_b,
                fname: String::from("add_with"),
                params: vec![
                    ScVal::Address(ScAddress::Contract(contract_a)),
                    ScVal::U32(5),
                    ScVal::U32(15),
                ],
            };
            println!("{}", serde_json::json!(snapshot));
        }

        #[test]
        fn transfer_ignore_fees() {
            let source_account = AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(
                stellar_strkey::ed25519::PublicKey::from_string(
                    "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
                )
                .unwrap()
                .0,
            )));
            let source_key = LedgerKey::Account(LedgerKeyAccount {
                account_id: source_account.clone(),
            });
            let source_entry = LedgerEntry {
                last_modified_ledger_seq: 0,
                data: LedgerEntryData::Account(AccountEntry {
                    account_id: source_account.clone(),
                    balance: 1000,
                    flags: 0,
                    home_domain: String32::default(),
                    inflation_dest: None,
                    num_sub_entries: 1,
                    seq_num: SequenceNumber(0),
                    thresholds: Thresholds([1; 4]),
                    signers: VecM::default(),
                    ext: AccountEntryExt::V0,
                }),
                ext: LedgerEntryExt::V0,
            };

            let account_id = AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(
                stellar_strkey::ed25519::PublicKey::from_string(
                    "GDEOJOBOGUWAZHNXLTD7BIUXHVR4A4LPIMWQTC6Z4MTG6VNL7BIFUP7M",
                )
                .unwrap()
                .0,
            )));
            let account_key = LedgerKey::Account(LedgerKeyAccount {
                account_id: account_id.clone(),
            });
            let account_entry = LedgerEntry {
                last_modified_ledger_seq: 0,
                data: LedgerEntryData::Account(AccountEntry {
                    account_id: account_id.clone(),
                    balance: 0,
                    flags: 0,
                    home_domain: String32::default(),
                    inflation_dest: None,
                    num_sub_entries: 1,
                    seq_num: SequenceNumber(0),
                    thresholds: Thresholds([1; 4]),
                    signers: VecM::default(),
                    ext: AccountEntryExt::V0,
                }),
                ext: LedgerEntryExt::V0,
            };

            let contract_key = LedgerKey::ContractData(LedgerKeyContractData {
                contract: ScAddress::Contract(
                    stellar_strkey::Contract::from_string(
                        "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
                    )
                    .unwrap()
                    .0
                    .into(),
                ),
                key: ScVal::LedgerKeyContractInstance,
                durability: ContractDataDurability::Persistent,
            });

            let contract_entry = LedgerEntry {
                last_modified_ledger_seq: 0,
                data: LedgerEntryData::ContractData(ContractDataEntry {
                    contract: ScAddress::Contract(
                        stellar_strkey::Contract::from_string(
                            "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
                        )
                        .unwrap()
                        .0
                        .into(),
                    ),
                    key: ScVal::LedgerKeyContractInstance,
                    durability: ContractDataDurability::Persistent,
                    val: ScVal::ContractInstance(ScContractInstance {
                        executable: ContractExecutable::StellarAsset,
                        storage: Some(ScMap(
                            vec![
                                ScMapEntry {
                                    key: ScVal::Symbol(ScSymbol("METADATA".try_into().unwrap())),
                                    val: ScVal::Map(Some(ScMap(
                                        vec![
                                            ScMapEntry {
                                                key: ScVal::Symbol(ScSymbol(
                                                    "decimal".try_into().unwrap(),
                                                )),
                                                val: ScVal::U32(7),
                                            },
                                            ScMapEntry {
                                                key: ScVal::Symbol(ScSymbol(
                                                    "name".try_into().unwrap(),
                                                )),
                                                val: ScVal::String(ScString(
                                                    "native".try_into().unwrap(),
                                                )),
                                            },
                                            ScMapEntry {
                                                key: ScVal::Symbol(ScSymbol(
                                                    "symbol".try_into().unwrap(),
                                                )),
                                                val: ScVal::String(ScString(
                                                    "native".try_into().unwrap(),
                                                )),
                                            },
                                        ]
                                        .try_into()
                                        .unwrap(),
                                    ))),
                                },
                                ScMapEntry {
                                    key: ScVal::Vec(Some(ScVec(
                                        vec![ScVal::Symbol(ScSymbol("Admin".try_into().unwrap()))]
                                            .try_into()
                                            .unwrap(),
                                    ))),
                                    val: ScVal::Address(ScAddress::Account(source_account.clone())),
                                },
                                ScMapEntry {
                                    key: ScVal::Vec(Some(ScVec(
                                        vec![ScVal::Symbol(ScSymbol(
                                            "AssetInfo".try_into().unwrap(),
                                        ))]
                                        .try_into()
                                        .unwrap(),
                                    ))),
                                    val: ScVal::Vec(Some(ScVec(
                                        vec![ScVal::Symbol(ScSymbol("Native".try_into().unwrap()))]
                                            .try_into()
                                            .unwrap(),
                                    ))),
                                },
                            ]
                            .try_into()
                            .unwrap(),
                        )),
                    }),
                    ext: ExtensionPoint::V0,
                }),
                ext: LedgerEntryExt::V0,
            };

            let snapshot = WithSnapshotInput {
                adjustment_config: SimulationAdjustmentConfig::default_adjustment(),
                network_config: None,
                network: Some("Test SDF Network ; September 2015".into()),
                ledger_sequence: 50,
                source: [0; 32],
                ledger_entries: vec![
                    (
                        contract_key,
                        (contract_entry, Some(100))
                    ),
                    (
                        account_key,
                        (account_entry, Some(100))
                    ),
                    (
                        source_key,
                        (source_entry, Some(100))
                    ),
                ],
                contract_id: stellar_strkey::Contract::from_string(
                    "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
                )
                .unwrap()
                .0,
                fname: String::from("transfer"),
                params: vec![
                    ScVal::Address(ScAddress::Account(source_account)),
                    ScVal::Address(ScAddress::Account(account_id)),
                    ScVal::I128(Int128Parts { hi: 0, lo: 1000 }),
                ],
            };
            println!("{}", serde_json::json!(snapshot));
        }

        #[test]
        fn transfer() {
            let source_account = AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(
                stellar_strkey::ed25519::PublicKey::from_string(
                    "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
                )
                .unwrap()
                .0,
            )));
            let source_key = LedgerKey::Account(LedgerKeyAccount {
                account_id: source_account.clone(),
            });
            let source_entry = LedgerEntry {
                last_modified_ledger_seq: 0,
                data: LedgerEntryData::Account(AccountEntry {
                    account_id: source_account.clone(),
                    balance: 1000,
                    flags: 0,
                    home_domain: String32::default(),
                    inflation_dest: None,
                    num_sub_entries: 1,
                    seq_num: SequenceNumber(0),
                    thresholds: Thresholds([1; 4]),
                    signers: VecM::default(),
                    ext: AccountEntryExt::V0,
                }),
                ext: LedgerEntryExt::V0,
            };

            let account_id = AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(
                stellar_strkey::ed25519::PublicKey::from_string(
                    "GDEOJOBOGUWAZHNXLTD7BIUXHVR4A4LPIMWQTC6Z4MTG6VNL7BIFUP7M",
                )
                .unwrap()
                .0,
            )));
            let account_key = LedgerKey::Account(LedgerKeyAccount {
                account_id: account_id.clone(),
            });
            let account_entry = LedgerEntry {
                last_modified_ledger_seq: 0,
                data: LedgerEntryData::Account(AccountEntry {
                    account_id: account_id.clone(),
                    balance: 0,
                    flags: 0,
                    home_domain: String32::default(),
                    inflation_dest: None,
                    num_sub_entries: 1,
                    seq_num: SequenceNumber(0),
                    thresholds: Thresholds([1; 4]),
                    signers: VecM::default(),
                    ext: AccountEntryExt::V0,
                }),
                ext: LedgerEntryExt::V0,
            };

            let contract_key = LedgerKey::ContractData(LedgerKeyContractData {
                contract: ScAddress::Contract(
                    stellar_strkey::Contract::from_string(
                        "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
                    )
                    .unwrap()
                    .0
                    .into(),
                ),
                key: ScVal::LedgerKeyContractInstance,
                durability: ContractDataDurability::Persistent,
            });

            let contract_entry = LedgerEntry {
                last_modified_ledger_seq: 0,
                data: LedgerEntryData::ContractData(ContractDataEntry {
                    contract: ScAddress::Contract(
                        stellar_strkey::Contract::from_string(
                            "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
                        )
                        .unwrap()
                        .0
                        .into(),
                    ),
                    key: ScVal::LedgerKeyContractInstance,
                    durability: ContractDataDurability::Persistent,
                    val: ScVal::ContractInstance(ScContractInstance {
                        executable: ContractExecutable::StellarAsset,
                        storage: Some(ScMap(
                            vec![
                                ScMapEntry {
                                    key: ScVal::Symbol(ScSymbol("METADATA".try_into().unwrap())),
                                    val: ScVal::Map(Some(ScMap(
                                        vec![
                                            ScMapEntry {
                                                key: ScVal::Symbol(ScSymbol(
                                                    "decimal".try_into().unwrap(),
                                                )),
                                                val: ScVal::U32(7),
                                            },
                                            ScMapEntry {
                                                key: ScVal::Symbol(ScSymbol(
                                                    "name".try_into().unwrap(),
                                                )),
                                                val: ScVal::String(ScString(
                                                    "native".try_into().unwrap(),
                                                )),
                                            },
                                            ScMapEntry {
                                                key: ScVal::Symbol(ScSymbol(
                                                    "symbol".try_into().unwrap(),
                                                )),
                                                val: ScVal::String(ScString(
                                                    "native".try_into().unwrap(),
                                                )),
                                            },
                                        ]
                                        .try_into()
                                        .unwrap(),
                                    ))),
                                },
                                ScMapEntry {
                                    key: ScVal::Vec(Some(ScVec(
                                        vec![ScVal::Symbol(ScSymbol("Admin".try_into().unwrap()))]
                                            .try_into()
                                            .unwrap(),
                                    ))),
                                    val: ScVal::Address(ScAddress::Account(source_account.clone())),
                                },
                                ScMapEntry {
                                    key: ScVal::Vec(Some(ScVec(
                                        vec![ScVal::Symbol(ScSymbol(
                                            "AssetInfo".try_into().unwrap(),
                                        ))]
                                        .try_into()
                                        .unwrap(),
                                    ))),
                                    val: ScVal::Vec(Some(ScVec(
                                        vec![ScVal::Symbol(ScSymbol("Native".try_into().unwrap()))]
                                            .try_into()
                                            .unwrap(),
                                    ))),
                                },
                            ]
                            .try_into()
                            .unwrap(),
                        )),
                    }),
                    ext: ExtensionPoint::V0,
                }),
                ext: LedgerEntryExt::V0,
            };

            let snapshot = WithSnapshotInput {
                adjustment_config: SimulationAdjustmentConfig::default_adjustment(),
                network_config: Some(NetworkConfig {
                    fee_configuration: FeeConfiguration {
                        fee_per_read_1kb: 1786,
                        fee_per_read_entry: 6250,
                        fee_per_contract_event_1kb: 10000,
                        fee_per_instruction_increment: 25,
                        fee_per_write_entry: 10000,
                        fee_per_write_1kb: 1786,
                        fee_per_historical_1kb: 16235,
                        fee_per_transaction_size_1kb: 1624,
                    },
                    rent_fee_configuration: RentFeeConfiguration {
                        fee_per_write_1kb: 1786,
                        fee_per_write_entry: 10000,
                        persistent_rent_rate_denominator: 1402,
                        temporary_rent_rate_denominator: 2804,
                    },
                    tx_max_instructions: 100000000,
                    tx_memory_limit: 41943040,
                    cpu_cost_params: ContractCostParams(
                        vec![
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 4,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 434,
                                linear_term: 16,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 42,
                                linear_term: 16,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 44,
                                linear_term: 16,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 295,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 60,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 221,
                                linear_term: 26,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 331,
                                linear_term: 4369,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 3636,
                                linear_term: 7013,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 40256,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 377551,
                                linear_term: 4059,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 417482,
                                linear_term: 45712,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 417482,
                                linear_term: 45712,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 1945,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 6481,
                                linear_term: 5943,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 711,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 2314804,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 4176,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 4716,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 4680,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 4256,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 884,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 1059,
                                linear_term: 502,
                            },
                        ]
                        .try_into()
                        .unwrap(),
                    ),
                    memory_cost_params: ContractCostParams(
                        vec![
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 0,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 16,
                                linear_term: 128,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 0,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 0,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 0,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 0,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 242,
                                linear_term: 384,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 0,
                                linear_term: 384,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 0,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 0,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 0,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 132773,
                                linear_term: 4903,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 132773,
                                linear_term: 4903,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 14,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 0,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 0,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 181,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 99,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 99,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 99,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 99,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 99,
                                linear_term: 0,
                            },
                            ContractCostParamEntry {
                                ext: ExtensionPoint::V0,
                                const_term: 0,
                                linear_term: 0,
                            },
                        ]
                        .try_into()
                        .unwrap(),
                    ),
                    min_temp_entry_ttl: 17280,
                    min_persistent_entry_ttl: 2073600,
                    max_entry_ttl: 3110400,
                }),
                network: Some("Test SDF Network ; September 2015".into()),
                ledger_sequence: 50,
                source: [0; 32],
                ledger_entries: vec![
                    (
                        contract_key,
                        (contract_entry, Some(100))
                    ),
                    (
                        account_key,
                        (account_entry, Some(100))
                    ),
                    (
                        source_key,
                        (source_entry, Some(100))
                    ),
                ],
                contract_id: stellar_strkey::Contract::from_string(
                    "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
                )
                .unwrap()
                .0,
                fname: String::from("transfer"),
                params: vec![
                    ScVal::Address(ScAddress::Account(source_account)),
                    ScVal::Address(ScAddress::Account(account_id)),
                    ScVal::I128(Int128Parts { hi: 0, lo: 1000 }),
                ],
            };
            println!("{}", serde_json::json!(snapshot));
        }
        /*
        #[test]
        fn admin() {
            let source_account = AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(
                stellar_strkey::ed25519::PublicKey::from_string(
                    "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
                )
                .unwrap()
                .0,
            )));

            let account_id = AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(stellar_strkey::ed25519::PublicKey::from_string("GDEOJOBOGUWAZHNXLTD7BIUXHVR4A4LPIMWQTC6Z4MTG6VNL7BIFUP7M").unwrap().0)));
            let account_key = LedgerKey::Account(LedgerKeyAccount {
                account_id: account_id.clone()
            });
            let account_entry = LedgerEntry {
                last_modified_ledger_seq: 0,
                data: LedgerEntryData::Account(AccountEntry {
                    account_id: account_id.clone(),
                    balance: 0,
                    flags: 0,
                    home_domain: String32::default(),
                    inflation_dest: None,
                    num_sub_entries: 1,
                    seq_num: SequenceNumber(0),
                    thresholds: Thresholds([1; 4]),
                    signers: VecM::default(),
                    ext: AccountEntryExt::V0,
                }),
                ext: LedgerEntryExt::V0
            };

            let contract_key = LedgerKey::ContractData(LedgerKeyContractData {
                contract: ScAddress::Contract(stellar_strkey::Contract::from_string("CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC").unwrap().0.into()),
                key: ScVal::LedgerKeyContractInstance,
                durability: ContractDataDurability::Persistent,
            });

            let contract_entry = LedgerEntry {
                last_modified_ledger_seq: 0,
                data: LedgerEntryData::ContractData(ContractDataEntry {
                    contract: ScAddress::Contract(stellar_strkey::Contract::from_string("CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC").unwrap().0.into()),
                    key: ScVal::LedgerKeyContractInstance,
                    durability: ContractDataDurability::Persistent,
                    val: ScVal::ContractInstance(ScContractInstance {
                        executable: ContractExecutable::StellarAsset,
                        storage: Some(ScMap(vec![
                            ScMapEntry {
                                key: ScVal::Symbol(ScSymbol("METADATA".try_into().unwrap())),
                                val: ScVal::Map(Some(ScMap(vec![
                                    ScMapEntry {
                                        key: ScVal::Symbol(ScSymbol("decimal".try_into().unwrap())),
                                        val: ScVal::U32(7)
                                    },
                                    ScMapEntry {
                                        key: ScVal::Symbol(ScSymbol("name".try_into().unwrap())),
                                        val: ScVal::String(ScString("native".try_into().unwrap()))
                                    },
                                    ScMapEntry {
                                        key: ScVal::Symbol(ScSymbol("symbol".try_into().unwrap())),
                                        val: ScVal::String(ScString("native".try_into().unwrap()))
                                    },
                                ].try_into().unwrap())))
                            },
                            ScMapEntry {
                                key: ScVal::Vec(Some(ScVec(vec![ScVal::Symbol(ScSymbol("Admin".try_into().unwrap()))].try_into().unwrap()))),
                                val: ScVal::Address(ScAddress::Account(source_account.clone()))
                            },
                            ScMapEntry {
                                key: ScVal::Vec(Some(ScVec(vec![ScVal::Symbol(ScSymbol("AssetInfo".try_into().unwrap()))].try_into().unwrap()))),
                                val: ScVal::Vec(Some(ScVec(vec![ScVal::Symbol(ScSymbol("Native".try_into().unwrap()))].try_into().unwrap()))),
                            },
                        ].try_into().unwrap())),
                    }),
                    ext: ExtensionPoint::V0,
                }),
                ext: LedgerEntryExt::V0,
            };

            let snapshot = WithSnapshotInput {
                network: Some("Test SDF Network ; September 2015".into()),
                ledger_sequence: 50,
                keys: vec![contract_key, account_key],
                vals: vec![EntryWithLifetime {
                    entry: contract_entry,
                    live_until: Some(100),
                }, EntryWithLifetime {
                    entry: account_entry,
                    live_until: Some(100),
                }],
                contract_id: stellar_strkey::Contract::from_string("CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC").unwrap().0,
                fname: String::from("admin"),
                params: vec![],
            };
            println!("{}", serde_json::json!(snapshot));
        }*/
    }
}
