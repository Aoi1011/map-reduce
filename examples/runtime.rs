use runtime::{
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

    fn poll(&mut self) -> PollState<Self::Output> {
        loop {
            match self.state {
                State::Start => {
                    println!("Program starting");
                    let fut = Box::new(Http::get("/600/HelloWorld1"));
                    self.state = State::Wait1(fut);
                }
                State::Wait1(ref mut fut) => match fut.poll() {
                    PollState::Ready(txt) => {
                        println!("{txt}");
                        let fut2 = Box::new(Http::get("/400/HelloWorld2"));
                        self.state = State::Wait2(fut2);
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State::Wait2(ref mut fut2) => match fut2.poll() {
                    PollState::Ready(txt2) => {
                        println!("{txt2}");
                        self.state = State::Resolved;
                        break PollState::Ready(String::new());
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State::Resolved => panic!("Polled a resolved future"),
            }
        }
    }
}

fn async_main() -> impl Future<Output = String> {
    Coroutine::new()
}

// fn async_main() {
//     println!("Program starting");
//     let txt = http::Http::get("/600/HelloAsyncAwait").await;
//     println!("{txt}");
//     let txt = http::Http::get("/600/HelloAsyncAwait").await;
//     println!("{txt}");
// }

fn main() {
    let future = async_main();
    let mut runtime = Runtime::new();
    runtime.block_on(future);
}
