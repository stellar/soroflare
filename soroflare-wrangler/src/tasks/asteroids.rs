use crate::{
    embedded::contracts,
    response::{BasicJsonResponse, JsonResponse},
    TaskRegistry,
};
use soroban_env_host::{
    budget::Budget,
    xdr::{Hash, ScAddress, ScVal, WriteXdr},
};
use soroflare_vm::{contract_id, helpers::*, soroban_vm, soroflare_utils};
use worker::{Request, Response, RouteContext};

use super::TaskResult;
use base64::Engine as _;

pub struct Asteroids;

impl super::Task for Asteroids {
    fn solve(
        &self,
        raw_wasm: &[u8],
        _: &Request,
        ctx: &RouteContext<TaskRegistry<'_>>,
    ) -> Result<Option<TaskResult>, Result<Response, worker::Error>> {
        let exec_time = chrono::Utc::now().timestamp_millis();
        let engine_id = contract_id!(0);

        let mut state = soroflare_utils::empty_ledger_snapshot();
        let deploy_engine_result =
            soroban_vm::deploy(&contracts::ASTEROIDS_ENGINE, &engine_id, &mut state);

        if deploy_engine_result.is_err() {
            return Err(BasicJsonResponse::new("Error deploying game engine contract", 500).into());
        }

        let seed = ctx
            .env
            .var("ASTEROIDS_SEED")
            .map(|b| b.to_string().parse::<u64>().ok())
            .unwrap_or(None);

        if seed.is_none() {
            return Err(BasicJsonResponse::new(
                "Failed to initialize game engine contract: No or invalid seed was set!",
                500,
            )
            .into());
        }

        // engine.rs is defines init(..) as:
        // move_step: u32,
        // laser_range: u32,
        // seed: u64,
        // view_range: u32,
        // fuel: (u32, u32, u32, u32),
        // asteroid_reward: u32,
        // asteroid_density: u32,
        // pod_density: u32,

        let engine_init_args: Vec<ScVal> = vec![
            1u32.into(),
            3u32.into(),
            seed.into(),
            16u32.into(),
            // the unwrap() here is not nice but it _should_ work
            // tuples are stored as ScVecs
            ScValHelper::try_from(vec![50u32, 5u32, 2u32, 1u32])
                .unwrap()
                .into(),
            1u32.into(),
            6u32.into(),
            2u32.into(),
        ];

        let engine_init_result =
            soroban_vm::invoke(&engine_id, "init", &engine_init_args, &mut state);

        if let Err(e) = engine_init_result {
            return Err(JsonResponse::new("Failed to initialize game engine", 500)
                .with_opt(e.to_string())
                .into());
        }

        let solution_id = contract_id!(1);

        let solution_deploy_result = soroban_vm::deploy(raw_wasm, &solution_id, &mut state);

        if let Err(e) = solution_deploy_result {
            return Err(JsonResponse::new("Failed to deploy user contract", 500)
                .with_opt(e.to_string())
                .into());
        }

        let cpu_limit: u64 = ctx
            .env
            .var("SOROBAN_CPU_BUDGET")
            .map(|b| b.to_string().parse::<u64>().ok())
            .unwrap_or(None)
            .unwrap_or(u64::MAX);

        let advanced_budget = Budget::default();
        advanced_budget.reset_limits(cpu_limit, u64::MAX);

        let solution_solve_result = soroban_vm::invoke_with_budget(
            &solution_id,
            "solve",
            &vec![ScVal::Address(ScAddress::Contract(Hash::from(engine_id)))],
            &mut state,
            Some(advanced_budget),
        );

        if let Err(e) = solution_solve_result {
            let msg = e.to_string();

            if msg.contains("LimitExceeded") {
                // over the budgt
                return Err(BasicJsonResponse::new("User contract exceeded budget", 400).into());
            }

            // This error is most likely the users fault...
            return Err(
                JsonResponse::new("Failed to call solve on user contract", 400)
                    .with_opt(e.to_string())
                    .into(),
            );
        }
        let (user_solve_result, (_, user_solve_budget, _)) = solution_solve_result.unwrap();

        let points_req = soroban_vm::invoke(&engine_id, "p_points", &vec![], &mut state).unwrap();
        let fuel_req = soroban_vm::invoke(&engine_id, "p_fuel", &vec![], &mut state).unwrap();
        let pos_req = soroban_vm::invoke(&engine_id, "p_pos", &vec![], &mut state).unwrap();

        let cpu_count = user_solve_budget.get_cpu_insns_count();
        let mem_count = user_solve_budget.get_mem_bytes_count();

        if let ScVal::U32(i) = points_req.0 {
            if i < 100u32 {
                return Err(BasicJsonResponse::new(
                    "User did not solve challenge! (points!=100)",
                    400,
                )
                .into());
            }
        } else {
            return Err(BasicJsonResponse::new("Unable to parse final score!", 500).into());
        }

        let mut results = vec![];

        for val in vec![user_solve_result, points_req.0, fuel_req.0, pos_req.0] {
            let mut buf = Vec::new();
            let _ = val.write_xdr(&mut buf);
            results.push(base64::engine::general_purpose::STANDARD.encode(&buf));
        }

        Ok(Some(TaskResult {
            mem: mem_count,
            cpu: cpu_count,
            size: raw_wasm.len() as u64,
            submission_time: exec_time,
            result_xdr: results,
            opt: vec![],
            interface_version: soroban_env_host::meta::INTERFACE_VERSION,
        }))
    }
}
