use std::{io, thread};
use std::process::ExitCode;

use conn_test::{checks, os, output};
use conn_test::utils::Url;

use tokio::sync;

#[derive(Debug, arg::Args)]
#[arg(infer_name, env_prefix = "CONN_TEST", env_value_all)]
///Connectivity testing utility
struct Args {
    #[arg(long, short, default_value = "false")]
    ///Specifies whether to silence verbose logging of errors.
    quiet: bool,
    #[arg(long, default_value = "5")]
    ///Specifies timeout in seconds for TCP connections. Defaults to 5s
    tcp_timeout: u32,
    #[arg(long)]
    ///Hostname alongside port to check by simple TCP connection
    tcp: Vec<Url>,
    #[arg(long, default_value = "5")]
    ///Specifies timeout in seconds for Kafka connections. Defaults to 5s
    kafka_timeout: u32,
    #[arg(long)]
    ///Bootstrap broker to check by performing connection over plain SASL. Credentials can be supplied in URL
    kafka: Vec<Url>,
}

async fn await_done(checks_done: tokio::sync::oneshot::Receiver<()>, term: impl Future<Output = ()>) {
    tokio::select! {
        _ = checks_done => {
        },
        _ = term => {
            println!("INTERRUPT");
        },
    }
}

fn process_checks(args: Args, tasks: sync::mpsc::Sender<checks::Task>, mut results: sync::mpsc::Receiver<checks::CheckResult>, checks_done: sync::oneshot::Sender<()>) {
    for destination in args.tcp {
        let tcp = checks::Tcp {
            timeout: tokio::time::Duration::from_secs(args.tcp_timeout as _),
            destination,
        };
        if tasks.blocking_send(tcp.into()).is_err() {
            break;
        }
    }

    for destination in args.kafka {
        let kafka = checks::Kafka {
            timeout: tokio::time::Duration::from_secs(args.kafka_timeout as _),
            destination,
        };
        if tasks.blocking_send(kafka.into()).is_err() {
            break;
        }
    }

    //Drop tasks pipe to notify tasks dispatcher
    drop(tasks);

    let mut result_out = Vec::new();
    while let Some(result) = results.blocking_recv() {
        if !args.quiet && !result.log.is_empty() {
            println!(">{} task '{}' log:\n{}", result.kind, result.destination, result.log);
        }
        result_out.push(result);
    }

    let tabled = output::tabled(&result_out);
    if let Err(error) = tabled.with(tabled::settings::Style::psql()).build(io::stdout().lock()) {
        eprintln!("Unexpected IO error: {error}");
    }

    let _ = checks_done.send(());
}

fn run(rt: tokio::runtime::LocalRuntime, args: Args) -> ExitCode {
    let (task_sender, task_recv) = sync::mpsc::channel(1);
    let (result_sender, result_recv) = sync::mpsc::channel(1);
    let (checks_done_sender, checks_done_recv) = sync::oneshot::channel();

    let checks_worker = move || process_checks(args, task_sender, result_recv, checks_done_sender);
    let thread = match thread::Builder::new().name("checks-worker".to_owned()).spawn(checks_worker) {
        Ok(thread) => thread,
        Err(error) => {
            eprintln!("Unable to start checks thread: {error}");
            return ExitCode::FAILURE;
        }
    };

    let term = rt.block_on(async {
        os::Termination::new()
    });
    rt.spawn_local(checks::task(task_recv, result_sender));
    match term {
        Ok(term) => rt.block_on(await_done(checks_done_recv, term)),
        Err(error) => {
            eprintln!("Unable to setup signal handler: {error}");
            rt.block_on(await_done(checks_done_recv, core::future::pending::<()>()));
        }
    }

    rt.shutdown_background();
    if let Err(error) = thread.join() {
        let message = panic_ext::downcast_payload(&error);
        eprintln!("Unable to finish checks thread: {message}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn main() -> ExitCode {
    let args = arg::parse_args::<Args>();

    let options = tokio::runtime::LocalOptions::default();
    match tokio::runtime::Builder::new_current_thread().enable_all().build_local(options) {
        Ok(rt) => {
            run(rt, args)
        },
        Err(error) => {
            eprintln!("Failed to create IO runtime: {error}");
            ExitCode::FAILURE
        }
    }
}
