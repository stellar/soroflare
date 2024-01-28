use tasks::TaskRegistry;
use worker::{console_log, event, Cors, Date, Env, Method, Request, Response, Result, Router};

mod embedded;
mod response;
mod routes;
mod tasks;
mod utils;

pub fn setup(reg: &mut TaskRegistry) {
    tasks::setup(reg);
}

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    log_request(&req);

    let mut task_reg = TaskRegistry::default();
    tasks::setup(&mut task_reg);

    task_reg.debug = true;

    let mut router = Router::with_data(task_reg.clone());
    router = router
        .options("/submit", |_req, _ctx| Response::empty())
        .post_async("/submit", routes::submit::handle)
        .options("/execute", |_req, _ctx| Response::empty())
        .post_async("/execute", routes::generic::handle)
        .options("/executesnapshot", |_req, _ctx| Response::empty())
        .post_async("/executesnapshot", routes::generic::handle_snapshot);

    let cors = Cors::new()
        .with_allowed_headers(["*"])
        .with_origins(["*"])
        .with_max_age(86400)
        .with_methods([Method::Get, Method::Post, Method::Options]);

    router
        .run(req, env)
        .await
        .and_then(|success| success.with_cors(&cors))
}
