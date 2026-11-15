use core::fmt::Write;

use crate::utils::{Url, LookupResult, lookup_host};
use super::{CheckStatus, CheckResult, CheckResultSender, CheckKind};

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
    tcp_msg.pop();

    let _ = result_pipe.send(CheckResult {
        kind: CheckKind::Tcp,
        status: if addrs_ok > 0 {
            CheckStatus::Ok
        } else {
            CheckStatus::Error
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

        match lookup_host(&destination, 80).await {
            Ok(LookupResult::AlreadyAddr(addr)) => {
                run_check_on([addr], destination, timeout, result_pipe).await
            },
            Ok(LookupResult::ResolvedAddrs(addrs)) => {
                let _ = result_pipe.send(CheckResult {
                    kind: CheckKind::Dns,
                    status: CheckStatus::Ok,
                    destination: destination.clone(),
                    message: CompactString::new("Host resolved"),
                    log: String::new()
                }).await;

                run_check_on(addrs, destination, timeout, result_pipe).await
            },
            Err(error) => {
                let mut message = CompactString::new("");
                let _ = write!(&mut message, "{error}");
                let _ = result_pipe.send(CheckResult {
                    kind: CheckKind::Dns,
                    status: CheckStatus::Error,
                    destination,
                    message,
                    log: String::new()
                }).await;
            }
        }
    }
}
