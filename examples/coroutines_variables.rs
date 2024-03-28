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

#[derive(Default)]
struct Stack {
    counter: Option<usize>,
}

struct Coroutine {
    state: State,
    stack: Stack,
}

impl Coroutine {
    pub fn new() -> Self {
        Self {
            state: State::Start,
            stack: Stack::default(),
        }
    }
}

impl Future for Coroutine {
    type Output = String;

    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        loop {
            match self.state {
                State::Start => {
                    self.stack.counter = Some(0);

                    println!("Program starting");

                    let fut1 = Box::new(Http::get("/6000/HelloAsyncAwait1-1"));
                    self.state = State::Wait1(fut1);
                }
                State::Wait1(ref mut fut1) => match fut1.poll(waker) {
                    PollState::Ready(txt) => {
                        let mut counter = self.stack.counter.take().unwrap();

                        println!("{txt}");
                        counter += 1;

                        let fut2 = Box::new(Http::get("/7000/HelloAsyncAwait1-2"));

                        self.state = State::Wait2(fut2);
                        self.stack.counter = Some(counter);
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State::Wait2(ref mut fut2) => match fut2.poll(waker) {
                    PollState::Ready(txt) => {
                        let mut counter = self.stack.counter.take().unwrap();

                        println!("{txt}");
                        counter += 1;
                        println!("Received {counter} responses.");

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
