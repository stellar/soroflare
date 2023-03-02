use std::collections::HashMap;

use crate::{
    fca00c::{
        response::{BasicJsonResponse, JsonResponse},
        tasks::TaskResult,
        TaskRegistry,
    },
};
use serde::Serialize;
use worker::{Request, Response, RouteContext};

pub async fn handle(
    mut req: Request,
    ctx: RouteContext<TaskRegistry<'_>>,
) -> Result<Response, worker::Error> {
    // extract task
    let get_query: HashMap<_, _> = req.url()?.query_pairs().into_owned().collect();

    let task = get_query
        .get("task")
        .and_then(|b| b.parse::<u64>().ok());

    if task.is_none() {
        return BasicJsonResponse::new("No integer value specified for `task`", 400).into();
    }

    let task = task.unwrap();
    let task_impl = ctx.data.get_task(&task);

    let incoming_raw = req.bytes().await;

    if incoming_raw.is_err() {
        return BasicJsonResponse::new("Error reading submitted data in body", 400).into();
    };

    let data = incoming_raw.unwrap();

    // validate WASM magic word
    if data.len() <= 4
        || !(data[0] == 0x00 && data[1] == 0x61 && data[2] == 0x73 && data[3] == 0x6d)
    {
        return BasicJsonResponse::new("Submitted data does not contain valid WASM code", 400)
            .into();
    }

    // validate task
    let task_impl = task_impl.unwrap();

    let result = task_impl.solve(&data, &req, &ctx);

    if let Err(err) = result {
        return err;
    }

    let result = result.unwrap();

    if let Some(result) = result {
        #[derive(Serialize)]
        struct Resp {
            submission: TaskResult,
        }

        // task completed sucessfully
        return JsonResponse::new("Successfully completed challenge", 200)
            .with_opt(Resp {
                submission: result,
            })
            .into();
    }

    BasicJsonResponse::new(
        "Unexpected error while computing the solution",
        500,
    )
    .into()
}
