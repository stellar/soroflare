use std::rc::Rc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use soroban_env_host::{storage::SnapshotSource, xdr::{LedgerEntry, LedgerKey, ScError, ScErrorCode}, LedgerInfo};

pub fn hashed_network_id(passphrase: &str) -> [u8; 32] {
    Sha256::digest(passphrase.as_bytes()).into()
}

#[derive(Debug, Default, Clone)]
pub struct LedgerSnapshot {
    pub protocol_version: u32,
    pub sequence_number: u32,
    pub timestamp: u64,
    pub network_id: [u8; 32],
    pub base_reserve: u32,
    pub min_persistent_entry_ttl: u32,
    pub min_temp_entry_ttl: u32,
    pub max_entry_ttl: u32,
    pub ledger_entries: Vec<(Box<LedgerKey>, (Box<LedgerEntry>, Option<u32>))>,
}

impl LedgerSnapshot {
    pub fn ledger_info(&self) -> LedgerInfo {
        LedgerInfo {
            protocol_version: self.protocol_version,
            sequence_number: self.sequence_number,
            timestamp: self.timestamp,
            network_id: self.network_id,
            base_reserve: self.base_reserve,
            min_persistent_entry_ttl: self.min_persistent_entry_ttl,
            min_temp_entry_ttl: self.min_temp_entry_ttl,
            max_entry_ttl: self.max_entry_ttl,
        }
    }
}

impl SnapshotSource for LedgerSnapshot {
    fn get(
        &self,
        key: &std::rc::Rc<LedgerKey>,
    ) -> Result<Option<soroban_env_host::storage::EntryWithLiveUntil>, soroban_env_host::HostError>
    {
        match self.ledger_entries.iter().find(|(k, _)| **k == **key) {
            Some((_, v)) => Ok(Some((Rc::new(*v.0.clone()), v.1))),
            None => Err(ScError::Storage(ScErrorCode::MissingValue).into()),
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct EntryWithLifetime {
    pub entry: LedgerEntry,
    pub live_until: Option<u32>,
}

impl EntryWithLifetime {
    pub fn is_live(&self, ledger_sequence: u32) -> bool {
        if let Some(ttl) = self.live_until {
            ledger_sequence < ttl
        } else {
            true // all other entries are always live
        }
    }
}

pub fn ledger_snapshot_from_entries_and_ledger(
    ledger_sequence: u32,
    keys: Vec<LedgerKey>,
    vals: Vec<EntryWithLifetime>,
    network: Option<&str>,
) -> LedgerSnapshot {
    // Using a custom network id isn't really required at this point, but keeping it
    // to distinguish from other real networks. 
    let network_id = network.unwrap_or("Soroflare Stellar Network ; March 2024");
    let mut ledger_entries = Vec::new();

    for (idx, key) in keys.iter().enumerate() {
        let entry_with_lifetime = &vals[idx];
        ledger_entries.push((
            Box::new(key.clone()),
            (
                Box::new(entry_with_lifetime.entry.clone()),
                entry_with_lifetime.live_until,
            ),
        ))
    }

    LedgerSnapshot {
        network_id: hashed_network_id(network_id),
        sequence_number: ledger_sequence,
        ledger_entries,
        protocol_version: 20,
        ..Default::default()
    }
}

