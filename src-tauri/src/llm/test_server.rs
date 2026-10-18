//! A small HTTP server on 127.0.0.1 for provider tests: it answers each request with the next
//! canned response and records what it received.

use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub struct Reply {
    status: u16,
    content_type: &'static str,
    body: String,
}

impl Reply {
    pub fn json(status: u16, body: &str) -> Self {
        Self {
            status,
            content_type: "application/json",
            body: body.to_string(),
        }
    }

    /// A streamed body (SSE).
    pub fn stream(content_type: &'static str, body: &str) -> Self {
        Self {
            status: 200,
            content_type,
            body: body.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Request {
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: serde_json::Value,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

pub struct TestServer {
    pub url: String,
    requests: Arc<Mutex<Vec<Request>>>,
    thread: Option<JoinHandle<()>>,
}

/// How long the server waits for each expected request. A client that sends fewer requests
/// than expected makes the test fail (missing requests) instead of hanging.
const WAIT: Duration = Duration::from_secs(10);

impl TestServer {
    /// Serves `replies` in order, one connection each, then stops.
    pub fn start(replies: Vec<Reply>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind a local port");
        listener
            .set_nonblocking(true)
            .expect("make the listener non-blocking");
        let url = format!(
            "http://{}",
            listener.local_addr().expect("bound socket has an address")
        );
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = requests.clone();
        let thread = std::thread::spawn(move || {
            for reply in replies {
                let Some(stream) = accept_within(&listener, WAIT) else {
                    return;
                };
                if let Some(request) = serve(stream, &reply) {
                    recorded.lock().unwrap().push(request);
                }
            }
        });
        Self {
            url,
            requests,
            thread: Some(thread),
        }
    }

    /// The requests received so far, after waiting for every reply to be sent.
    pub fn finish(mut self) -> Vec<Request> {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        self.requests.lock().unwrap().clone()
    }
}

/// An address nothing listens on: port 1 (tcpmux) is never served on a desktop, and unlike a
/// freshly released port no other test's server can take it.
pub fn closed_port_url() -> String {
    "http://127.0.0.1:1".to_string()
}

fn accept_within(listener: &TcpListener, wait: Duration) -> Option<TcpStream> {
    let deadline = Instant::now() + wait;
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(false).ok()?;
                stream.set_read_timeout(Some(wait)).ok()?;
                return Some(stream);
            }
            Err(err) if err.kind() == ErrorKind::WouldBlock && Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(_) => return None,
        }
    }
}

/// Reads one request and answers it. The body goes out in two parts, split mid-way (possibly
/// inside a UTF-8 character), so clients must join frames across chunks.
fn serve(stream: TcpStream, reply: &Reply) -> Option<Request> {
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).ok()?;
    let path = request_line.split_whitespace().nth(1)?.to_string();

    let mut headers = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).ok()?;
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_string(), value.trim().to_string()));
        }
    }
    let length = headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.parse::<usize>().ok())
        .unwrap_or(0);
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body).ok()?;

    let mut out = stream;
    let head = format!(
        "HTTP/1.1 {} X\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        reply.status,
        reply.content_type,
        reply.body.len()
    );
    out.write_all(head.as_bytes()).ok()?;
    let bytes = reply.body.as_bytes();
    let (first, rest) = bytes.split_at(bytes.len() / 2);
    out.write_all(first).ok()?;
    out.flush().ok()?;
    std::thread::sleep(Duration::from_millis(20));
    out.write_all(rest).ok()?;
    out.flush().ok()?;

    Some(Request {
        path,
        headers,
        body: serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null),
    })
}
