use std::{
    io::{ErrorKind, Read, Write},
    net::TcpStream,
};

use mio::{Interest, Token};

use crate::{
    future::{Future, PollState},
    runtime::{executor::Waker, reactor, registry},
};

pub struct Http;

impl Http {
    pub fn get(path: &str) -> impl Future<Output = String> {
        HttpGetFuture::new(path)
    }
}

pub struct HttpGetFuture {
    stream: Option<mio::net::TcpStream>,
    buffer: Vec<u8>,
    path: String,
    id: usize,
}

impl HttpGetFuture {
    pub fn new(path: &str) -> Self {
        let id = reactor::reactor().next_id();
        Self {
            stream: None,
            buffer: Vec::new(),
            path: path.to_string(),
            id,
        }
    }

    pub fn write_request(&mut self) {
        let stream = TcpStream::connect("127.0.0.1:8080").expect("Couldn't connect the server");
        stream
            .set_nonblocking(true)
            .expect("set_nonblocking call failed");

        let mut stream = mio::net::TcpStream::from_std(stream);
        stream.write_all(get_req(&self.path).as_bytes()).unwrap();

        self.stream = Some(stream);
    }
}

impl Future for HttpGetFuture {
    type Output = String;

    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        if self.stream.is_none() {
            println!("FIRST POLL - START OPERATION");
            self.write_request();

            let stream = self.stream.as_mut().unwrap();
            reactor::reactor().register(stream, Interest::READABLE, self.id);
            reactor::reactor().set_waker(waker, self.id);
        }

        let mut buf = vec![0u8; 4096];
        loop {
            match self.stream.as_mut().unwrap().read(&mut buf) {
                Ok(0) => {
                    let txt = String::from_utf8_lossy(&self.buffer);
                    reactor::reactor().deregister(self.stream.as_mut().unwrap(), self.id);
                    break PollState::Ready(txt.to_string());
                }
                Ok(n) => {
                    self.buffer.extend(&buf[0..n]);
                    continue;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    reactor::reactor().set_waker(waker, self.id);
                    break PollState::NotReady;
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {
                    continue;
                }

                Err(e) => panic!("{e:?}"),
            }
        }
    }
}

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n
        Host: localhost\r\n
        Connection: close\r\n
        \r\n"
    )
}
