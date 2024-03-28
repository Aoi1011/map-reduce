use std::{
    collections::HashMap,
    io,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, OnceLock,
    },
    thread::spawn,
};

use mio::{net::TcpStream, Events, Interest, Poll, Registry, Token};

use crate::executor::Waker;

type Wakers = Arc<Mutex<HashMap<usize, Waker>>>;

static REACTOR: OnceLock<Reactor> = OnceLock::new();

pub fn reactor() -> &'static Reactor {
    REACTOR.get().expect("Called outside an runtime context")
}

pub struct Reactor {
    wakers: Wakers,
    registry: Registry,
    next_id: AtomicUsize,
}

impl Reactor {
    pub fn new(wakers: Wakers, registry: Registry) -> Self {
        let next_id = AtomicUsize::new(1);

        Self {
            wakers,
            registry,
            next_id,
        }
    }

    pub fn register(
        &self,
        stream: &mut TcpStream,
        interest: Interest,
        id: usize,
    ) -> io::Result<()> {
        self.registry.register(stream, Token(id), interest)
    }

    pub fn set_waker(&self, waker: &Waker, id: usize) -> Option<Waker> {
        let mut wakers = self.wakers.lock().unwrap();
        wakers.insert(id, waker.clone())
    }

    pub fn deregister(&self, stream: &mut TcpStream, id: usize) -> io::Result<()> {
        self.wakers.lock().map(|mut w| w.remove(&id)).unwrap();
        self.registry.deregister(stream)
    }

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
    let wakers: Wakers = Arc::new(Mutex::new(HashMap::new()));
    let poll = Poll::new().unwrap();
    let registry = poll.registry().try_clone().unwrap();
    let reactor = Reactor::new(wakers.clone(), registry);

    REACTOR.set(reactor).ok();
    spawn(move || event_loop(poll, wakers));
}
