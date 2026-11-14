use core::fmt;
use core::future::Future;

use crate::utils::Url;

use compact_str::CompactString;
use tokio::sync::mpsc;

mod status {
    pub const OK: &str = "OK";
    pub const ERROR: &str = "ERR";
}
pub mod tcp;
pub use tcp::Tcp;
pub mod kafka;
pub use kafka::Kafka;
pub mod mongo;
pub use mongo::Mongo;

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CheckKind {
    Dns,
    Tcp,
    Kafka,
    Mongo,
}

impl CheckKind {
    #[inline(always)]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Dns => "DNS",
            Self::Tcp => "TCP",
            Self::Kafka => "KAFKA",
            Self::Mongo => "MONGO",
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

pub struct CheckResultSender(mpsc::Sender<CheckResult>);

impl CheckResultSender {
    #[inline]
    fn send(&self, mut result: CheckResult) -> impl Future<Output = Result<(), mpsc::error::SendError<CheckResult>>> + Send {
        result.destination.make_non_sensitive();
        self.0.send(result)
    }
}

macro_rules! impl_task {
    ($($name:ident),+ $(,)?) => {

        pub enum Task {
            $(
                $name($name),
            )+
        }

        $(
            impl From<$name> for Task {
                #[inline(always)]
                fn from(task: $name) -> Self {
                    Self::$name(task)
                }
            }
        )+
    };
}

impl_task!(Tcp, Kafka, Mongo);

pub async fn task(mut task_pipe: mpsc::Receiver<Task>, result_pipe: mpsc::Sender<CheckResult>) {
    while let Some(task) = task_pipe.recv().await {
        match task {
            Task::Tcp(task) => {
                tokio::task::spawn(task.run(CheckResultSender(result_pipe.clone())));
            },
            Task::Kafka(task) => {
                tokio::task::spawn(task.run(CheckResultSender(result_pipe.clone())));
            },
            Task::Mongo(task) => {
                tokio::task::spawn(task.run(CheckResultSender(result_pipe.clone())));
            },
        }
    }
}
