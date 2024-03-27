use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, OnceLock,
    },
};

use mio::{net::TcpStream, Events, Interest, Poll, Registry, Token};

use crate::runtime::executor::Waker;

type Wakers = Arc<Mutex<HashMap<usize, Waker>>>;

static REACTOR: OnceLock<Reactor> = OnceLock::new();

pub fn reactor() -> &'static Reactor {
    REACTOR.get().expect("Called outside an runtime context")
}

pub struct Reactor {
    /// Each identified by an integer
    wakers: Wakers,

    /// To interact with the event queue in mio
    registry: Registry,

    /// Stores the next available ID so that we can track which event occurred and which Waker
    /// should be woken
    next_id: AtomicUsize,
}

impl Reactor {
    /// Thin wrapper around `Registry::register`.
    pub fn register(&self, stream: &mut TcpStream, interest: Interest, id: usize) {
        self.registry.register(stream, Token(id), interest).unwrap();
    }

    /// Adds a `Waker` to our `HashMap` using the provided id property as a key to identify it.
    pub fn set_waker(&self, waker: &Waker, id: usize) {
        let _ = self
            .wakers
            .lock()
            .map(|mut w| w.insert(id, waker.clone()).is_none())
            .unwrap();
    }

    /// Does 2 things
    /// 1. Removes the Waker from our `waker` collection.
    /// 2. Deregister the `TcpStream` from our `Poll` instance.
    pub fn deregister(&self, stream: &mut TcpStream, id: usize) {
        self.wakers.lock().map(|mut w| w.remove(&id)).unwrap();
        self.registry.deregister(stream).unwrap();
    }

    /// Get the current `next_id` value and increments the counter atomically.
    pub fn next_id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

fn event_loop(mut poll: Poll, wakers: Wakers) {
    let mut events = Events::with_capacity(100);
    loop {
        poll.poll(&mut events, None).unwrap();
        for e in events.iter() {
            let Token(id) = e.token();
            let wakers = wakers.lock().unwrap();
            if let Some(waker) = wakers.get(&id) {
                waker.wake();
            }
        }
    }
}

pub fn start() {
    let wakers = Arc::new(Mutex::new(HashMap::new()));
    let poll = Poll::new().unwrap();
    let registry = poll.registry().try_clone().unwrap();
    let next_id = AtomicUsize::new(1);
    let reactor = Reactor {
        wakers: wakers.clone(),
        registry,
        next_id,
    };

    REACTOR.set(reactor).ok().expect("REACTOR already running");
    std::thread::spawn(move || event_loop(poll, wakers));
}
