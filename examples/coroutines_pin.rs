use std::{marker::PhantomPinned, pin::Pin};

use coroutines_pin::{executor::Waker, init, Future, Http, PollState};

fn main() {
    let mut executor = init();
    executor.block_on(async_main());
}

fn async_main() -> impl Future<Output = String> {
    Coroutine::new()
}

enum State {
    Start,
    Wait1(Pin<Box<dyn Future<Output = String>>>),
    Wait2(Pin<Box<dyn Future<Output = String>>>),
    Resolved,
}

#[derive(Default)]
struct Stack {
    counter: Option<usize>,
}

struct Coroutine {
    state: State,
    stack: Stack,
    _pin: PhantomPinned,
}

impl Coroutine {
    pub fn new() -> Self {
        Self {
            state: State::Start,
            stack: Stack::default(),
            _pin: PhantomPinned,
        }
    }
}

impl Future for Coroutine {
    type Output = String;

    fn poll(self: Pin<&mut Self>, waker: &Waker) -> PollState<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        loop {
            match this.state {
                State::Start => {
                    this.stack.counter = Some(0);

                    println!("Program starting");

                    let fut1 = Box::pin(Http::get("/6000/HelloAsyncAwait1-1"));
                    this.state = State::Wait1(fut1);
                }
                State::Wait1(ref mut fut1) => match fut1.as_mut().poll(waker) {
                    PollState::Ready(txt) => {
                        let mut counter = this.stack.counter.take().unwrap();

                        println!("{txt}");
                        counter += 1;

                        let fut2 = Box::pin(Http::get("/7000/HelloAsyncAwait1-2"));

                        this.state = State::Wait2(fut2);
                        this.stack.counter = Some(counter);
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State::Wait2(ref mut fut2) => match fut2.as_mut().poll(waker) {
                    PollState::Ready(txt) => {
                        let mut counter = this.stack.counter.take().unwrap();

                        println!("{txt}");
                        counter += 1;
                        println!("Received {counter} responses.");

                        this.state = State::Resolved;
                        break PollState::Ready(String::new());
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State::Resolved => panic!("Polled a resoved future"),
            }
        }
    }
}
