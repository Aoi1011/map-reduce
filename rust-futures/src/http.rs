use std::{
    future::Future,
    io::{ErrorKind, Read, Write},
    pin::Pin,
    task::{Context, Poll},
};

use mio::Interest;

use crate::{reactor::reactor, runtime};

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
        Host: localhost\r\n\
        Connection: close\r\n\
        \r\n"
    )
}

pub struct Http;

impl Http {
    pub fn get(path: &str) -> impl Future<Output = String> {
        HttpGetFuture::new(path.to_string())
    }
}

struct HttpGetFuture {
    stream: Option<mio::net::TcpStream>,
    buffer: Vec<u8>,
    path: String,
    id: usize,
}

impl HttpGetFuture {
    fn new(path: String) -> Self {
        let id = reactor().next_id();
        Self {
            stream: None,
            buffer: vec![],
            path,
            id,
        }
    }

    fn write_request(&mut self) {
        let stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();
        stream.set_nonblocking(true).unwrap();
        let mut stream = mio::net::TcpStream::from_std(stream);
        stream.write_all(get_req(&self.path).as_bytes()).unwrap();
        self.stream = Some(stream);
    }
}

impl Future for HttpGetFuture {
    type Output = String;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let id = self.id;
        if self.stream.is_none() {
            println!("FIRST POLL - START OPERATION");
            self.write_request();
            let stream = (&mut self).stream.as_mut().unwrap();
            runtime::reactor::reactor()
                .register(stream, Interest::READABLE, id)
                .expect("register");
            runtime::reactor::reactor().set_waker(cx, self.id);
        }

        let mut buf = vec![0u8; 147];
        loop {
            match self.stream.as_mut().unwrap().read(&mut buf) {
                Ok(0) => {
                    let s = String::from_utf8_lossy(&self.buffer).to_string();
                    runtime::reactor::reactor()
                        .deregister(self.stream.as_mut().unwrap(), id)
                        .expect("deregister");
                    break Poll::Ready(s);
                }
                Ok(n) => {
                    self.buffer.extend(&buf[0..n]);
                    continue;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    runtime::reactor::reactor().set_waker(cx, self.id);
                    break Poll::Pending;
                }
                Err(e) => panic!("{e:?}"),
            }
        }
    }
}
