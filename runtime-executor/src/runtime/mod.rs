use std::sync::OnceLock;

use mio::{Poll, Registry};

use self::executor::Executor;

pub mod executor;
pub mod reactor;

pub fn init() -> Executor {
    reactor::start();
    Executor::new()
}

static REGISTRY: OnceLock<Registry> = OnceLock::new();

pub fn registry() -> &'static Registry {
    REGISTRY.get().unwrap()
}

pub struct Runtime {
    poll: Poll,
}

impl Runtime {
    pub fn new() -> Self {
        let poll = Poll::new().unwrap();
        let registry = poll.registry().try_clone().unwrap();

        REGISTRY.set(registry).unwrap();

        Runtime { poll }
    }
}
