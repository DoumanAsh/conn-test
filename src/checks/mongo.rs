use core::time;
use core::fmt::Write;

use crate::APP_NAME;
use crate::utils::Url;
use super::{status, CheckResult, CheckResultSender, CheckKind};

use compact_str::CompactString;

pub struct Mongo {
    pub destination: Url,
    pub timeout: time::Duration,
}

impl Mongo {
    pub async fn run(self, result_pipe: CheckResultSender) {
        let Self { destination, timeout } = self;

        let kind = CheckKind::Mongo;
        let mut message = CompactString::new("");
        let log = String::new();

        let options = match mongodb::options::ClientOptions::parse(destination.as_str()).await {
            Ok(mut options) => {
                options.max_connecting = Some(1);
                if options.app_name.is_none() {
                    options.app_name = Some(APP_NAME.to_owned());
                }
                if !timeout.is_zero() {
                    options.connect_timeout = Some(timeout);
                }
                options
            },
            Err(error) => {
                let _ = write!(&mut message, "URL error: {error}");
                let _ = result_pipe.send(CheckResult {
                    kind,
                    status: status::ERROR,
                    destination,
                    message,
                    log,
                }).await;
                return;
            }
        };

        let client = match mongodb::Client::with_options(options) {
            Ok(client) => client,
            Err(error) => {
                let _ = write!(&mut message, "Client error: {error}");
                let _ = result_pipe.send(CheckResult {
                    kind,
                    status: status::ERROR,
                    destination,
                    message,
                    log,
                }).await;
                return;
            }
        };

        match client.list_databases().authorized_databases(true).await {
            Ok(databases) => {
                let _ = write!(&mut message, "Databases: ");
                for database in databases {
                    message.push_str(database.name.as_str());
                    message.push(';');
                }
                let _ = message.pop();
                let _ = result_pipe.send(CheckResult {
                    kind,
                    status: status::OK,
                    destination,
                    message,
                    log,
                }).await;
            },
            Err(error) => {
                let _ = write!(&mut message, "{error}");
                let _ = result_pipe.send(CheckResult {
                    kind,
                    status: status::ERROR,
                    destination,
                    message,
                    log,
                }).await;
            }
        }
        client.shutdown().await;
    }
}
