use core::fmt::Write;

use crate::utils::Url;
use super::{status, CheckResult, CheckResultSender, CheckKind};

use compact_str::CompactString;
use tokio::{time, net};
use tokio::io::AsyncWriteExt;

async fn run_check_on(addrs: impl IntoIterator<Item = std::net::SocketAddr>, destination: Url, timeout: time::Duration, result_pipe: CheckResultSender) {
    let mut tcp_msg = CompactString::new("");
    let mut tcp_log = String::new();
    let mut addrs_ok = 0u16;
    for addr in addrs {
        let _ = write!(&mut tcp_msg, "{}", addr.ip());
        match time::timeout(timeout, net::TcpStream::connect(addr)).await {
            Ok(Ok(mut socket)) => {
                let _ = write!(&mut tcp_msg, "=OK;");
                let _ = socket.shutdown().await;
                addrs_ok = addrs_ok.saturating_add(1);
            },
            Ok(Err(error)) => {
                let _ = write!(&mut tcp_msg, "=ERR;");
                let _ = writeln!(&mut tcp_log, "{addr}: Failed to connect: {error}");
            },
            Err(_elapsed) => {
                let _ = write!(&mut tcp_msg, "=TIMEOUT;");
                let _ = writeln!(&mut tcp_log, "{addr}: Unable to connect in time");
            }
        }
    }

    let _ = result_pipe.send(CheckResult {
        kind: CheckKind::Tcp,
        status: if addrs_ok > 0 {
            status::OK
        } else {
            status::ERROR
        },
        destination,
        message: tcp_msg,
        log: tcp_log,
    }).await;
}

pub struct Tcp {
    pub destination: Url,
    pub timeout: time::Duration,
}

impl Tcp {
    pub async fn run(self, result_pipe: CheckResultSender) {
        let Self { destination, timeout } = self;

        if let Ok(addr) = destination.host().parse::<std::net::IpAddr>() {
            let port = destination.port_u16().unwrap_or(80);
            return run_check_on([std::net::SocketAddr::new(addr, port)], destination, timeout, result_pipe).await;
        }

        let mut host = destination.host().to_owned();
        host.push(':');
        if let Some(port) = destination.port() {
            host.push_str(port);
        } else {
            host.push_str("80");
        }

        let addrs = match tokio::net::lookup_host(host).await {
            Ok(addrs) => addrs,
            Err(error) => {
                let mut message = CompactString::new("Lookup failed: ");
                let _ = write!(&mut message, "{error}");
                let _ = result_pipe.send(CheckResult {
                    kind: CheckKind::Dns,
                    status: status::ERROR,
                    destination,
                    message,
                    log: String::new()
                }).await;
                return;
            }
        };

        let _ = result_pipe.send(CheckResult {
            kind: CheckKind::Dns,
            status: status::OK,
            destination: destination.clone(),
            message: CompactString::new("Host resolved"),
            log: String::new()
        }).await;

        run_check_on(addrs, destination, timeout, result_pipe).await;
    }
}
