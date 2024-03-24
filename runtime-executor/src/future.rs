trait Future {
    type Output;

    fn poll(&mut self) -> PollState<Self::Output>;
}

enum PollState<T> {
    Ready(T),
    NotReady,
}
