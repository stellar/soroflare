use std::rc::Rc;
use serde::{Deserialize, Serialize};
use snapshot::{hashed_network_id, LedgerSnapshot};
use soroban_env_host::xdr::{AccountId, Hash, HostFunction, InvokeContractArgs, LedgerEntry, LedgerKey, PublicKey, ScAddress, ScSymbol, ScVal, ScVec, StringM, Uint256};
use soroban_simulation::{simulation::{InvokeHostFunctionSimulationResult, SimulationAdjustmentConfig}, NetworkConfig};

mod snapshot;

#[derive(Serialize, Deserialize)]
pub struct SoroflareInvocationParams {
    fname: String,
    contract: [u8; 32],
    args: Vec<ScVal>,
    source_account: [u8; 32],
    ledger_sequence: u32,
    ledger_entries: Vec<(LedgerKey, (LedgerEntry, Option<u32>))>,
    network: Option<String>,
    network_config: Option<NetworkConfig>,
    adjustment_config: Option<SimulationAdjustmentConfig>,
}

impl SoroflareInvocationParams {
    pub fn new(
        fname: String,
        contract: [u8; 32],
        args: Vec<ScVal>,
        source_account: [u8; 32],
        ledger_sequence: u32,
        ledger_entries: Vec<(LedgerKey, (LedgerEntry, Option<u32>))>,
        network: Option<String>,
        network_config: Option<NetworkConfig>,
        adjustment_config: Option<SimulationAdjustmentConfig>,
    ) -> Self {
        SoroflareInvocationParams {
            fname,
            contract,
            args,
            source_account,
            ledger_sequence,
            ledger_entries,
            network,
            network_config,
            adjustment_config,
        }
    }
    
    pub fn entries(&self) -> Vec<(LedgerKey, (LedgerEntry, Option<u32>))> {
        self.ledger_entries.clone()
    }

    pub fn set_entries(&mut self, entries: Vec<(LedgerKey, (LedgerEntry, Option<u32>))>) {
        self.ledger_entries = entries
    }

    pub fn host_function(&self) -> HostFunction {
        let mut complete_args = vec![];
        complete_args.extend_from_slice(self.args.as_slice());

        let invoke_args = InvokeContractArgs {
            contract_address: ScAddress::Contract(Hash(self.contract.clone())),
            function_name: ScSymbol(<_ as TryInto<StringM<32>>>::try_into(&self.fname).unwrap()),
            args: complete_args
                .try_into()
                .unwrap(),
        };

        HostFunction::InvokeContract(invoke_args)
    }

    pub fn snapshot(&self) -> LedgerSnapshot {
        let network = &self.network.clone().unwrap_or("Soroflare Stellar Network ; March 2024".into());
        let ledger_entries = self.ledger_entries.iter().map(|entry| {
            (Box::new(entry.clone().0), (Box::new(entry.clone().1.0), entry.1.1))
        }).collect();

        LedgerSnapshot {
            network_id: hashed_network_id(network),
            sequence_number: self.ledger_sequence,
            ledger_entries,
            protocol_version: 20,
            ..Default::default()
        }
    }
}


pub struct ConfigSetup {
    network_config: Option<NetworkConfig>,
    adjustment_config: SimulationAdjustmentConfig,
}

pub struct SoroflareInvocation {
    config_setup: ConfigSetup,
    host_fn: HostFunction,
    source_account: AccountId,
    snapshot: Rc<LedgerSnapshot>
}

// todo: implement restore preamble

impl SoroflareInvocation {
    pub fn new(params: SoroflareInvocationParams) -> Self {
        let host_fn = params.host_function();
        let snapshot = Rc::new(params.snapshot());

        let config_setup = ConfigSetup { 
            network_config: params.network_config, 
            adjustment_config: params.adjustment_config.unwrap_or(SimulationAdjustmentConfig::default_adjustment()), 
        };        


        Self { 
            config_setup,
            host_fn, 
            source_account: AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(params.source_account))), 
            snapshot
        }
    }

    pub fn resolve(&self) -> InvokeHostFunctionSimulationResult {
        let snapshot_source = self.snapshot.clone();

        soroban_simulation::simulation::simulate_invoke_host_function_op(
            snapshot_source.clone(), 
            self.config_setup.network_config.clone(), 
            &self.config_setup.adjustment_config, 
            &snapshot_source.ledger_info(), 
            self.host_fn.clone(), 
            None, 
            &self.source_account, 
            [0; 32], 
            true
        ).unwrap()
    }
}
