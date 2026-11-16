use core::time;
use core::fmt::Write;

use compact_str::CompactString;

use crate::utils::{net, Url, LookupResult, lookup_host};
use super::{tcp, CheckStatus, CheckResult, CheckResultSender, CheckKind};

const PING: &str = "*1\r\n$4\r\nPING\r\n";

pub struct Redis {
    pub destination: Url,
    pub timeout: time::Duration,
}

impl Redis {
    pub async fn run(self, result_pipe: CheckResultSender) {
        let Self { destination, timeout } = self;

        let socket = match lookup_host(&destination, 6379).await {
            Ok(LookupResult::AlreadyAddr(addr)) => tcp::connect_to_one_addr([addr], timeout).await,
            Ok(LookupResult::ResolvedAddrs(addrs)) => {
                let _ = result_pipe.send(CheckResult {
                    kind: CheckKind::Dns,
                    status: CheckStatus::Ok,
                    destination: destination.clone(),
                    message: CompactString::new("Host resolved"),
                    log: String::new()
                }).await;

                tcp::connect_to_one_addr(addrs, timeout).await
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
                return
            },
        };

        let socket = match socket {
            Some(Ok(socket)) => {
                let mut message = CompactString::new("");
                if let Ok(addr) = socket.peer_addr() {
                    let _ = write!(&mut message, "{addr}=");
                }
                message.push_str("OK");
                let _ = result_pipe.send(CheckResult {
                    kind: CheckKind::Tcp,
                    status: CheckStatus::Ok,
                    destination: destination.clone(),
                    message,
                    log: String::new()
                }).await;

                socket
            },
            Some(Err(error)) => {
                let mut message = CompactString::new("");
                let _ = write!(&mut message, "{error}");
                let _ = result_pipe.send(CheckResult {
                    kind: CheckKind::Tcp,
                    status: CheckStatus::Error,
                    destination,
                    message,
                    log: String::new()
                }).await;
                return
            },
            None => {
                let _ = result_pipe.send(CheckResult {
                    kind: CheckKind::Tcp,
                    status: CheckStatus::Error,
                    destination,
                    message: CompactString::new("No IP available to connect"),
                    log: String::new()
                }).await;
                return
            },
        };

        let client = match destination.schema() {
            Some("rediss") => match net::TlsConnector::new(&destination) {
                Ok(connector) => match net::TcpClient::tls(socket, connector, timeout).await {
                    Ok(client) => {
                        let _ = result_pipe.send(CheckResult {
                            kind: CheckKind::Tls,
                            status: CheckStatus::Ok,
                            destination: destination.clone(),
                            message: CompactString::new(""),
                            log: String::new()
                        }).await;
                        client
                    },
                    Err(error) => {
                        let mut message = CompactString::new("");
                        let _ = write!(&mut message, "{error}");
                        let _ = result_pipe.send(CheckResult {
                            kind: CheckKind::Tls,
                            status: CheckStatus::Error,
                            destination,
                            message,
                            log: String::new()
                        }).await;
                        return
                    }
                }
                Err(_) => {
                    let _ = result_pipe.send(CheckResult {
                        kind: CheckKind::Tls,
                        status: CheckStatus::Error,
                        destination,
                        message: CompactString::new("Invalid server name"),
                        log: String::new()
                    }).await;
                    return
                }
            },
            _ => net::TcpClient::plain(socket),
        };

        let mut client = client.buffered::<128>(timeout);
        let mut buffer = CompactString::new("");
        let status = if let Some(pass) = destination.password() {
            let pass_len = pass.len();
            let _ = if let Some(user) = destination.username() {
                let user_len = user.len();
                write!(&mut buffer, "*3\r\n$4\r\nAUTH\r\n${user_len}\r\n{user}\r\n${pass_len}\r\n{pass}\r\n")
            } else {
                write!(&mut buffer, "*2\r\n$4\r\nAUTH\r\n${pass_len}\r\n{pass}\r\n")
            };

            match client.write_all(buffer.as_bytes()).await {
                Ok(()) => match client.read_str().await {
                    Ok("+OK") => {
                        buffer.clear();
                        buffer.push_str("Authenticated");
                        CheckStatus::Ok
                    },
                    Ok(unexpected) => {
                        buffer.clear();
                        let _ = write!(&mut buffer, "{unexpected}");
                        CheckStatus::Error
                    },
                    Err(error) => {
                        buffer.clear();
                        let _ = write!(&mut buffer, "Read failed: {error}");
                        CheckStatus::Error
                    }
                },
                Err(error) => {
                    buffer.clear();
                    let _ = write!(&mut buffer, "Write failed: {error}");
                    CheckStatus::Error
                }
            }
        } else {
            match client.write_all(PING.as_bytes()).await {
                Ok(()) => match client.read_str().await {
                    Ok("+PONG") => {
                        buffer.clear();
                        CheckStatus::Ok
                    },
                    Ok(unexpected) => {
                        buffer.clear();
                        let _ = write!(&mut buffer, "{unexpected}");
                        CheckStatus::Error
                    },
                    Err(error) => {
                        buffer.clear();
                        let _ = write!(&mut buffer, "Read failed: {error}");
                        CheckStatus::Error
                    }
                },
                Err(error) => {
                    buffer.clear();
                    let _ = write!(&mut buffer, "Write failed: {error}");
                    CheckStatus::Error
                }
            }
        };

        let _ = result_pipe.send(CheckResult {
            kind: CheckKind::Redis,
            status,
            destination,
            message: buffer,
            log: String::new()
        }).await;

        client.shutdown().await;
    }
}
