use core::fmt;

use crate::utils::Url;

use compact_str::CompactString;
use tokio::sync::mpsc;

mod status {
    pub const OK: &str = "OK";
    pub const ERROR: &str = "ERR";
}
pub mod tcp;
pub use tcp::Tcp;

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CheckKind {
    Dns,
    Tcp,
}

impl CheckKind {
    #[inline(always)]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Dns => "DNS",
            Self::Tcp => "TCP",
        }
    }
}

impl fmt::Display for CheckKind {
    #[inline(always)]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(self.as_str())
    }
}

pub struct CheckResult {
    pub kind: CheckKind,
    pub status: &'static str,
    pub destination: Url,
    pub message: CompactString,
    pub log: String,
}

pub enum Task {
    Tcp(tcp::Tcp)
}

pub async fn task(mut task_pipe: mpsc::Receiver<Task>, result_pipe: mpsc::Sender<CheckResult>) {
    while let Some(task) = task_pipe.recv().await {
        match task {
            Task::Tcp(tcp) => {
                tokio::task::spawn(tcp.run(result_pipe.clone()));
            },
        }
    }
}
