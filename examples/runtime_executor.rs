use runtime_executor::{
    future::{Future, PollState},
    http::Http,
    runtime::Runtime,
};

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

    fn poll(&mut self) -> runtime_executor::future::PollState<Self::Output> {
        loop {
            match self.state {
                State::Start => {
                    let fut = Box::new(Http::get("/2000/ReactorExecutor1"));
                    self.state = State::Wait1(fut);
                }
                State::Wait1(ref mut fut1) => match fut1.poll() {
                    PollState::Ready(txt) => {
                        println!("{txt}");
                        let fut2 = Box::new(Http::get("/4000/ReactorExecutor2"));
                        self.state = State::Wait2(fut2);
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State::Wait2(ref mut fut2) => match fut2.poll() {
                    PollState::Ready(txt) => {
                        println!("{txt}");
                        self.state = State::Resolved;

                        break PollState::Ready("Done".to_string());
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State::Resolved => panic!(""),
            }
        }
    }
}

pub fn async_main() -> impl Future<Output = String> {
    Coroutine::new()
}

fn main() {
    let future = async_main();

    let mut runtime = Runtime::new();
    runtime.block_on(future);
}
