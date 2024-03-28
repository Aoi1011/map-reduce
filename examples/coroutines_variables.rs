use coroutines_variables::{executor::Waker, init, Future, Http, PollState};

fn main() {
    let mut executor = init();
    executor.block_on(async_main());
}

fn async_main() -> impl Future<Output = String> {
    Coroutine::new()
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
    pub fn new() -> Self {
        Self {
            state: State::Start,
        }
    }
}

impl Future for Coroutine {
    type Output = String;

    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        loop {
            match self.state {
                State::Start => {
                    println!("Program starting");

                    let fut1 = Box::new(Http::get("/600/HelloAsyncAwait"));
                    self.state = State::Wait1(fut1);
                }
                State::Wait1(ref mut fut1) => match fut1.poll(waker) {
                    PollState::Ready(txt) => {
                        println!("{txt}");

                        let fut2 = Box::new(Http::get("/400/HelloAsyncAwait"));
                        self.state = State::Wait2(fut2)
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State::Wait2(ref mut fut2) => match fut2.poll(waker) {
                    PollState::Ready(txt) => {
                        println!("{txt}");

                        self.state = State::Resolved;
                        break PollState::Ready(String::new());
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State::Resolved => panic!("Polled a resoved future"),
            }
        }
    }
}
