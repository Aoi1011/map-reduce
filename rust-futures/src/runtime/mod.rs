use self::{executor::Executor, reactor::start};

pub mod executor;
pub mod reactor;

pub fn init() -> Executor {
    start();
    Executor::new()
}
