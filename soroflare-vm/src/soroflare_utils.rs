use serde_derive::{Deserialize, Serialize};
use soroban_env_host::xdr::{LedgerEntry, LedgerKey};
use soroban_ledger_snapshot::LedgerSnapshot;

use crate::soroban_cli::network::sandbox_network_id;

pub fn empty_ledger_snapshot() -> LedgerSnapshot {
    LedgerSnapshot {
        network_id: sandbox_network_id(),
        ..Default::default()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EntryWithLifetime {
    pub entry: LedgerEntry,
    pub live_until: Option<u32>,
}

pub fn ledger_snapshot_from_entries_and_ledger(
    ledger_sequence: u32,
    keys: Vec<LedgerKey>,
    vals: Vec<EntryWithLifetime>,
) -> Result<LedgerSnapshot, crate::soroban_vm::Error> {
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

    Ok(LedgerSnapshot {
        network_id: sandbox_network_id(),
        sequence_number: ledger_sequence,
        ledger_entries,
        ..Default::default()
    })
}
