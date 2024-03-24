use std::{
    io::{ErrorKind, Read, Write},
    net::TcpStream,
    sync::OnceLock,
};

use mio::{Events, Interest, Poll, Registry, Token};

static REGISTRY: OnceLock<Registry> = OnceLock::new();

fn registry() -> &'static Registry {
    REGISTRY.get().expect("Called outside a runtime context")
}

trait Future {
    type Output;

    fn poll(&mut self) -> PollState<Self::Output>;
}

enum PollState<T> {
    Ready(T),
    NotReady,
}

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n
        Host: localhost\r\n
        Connection: close\r\n
        \r\n"
    )
}

struct Http;

impl Http {
    fn get(path: &str) -> impl Future<Output = String> {
        HttpGetFuture::new(path)
    }
}

struct HttpGetFuture {
    stream: Option<mio::net::TcpStream>,
    buffer: Vec<u8>,
    path: String,
}

impl HttpGetFuture {
    fn new(path: &str) -> Self {
        Self {
            stream: None,
            buffer: Vec::new(),
            path: path.to_string(),
        }
    }

    fn write_request(&mut self) {
        let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
        stream.set_nonblocking(true).unwrap();

        let mut stream = mio::net::TcpStream::from_std(stream);

        stream.write_all(get_req(&self.path).as_bytes()).unwrap();

        self.stream = Some(stream);
    }
}

impl Future for HttpGetFuture {
    type Output = String;

    fn poll(&mut self) -> PollState<Self::Output> {
        if self.stream.is_none() {
            self.write_request();

            registry()
                .register(self.stream.as_mut().unwrap(), Token(0), Interest::READABLE)
                .unwrap();
        }

        let mut buf = vec![0u8; 4096];
        loop {
            match self.stream.as_mut().unwrap().read(&mut buf) {
                Ok(0) => {
                    let txt = String::from_utf8_lossy(&self.buffer);
                    break PollState::Ready(txt.to_string());
                }
                Ok(n) => {
                    self.buffer.extend(&buf[0..n]);
                    continue;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break PollState::NotReady,
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => panic!("{e:?}"),
            }
        }
    }
}

struct Runtime {
    poll: Poll,
}

impl Runtime {
    fn new() -> Self {
        let poll = Poll::new().unwrap();
        let registry = poll.registry().try_clone().unwrap();
        REGISTRY.set(registry).unwrap();

        Self { poll }
    }

    fn block_on<F>(&mut self, future: F)
    where
        F: Future<Output = String>,
    {
        let mut future = future;
        loop {
            match future.poll() {
                PollState::NotReady => {
                    let mut events = Events::with_capacity(100);
                    self.poll.poll(&mut events, None).unwrap();
                }
                PollState::Ready(_) => break,
            }
        }
    }
}

enum State {
    Start,
    Wait1(Box<dyn Future<Output = String>>),
    Wait2(Box<dyn Future<Output = String>>),
    Resolved,
}

struct Coroutine {
    state: State,
}

impl Coroutine {
    fn new() -> Self {
        Self {
            state: State::Start,
        }
    }
}

impl Future for Coroutine {
    type Output = String;

    fn poll(&mut self) -> PollState<Self::Output> {
        loop {
            match self.state {
                State::Start => {
                    let fut = Box::new(Http::get("/4000/HelloWorld4"));
                    self.state = State::Wait1(fut);
                }
                State::Wait1(ref mut fut1) => match fut1.poll() {
                    PollState::Ready(txt) => {
                        println!("{txt}");
                        let fut2 = Box::new(Http::get("/8000/HelloWorld8"));
                        self.state = State::Wait2(fut2);
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State::Wait2(ref mut fut2) => match fut2.poll() {
                    PollState::Ready(txt) => {
                        println!("{txt}");
                        self.state = State::Resolved;
                        break PollState::Ready(String::new());
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State::Resolved => panic!(""),
            }
        }
    }
}

fn async_main() -> impl Future<Output = String> {
    Coroutine::new()
}

fn main() {
    let future = async_main();
    let mut runtime = Runtime::new();
    runtime.block_on(future);
}
