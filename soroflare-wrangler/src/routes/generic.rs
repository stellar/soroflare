use std::collections::HashMap;

use crate::{
    response::{BasicJsonResponse, JsonResponse},
    TaskRegistry,
};
use serde::Serialize;
use soroban_env_host::{
    budget::Budget,
    xdr::{Limits, ReadXdr, ScVec, WriteXdr},
};
use soroflare_vm::{contract_id, soroban_vm, soroflare_utils};
use worker::{Request, Response, RouteContext};

#[derive(Serialize)]
pub struct ExecutionResult {
    cpu: u64,
    mem: u64,
    result: String,
}

pub struct Generic;

impl Generic {
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
            if let Ok(vec) = ScVec::from_xdr_base64(xdr, Limits::none()) {
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
