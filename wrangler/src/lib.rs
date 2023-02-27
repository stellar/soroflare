use fca00c::TaskRegistry;
use worker::{console_log, event, Cors, Date, Env, Method, Request, Response, Result, Router};

mod fca00c;
mod soroban;
mod utils;

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
    let mut task_reg = TaskRegistry::default();

    fca00c::setup(&mut task_reg);

    task_reg.debug = matches!(
        env.var("ENVIRONMENT")?.to_string().as_str(),
        "local" | "dev"
    );

    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    let mut router = Router::with_data(task_reg.clone());
    router = router
        .options("/submit", |_req, _ctx| Response::empty())
        .post_async("/submit", fca00c::routes::submit::handle);

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
