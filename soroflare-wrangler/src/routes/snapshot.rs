use core::{SoroflareInvocation, SoroflareInvocationParams};

use crate::{
    response::{BasicJsonResponse, JsonResponse},
    State,
};
use serde::Serialize;
use sha2::{Digest, Sha256};


use soroban_env_host::xdr::{BytesM, ContractCodeEntry, ContractExecutable, ExtensionPoint, Hash, LedgerEntry, LedgerEntryData, LedgerEntryExt, LedgerKey, LedgerKeyContractCode, ScVal};
use soroban_simulation::simulation::InvokeHostFunctionSimulationResult;

use worker::{kv::KvStore, Request, Response, RouteContext};

// TODO: wait on clarification about the preamble.
/// Instructions for the client to restore any potentially expired
/// ledger entries
#[derive(Serialize, Default)]
pub struct RestorePreamble {
    min_resource_fee: String,
    transaction_data: String,
}

pub struct Generic;

impl Generic {
    async fn run_with_snapshot(
        req: &mut Request,
        modules: KvStore,
    ) -> Result<InvokeHostFunctionSimulationResult, Result<Response, worker::Error>> {
        let mut params: SoroflareInvocationParams = req.json().await.unwrap();
        
        // Here soroflare automatically adds the binaries requested if needed
        let new_entries = {
            let mut new_entries = params.entries();
            let mut wasm_hashes = Vec::new();
            let keys: Vec<LedgerKey> = params.entries().iter().map(|e| e.0.clone()).collect();

            for val in &new_entries {
                if let LedgerEntryData::ContractData(contract_data) = &val.1.0.data {
                    if let ScVal::ContractInstance(instance) = &contract_data.val {
                        if let ContractExecutable::Wasm(hash) = &instance.executable {
                            wasm_hashes.push(hash.0)
                        }
                    }
                }
            }

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
                            let val = (
                                LedgerEntry {
                                    last_modified_ledger_seq: 0,
                                    data: LedgerEntryData::ContractCode(ContractCodeEntry {
                                        ext: ExtensionPoint::V0,
                                        hash: Hash(hash),
                                        code: BytesM::try_from(module).unwrap(),
                                    }),
                                    ext: LedgerEntryExt::V0,
                                },
                                Some(u32::MAX)
                            );

                            new_entries.push((key, val));
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

            new_entries
        };
        params.set_entries(new_entries);

        let soroflare_simulator = SoroflareInvocation::new(params);
        
        Ok(soroflare_simulator.resolve())
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
        BytesM, ContractCodeEntry, ContractDataDurability, ContractDataEntry,
        ContractEvent, ContractExecutable, ExtensionPoint, Hash, HostFunction, LedgerEntry,
        LedgerEntryData, LedgerEntryExt, LedgerFootprint, LedgerKey, LedgerKeyContractCode,
        LedgerKeyContractData, Limits, ReadXdr, ScAddress, ScContractInstance, ScSymbol,
        ScVal, ScVec, SorobanAddressCredentials, SorobanAuthorizationEntry, SorobanCredentials,
        SorobanResources, SorobanTransactionData, VecM, WriteXdr,
    };

    use super::*;
    
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
                                "cb212e08157def179b96989e9178d8cae62ce7b2155497ade08b08156f1921e8",
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

            let params = SoroflareInvocationParams::new("hello".into(), [0;32], vec![symbol], [0; 32], 50, vec![
                (contract_key, (contract_entry, Some(100)))
            ], None, None, None);

            println!("{}", serde_json::json!(params));
        }
   
    mod sac_snapshot_and_request {
        use soroban_env_host::{
            fees::{FeeConfiguration, RentFeeConfiguration},
            xdr::{ContractCostParamEntry, ContractCostParams},
            Env, Host, TryFromVal, Val,
        };
        use soroban_simulation::{simulation::SimulationAdjustmentConfig, NetworkConfig};

        use super::*;

        // NOTE: currently the source account is inferred within the vm wrapper itself and already has
        // a xlm balance of 10000 XLM strooops by default.
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

            let params = SoroflareInvocationParams::new(
                "transfer".into(), 
                stellar_strkey::Contract::from_string(
                    "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
                )
                .unwrap()
                .0, 
                vec![
                    ScVal::Address(ScAddress::Account(source_account)),
                    ScVal::Address(ScAddress::Account(account_id)),
                    ScVal::I128(Int128Parts { hi: 0, lo: 1000 }),
                ], 
                [0; 32], 
                50, 
                vec![
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
                Some("Test SDF Network ; September 2015".into()), 
                None, 
                None
            );

            println!("{}", serde_json::json!(params));
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

            let params = SoroflareInvocationParams::new(
                "transfer".into(), 
                stellar_strkey::Contract::from_string(
                    "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
                )
                .unwrap()
                .0, 
                vec![
                    ScVal::Address(ScAddress::Account(source_account)),
                    ScVal::Address(ScAddress::Account(account_id)),
                    ScVal::I128(Int128Parts { hi: 0, lo: 1000 }),
                ], 
                [0; 32], 
                50, 
                vec![
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
                Some("Test SDF Network ; September 2015".into()), 
                Some(NetworkConfig {
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
                Some(SimulationAdjustmentConfig::default_adjustment())
            );
            
            println!("{}", serde_json::json!(params));
        }
    }
}
