pub mod asteroids;

use serde::{Deserialize, Serialize};
use worker::{Request, Response, RouteContext};

use self::asteroids::Asteroids;

use super::TaskRegistry;

pub fn setup(reg: &mut TaskRegistry) {
    // register future tasks here!
    reg.register_task(0, &Asteroids);
}

pub trait Task {
    fn solve(
        &self,
        raw_wasm: &[u8],
        req: &Request,
        ctx: &RouteContext<TaskRegistry<'_>>,
    ) -> Result<Option<TaskResult>, Result<Response, worker::Error>>;
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TaskResult {
    pub mem: u64,
    pub cpu: u64,
    pub size: u64,
    pub submission_time: i64,
    pub interface_version: u64,
    #[serde(skip)]
    pub result_xdr: Vec<String>,
    #[serde(skip)]
    pub opt: Vec<String>,
}
