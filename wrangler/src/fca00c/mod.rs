use std::collections::HashMap;

use self::tasks::Task;

pub mod embedded;
pub mod routes;

pub mod response;
pub mod tasks;
#[derive(Default, Clone)]
pub struct TaskRegistry<'a> {
    map: HashMap<u64, &'a dyn Task>,
    pub debug: bool,
}

impl<'a> TaskRegistry<'a> {
    pub fn register_task(&mut self, task_id: u64, task: &'a dyn Task) {
        self.map.insert(task_id, task);
    }

    pub fn get_task(&self, task_id: &u64) -> Option<&dyn Task> {
        self.map.get(task_id).copied()
    }
}

pub fn setup(reg: &mut TaskRegistry) {
    tasks::setup(reg);
}
