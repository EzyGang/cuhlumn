//! The `healthcheck` task: one HTTP request to the local server, for container health checks.

use std::{
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpStream},
    time::Duration,
};

const TIMEOUT: Duration = Duration::from_secs(3);
const RESPONSE_MAX: u64 = 64 * 1024;

/// Sends `GET /api/health` to `127.0.0.1:port` and returns `true` for HTTP 200 with `"ok":true`.
///
/// It does not open the data folder.
#[must_use]
pub fn run(port: u16) -> bool {
    match request(port) {
        Ok(response) => is_healthy(&response),
        Err(err) => {
            tracing::error!(port, error = %err, "the health request failed");
            false
        }
    }
}

fn request(port: u16) -> io::Result<String> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let mut stream = TcpStream::connect_timeout(&address, TIMEOUT)?;
    stream.set_read_timeout(Some(TIMEOUT))?;
    stream.set_write_timeout(Some(TIMEOUT))?;
    let request = format!(
        "GET /api/health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes())?;
    let mut response = String::new();
    stream.take(RESPONSE_MAX).read_to_string(&mut response)?;
    Ok(response)
}

/// Returns `true` for an HTTP 200 response whose body has `"ok":true`.
fn is_healthy(response: &str) -> bool {
    let Some((head, body)) = response.split_once("\r\n\r\n") else {
        return false;
    };
    let status_ok = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        == Some("200");
    status_ok && body.contains("\"ok\":true")
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    use crate::healthcheck::{is_healthy, run};

    #[test]
    fn reads_status_and_body() {
        let ok = "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\r\n{\"ok\":true,\"version\":\"1\"}";
        assert!(is_healthy(ok));
        assert!(!is_healthy(
            "HTTP/1.1 503 Service Unavailable\r\n\r\n{\"ok\":true}"
        ));
        assert!(!is_healthy("HTTP/1.1 200 OK\r\n\r\n{\"ok\":false}"));
        assert!(!is_healthy("garbage"));
    }

    #[test]
    fn asks_the_local_port() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut chunk = [0_u8; 256];
            while !request.ends_with(b"\r\n\r\n") {
                let read = socket.read(&mut chunk).unwrap();
                assert!(read > 0, "the request ended early");
                request.extend_from_slice(&chunk[..read]);
            }
            assert!(request.starts_with(b"GET /api/health HTTP/1.1\r\n"));
            socket
                .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 11\r\n\r\n{\"ok\":true}")
                .unwrap();
        });
        assert!(run(port));
        server.join().unwrap();
    }
}
