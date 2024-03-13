// https://github.com/stellar/soroban-tools/blob/v0.8.0/cmd/soroban-cli/src/utils.rs
// and
use hex::FromHexError;
/// https://github.com/stellar/soroban-tools/blob/35d33ee0c00e6b8bb49df534b9427ed45b080b48/cmd/soroban-cli/src/utils.rs
use std::{io::ErrorKind, path::Path};

use ed25519_dalek::Signer;
use sha2::{Digest, Sha256};

use soroban_env_host::{
    storage::{AccessType, Footprint},
    xdr::{
        AccountEntry, AccountEntryExt, AccountId, ContractCodeEntry, ContractDataDurability,
        ContractDataEntry, ContractExecutable, DecoratedSignature, Error as XdrError,
        ExtensionPoint, Hash, LedgerEntry, LedgerEntryData, LedgerEntryExt, LedgerFootprint,
        LedgerKey, LedgerKeyContractCode, LedgerKeyContractData, Limits, ScAddress,
        ScContractInstance, ScSpecEntry, ScVal, SequenceNumber, Signature, SignatureHint, String32,
        Thresholds, Transaction, TransactionEnvelope, TransactionSignaturePayload,
        TransactionSignaturePayloadTaggedTransaction, TransactionV1Envelope, VecM, WriteXdr,
    },
};
//use soroban_sdk::token;

use crate::soroflare_utils::LedgerSnapshot;
use soroban_spec::read::FromWasmError;
use stellar_strkey::ed25519::PrivateKey;

use super::network::sandbox_network_id;

/// # Errors
///
/// Might return an error
pub fn contract_hash(contract: &[u8]) -> Result<Hash, XdrError> {
    Ok(Hash(Sha256::digest(contract).into()))
}

/// # Errors
///
/// Might return an error
pub fn add_contract_code_to_ledger_entries(
    entries: &mut LedgerSnapshotEntries,
    contract: Vec<u8>,
    min_persistent_entry_expiration: u32,
) -> Result<Hash, XdrError> {
    // Install the code
    let hash = contract_hash(contract.as_slice())?;
    let code_key = LedgerKey::ContractCode(LedgerKeyContractCode { hash: hash.clone() });
    let code_entry = LedgerEntry {
        last_modified_ledger_seq: 0,
        data: LedgerEntryData::ContractCode(ContractCodeEntry {
            ext: ExtensionPoint::V0,
            hash: hash.clone(),
            code: contract.try_into()?,
        }),
        ext: LedgerEntryExt::V0,
    };
    for (k, e) in &mut *entries {
        if **k == code_key {
            *e = (Box::new(code_entry), Some(min_persistent_entry_expiration));
            return Ok(hash);
        }
    }
    entries.push((
        Box::new(code_key),
        (Box::new(code_entry), Some(min_persistent_entry_expiration)),
    ));
    Ok(hash)
}

type LedgerSnapshotEntries = Vec<(Box<LedgerKey>, (Box<LedgerEntry>, Option<u32>))>;

pub fn add_contract_to_ledger_entries(
    entries: &mut LedgerSnapshotEntries,
    contract_id: [u8; 32],
    wasm_hash: [u8; 32],
    min_persistent_entry_expiration: u32,
) {
    // Create the contract
    let contract_key = LedgerKey::ContractData(LedgerKeyContractData {
        contract: ScAddress::Contract(contract_id.into()),
        key: ScVal::LedgerKeyContractInstance,
        durability: ContractDataDurability::Persistent,
    });

    let contract_entry = LedgerEntry {
        last_modified_ledger_seq: 0,
        data: LedgerEntryData::ContractData(ContractDataEntry {
            contract: ScAddress::Contract(contract_id.into()),
            key: ScVal::LedgerKeyContractInstance,
            durability: ContractDataDurability::Persistent,
            val: ScVal::ContractInstance(ScContractInstance {
                executable: ContractExecutable::Wasm(Hash(wasm_hash)),
                storage: None,
            }),
            ext: ExtensionPoint::V0,
        }),
        ext: LedgerEntryExt::V0,
    };
    for (k, e) in &mut *entries {
        if **k == contract_key {
            *e = (
                Box::new(contract_entry),
                Some(min_persistent_entry_expiration),
            );
            return;
        }
    }
    entries.push((
        Box::new(contract_key),
        (
            Box::new(contract_entry),
            Some(min_persistent_entry_expiration),
        ),
    ));
}

/// # Errors
///
/// Might return an error
pub fn padded_hex_from_str(s: &str, n: usize) -> Result<Vec<u8>, FromHexError> {
    if s.len() > n * 2 {
        return Err(FromHexError::InvalidStringLength);
    }
    let mut decoded = vec![0u8; n];
    let padded = format!("{s:0>width$}", width = n * 2);
    hex::decode_to_slice(padded, &mut decoded)?;
    Ok(decoded)
}

/// # Errors
///
/// Might return an error
pub fn transaction_hash(tx: &Transaction, network_passphrase: &str) -> Result<[u8; 32], XdrError> {
    let signature_payload = TransactionSignaturePayload {
        network_id: Hash(Sha256::digest(network_passphrase).into()),
        tagged_transaction: TransactionSignaturePayloadTaggedTransaction::Tx(tx.clone()),
    };
    Ok(Sha256::digest(signature_payload.to_xdr(Limits::none())?).into())
}

/// # Errors
///
/// Might return an error
pub fn sign_transaction(
    key: &ed25519_dalek::SigningKey,
    tx: &Transaction,
    network_passphrase: &str,
) -> Result<TransactionEnvelope, XdrError> {
    let tx_hash = transaction_hash(tx, network_passphrase)?;
    let tx_signature = key.sign(&tx_hash);

    let decorated_signature = DecoratedSignature {
        hint: SignatureHint(key.verifying_key().to_bytes()[28..].try_into()?),
        signature: Signature(tx_signature.to_bytes().try_into()?),
    };

    Ok(TransactionEnvelope::Tx(TransactionV1Envelope {
        tx: tx.clone(),
        signatures: vec![decorated_signature].try_into()?,
    }))
}

/// # Errors
///
/// Might return an error
pub fn id_from_str(contract_id: &str) -> Result<[u8; 32], stellar_strkey::DecodeError> {
    stellar_strkey::Contract::from_string(contract_id)
        .map(|strkey| strkey.0)
        .or_else(|_| {
            // strkey failed, try to parse it as a hex string, for backwards compatibility.
            padded_hex_from_str(contract_id, 32)
                .map_err(|_| stellar_strkey::DecodeError::Invalid)?
                .try_into()
                .map_err(|_| stellar_strkey::DecodeError::Invalid)
        })
        .map_err(|_| stellar_strkey::DecodeError::Invalid)
}

fn get_entry_from_snapshot(
    key: &LedgerKey,
    entries: &LedgerSnapshotEntries,
) -> Option<(Box<LedgerEntry>, Option<u32>)> {
    for (k, result) in entries {
        if *key == **k {
            return Some((*result).clone());
        }
    }
    None
}

/// # Errors
///
/// Might return an error
pub fn get_contract_spec_from_state(
    state: &LedgerSnapshot,
    contract_id: [u8; 32],
) -> Result<Option<Vec<ScSpecEntry>>, FromWasmError> {
    // Note:
    // We're not dealing with state expiration since it makes no sense
    // to do so in an execution that always spins up a brand-new ledger.

    let key = LedgerKey::ContractData(LedgerKeyContractData {
        contract: ScAddress::Contract(contract_id.into()),
        key: ScVal::LedgerKeyContractInstance,
        durability: ContractDataDurability::Persistent,
    });
    println!("searching");
    let (entry, _) = get_entry_from_snapshot(&key, &state.ledger_entries).unwrap();

    match *entry {
        LedgerEntry {
            data:
                LedgerEntryData::ContractData(ContractDataEntry {
                    val: ScVal::ContractInstance(ScContractInstance { executable, .. }),
                    ..
                }),
            ..
        } => match executable {
            ContractExecutable::StellarAsset => {
                Ok(None)
                // panic!("soroflare can't handle direct SACs invocations") // NB: this code is actually never going to be executed since
                // it would require the user to have loaded a SAC which is not possible.
            }
            ContractExecutable::Wasm(hash) => {
                // It's a contract code entry, so it should have an expiration if present
                let (entry, _) = match get_entry_from_snapshot(
                    &LedgerKey::ContractCode(LedgerKeyContractCode { hash: hash.clone() }),
                    &state.ledger_entries,
                ) {
                    // It's a contract data entry, so it should have an expiration if present
                    Some((entry, expiration)) => (entry, expiration.unwrap()),
                    None => return Err(FromWasmError::NotFound),
                };

                match *entry {
                    LedgerEntry {
                        data: LedgerEntryData::ContractCode(ContractCodeEntry { code, .. }),
                        ..
                    } => soroban_spec::read::from_wasm(code.as_vec()).map(|res| Some(res)),
                    _ => Err(FromWasmError::NotFound),
                }
            }
        },
        _ => Err(FromWasmError::NotFound),
    }
}

/// # Errors
///
/// Might return an error
pub fn vec_to_hash(res: &ScVal) -> Result<String, XdrError> {
    if let ScVal::Bytes(res_hash) = &res {
        let mut hash_bytes: [u8; 32] = [0; 32];
        for (i, b) in res_hash.iter().enumerate() {
            hash_bytes[i] = *b;
        }
        Ok(hex::encode(hash_bytes))
    } else {
        Err(XdrError::Invalid)
    }
}

/// # Panics
///
/// May panic
#[must_use]
pub fn create_ledger_footprint(footprint: &Footprint) -> LedgerFootprint {
    let mut read_only: Vec<LedgerKey> = vec![];
    let mut read_write: Vec<LedgerKey> = vec![];
    let Footprint(m) = footprint;
    for (k, v) in m {
        let dest = match v {
            AccessType::ReadOnly => &mut read_only,
            AccessType::ReadWrite => &mut read_write,
        };
        dest.push((**k).clone());
    }
    LedgerFootprint {
        read_only: read_only.try_into().unwrap(),
        read_write: read_write.try_into().unwrap(),
    }
}

#[must_use]
pub fn default_account_ledger_entry(account_id: AccountId) -> LedgerEntry {
    // TODO: Consider moving the definition of a default account ledger entry to
    // a location shared by the SDK and CLI. The SDK currently defines the same
    // value (see URL below). There's some benefit in only defining this once to
    // prevent the two from diverging, which would cause inconsistent test
    // behavior between the SDK and CLI. A good home for this is unclear at this
    // time.
    // https://github.com/stellar/rs-soroban-sdk/blob/b6f9a2c7ec54d2d5b5a1e02d1e38ae3158c22e78/soroban-sdk/src/accounts.rs#L470-L483.
    LedgerEntry {
        data: LedgerEntryData::Account(AccountEntry {
            account_id,
            balance: 100000000000, // todo, don't infer balances, should be provided as entries
            flags: 0,
            home_domain: String32::default(),
            inflation_dest: None,
            num_sub_entries: 1,
            seq_num: SequenceNumber(0),
            thresholds: Thresholds([1; 4]),
            signers: VecM::default(),
            ext: AccountEntryExt::V0,
        }),
        last_modified_ledger_seq: 0,
        ext: LedgerEntryExt::V0,
    }
}

/// # Errors
/// May not find a config dir
pub fn find_config_dir(mut pwd: std::path::PathBuf) -> std::io::Result<std::path::PathBuf> {
    let soroban_dir = |p: &std::path::Path| p.join(".soroban");
    while !soroban_dir(&pwd).exists() {
        if !pwd.pop() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "soroban directory not found",
            ));
        }
    }
    Ok(soroban_dir(&pwd))
}

pub(crate) fn into_signing_key(key: &PrivateKey) -> ed25519_dalek::SigningKey {
    let secret: ed25519_dalek::SecretKey = key.0;
    ed25519_dalek::SigningKey::from_bytes(&secret)
}

/// Used in tests
#[allow(unused)]
pub(crate) fn parse_secret_key(
    s: &str,
) -> Result<ed25519_dalek::SigningKey, stellar_strkey::DecodeError> {
    Ok(into_signing_key(&PrivateKey::from_string(s)?))
}
