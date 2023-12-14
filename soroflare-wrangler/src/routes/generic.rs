use worker::{Request, Response, RouteContext};
use soroflare_vm::{contract_id, helpers::*, soroban_vm, soroflare_utils};

pub struct GenericExecution;

impl GenericExecution {
    pub fn run(
        &self, 
        raw_wasm: &[u8],
        cpu_limit: u64,
        memory_limit: u64
        _: &Request
    ) -> Result<Option<TaskResult>, Result<Response, worker::Error>> {
        let mut state = soroflare_utils::empty_ledger_snapshot();
        let contract_id = contract_id!(0);


        let deploy_result = soroban_vm::deploy(raw_wasm, &mut state, &contract_id);

        if let Err(e) = deploy_result {
            return Err(JsonResponse::new("Failed to deploy user contract", 500)
                .with_opt(e.to_string())
                .into());
        }
    }
}

pub async fn handle(
    mut req: Request,
    ctx: RouteContext<_>,
) -> Result<Response, worker::Error> {
    // extract task
    let get_query: HashMap<_, _> = req.url()?.query_pairs().into_owned().collect();

    let cpu_limit = if let Some(limit) = get_query.get("cpu_limit").and_then(|b| b.parse::<u64>().ok()) {
        limit
    } else {
        return BasicJsonResponse::new("No integer value specified for `cpu_limit`", 400).into();
    };

    let memory_limit = if let Some(limit) = get_query.get("memory_limit").and_then(|b| b.parse::<u64>().ok()) {
        limit
    } else {
        return BasicJsonResponse::new("No integer value specified for `memory_limit`", 400).into();
    };

    let incoming_raw = if let Ok(data) = req.bytes().await {
        data
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

    let result = GenericExecution::run(&data, &req, &ctx);

    
    let result = if let Ok(res) = result {
        res
    } else {
        return result.err()
    };

    if let Some(result) = result {
        #[derive(Serialize)]
        struct Resp {
            submission: TaskResult,
        }

        // task completed sucessfully
        return JsonResponse::new("Successfully completed challenge", 200)
            .with_opt(Resp { submission: result })
            .into();
    }

    BasicJsonResponse::new("Unexpected error while computing the solution", 500).into()
}
