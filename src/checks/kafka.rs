use core::time;
use core::fmt::Write;

use crate::utils::Url;
use super::{status, CheckResult, CheckResultSender, CheckKind};

use compact_str::CompactString;

pub struct Kafka {
    pub destination: Url,
    pub timeout: time::Duration,
}

impl Kafka {
    pub async fn run(self, result_pipe: CheckResultSender) {
        let Self { destination, timeout } = self;
        let servers = vec![
            destination.host_port().to_owned()
        ];
        let deadline = if timeout.is_zero() {
            None
        } else {
            Some(timeout)
        };
        let backoff = rskafka::BackoffConfig {
            deadline,
            ..Default::default()
        };
        let builder = rskafka::client::ClientBuilder::new(servers).client_id("conn-test").backoff_config(backoff);

        let result = if let Some(user) = destination.username() {
            let creds = rskafka::client::Credentials {
                username: user.to_owned(),
                password: destination.password().unwrap_or("").to_owned()
            };
            builder.sasl_config(rskafka::client::SaslConfig::Plain(creds)).build()
        } else {
            builder.build()
        };

        match result.await {
            Ok(client) => match client.list_topics().await {
                Ok(topics) => {
                    let mut log = String::new();
                    let mut topics_num = 0u32;
                    let mut partitions_num = 0u32;

                    let _ = writeln!(&mut log, "{} topics:", topics.len());
                    for topic in topics.iter() {
                        let _ = writeln!(&mut log, "  topic '{}' with {} partitions:", topic.name, topic.partitions.len());
                        topics_num = topics_num.saturating_add(1);
                        partitions_num = partitions_num.saturating_add(topic.partitions.len() as _);
                    }

                    let mut message = CompactString::new("Topics/Partitions count: ");
                    let _ = write!(&mut message, "{topics_num}/{partitions_num}");
                    let _ = result_pipe.send(CheckResult {
                        kind: CheckKind::Kafka,
                        status: status::OK,
                        destination,
                        message,
                        log: String::new()
                    }).await;
                },
                Err(error) => {
                    let mut message = CompactString::new("Metadata read error: ");
                    let _ = write!(&mut message, "{error}");
                    let _ = result_pipe.send(CheckResult {
                        kind: CheckKind::Kafka,
                        status: status::ERROR,
                        destination,
                        message,
                        log: String::new()
                    }).await;
                }
            },
            Err(error) => {
                let mut message = CompactString::new("Connection failed: ");
                let _ = write!(&mut message, "{error}");
                let _ = result_pipe.send(CheckResult {
                    kind: CheckKind::Kafka,
                    status: status::ERROR,
                    destination,
                    message,
                    log: String::new()
                }).await;
            }
        }
    }
}
