use std::{
    collections::HashSet,
    fs,
    io::{self, Read, Write},
};

use mio::{event::Event, Interest, Poll, Token};

fn main() -> io::Result<()> {
    let mut poll = Poll::new()?;
    let n_events = 5;

    let mut streams = Vec::new();
    let addr = "localhost:8080";

    for i in 0..n_events {
        let delay = n_events * 10000;
        let url_path = format!("/{delay}/request-{i}");
        let request = get_req(&url_path);
        let std_stream = std::net::TcpStream::connect(addr)?;
        std_stream.set_nonblocking(true)?;

        let mut stream = mio::net::TcpStream::from_std(std_stream);

        stream.write_all(request.as_bytes())?;

        poll.registry()
            .register(&mut stream, Token(i), Interest::READABLE)?;

        streams.push(stream);
    }

    let mut handled_ids = HashSet::new();

    let mut handled_events = 0;
    let mut count = 0;
    while handled_events < n_events {
        count += 1;
        if let Some(num_threads) = num_threads() {
            println!("Num threads: {num_threads}");
        }

        let mut events = mio::Events::with_capacity(10);
        println!("Blocking the current thread");
        poll.poll(&mut events, None)?;

        println!("Hello from polling");

        if events.is_empty() {
            println!("TIMEOUT (OR SPURIOUS EVENT NOTIFICATION)");
            continue;
        }

        let events: Vec<Event> = events.into_iter().map(|e| e.clone()).collect();
        println!("Events count: {}", events.len());

        handled_events += handle_events(&events, &mut streams, &mut handled_ids)?;
    }

    println!("Count: {count}");
    println!("FINISHED");

    Ok(())
}

fn num_threads() -> Option<usize> {
    fs::read_to_string("/proc/self/stat")
        .ok()
        .as_ref()
        // Skip past the pid and (process name) fields
        .and_then(|stat| stat.rsplit(')').next())
        // 20th field, less the two we skipped
        .and_then(|rstat| rstat.split_whitespace().nth(17))
        .and_then(|num_threads| num_threads.parse::<usize>().ok())
}

fn handle_events(
    events: &[Event],
    streams: &mut [mio::net::TcpStream],
    handled: &mut HashSet<usize>,
) -> io::Result<usize> {
    let mut handled_events = 0;
    for event in events {
        println!("RECEIVED: {:?}", event);
        let index: usize = event.token().into();

        let mut data = vec![0u8; 4096];
        loop {
            match streams[index].read(&mut data) {
                Ok(n) if n == 0 => {
                    if !handled.insert(index) {
                        break;
                    }
                    handled_events += 1;
                    break;
                }
                Ok(n) => {
                    let txt = String::from_utf8_lossy(&data[..n]);
                    // println!("RECEIVED: {:?}", event);
                    println!("{txt}\n------\n");
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }
    }

    Ok(handled_events)
}

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n
        Host: localhost\r\n
        Connection: close\r\n
        \r\n"
    )
}
