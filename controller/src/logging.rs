extern crate ctrlc;
extern crate syslog;

use syslog::{Facility, Formatter3164, Logger, LoggerBackend};
use std::{process, sync::{mpsc::{channel, Receiver, Sender}, LazyLock}, thread};

const LOG_PROCESS: &str = "garagemon";

pub enum LogEvent {
    Starting, Activated, Exiting
}
pub type LogTx = Sender<LogEvent>;

type Syslogger = Logger<LoggerBackend, Formatter3164>;
fn logger_thread(mut logger: Syslogger, rx: Receiver<LogEvent>) {
    loop {
        match rx.recv() {
            Ok(LogEvent::Starting) => {
                let _ = logger.info("Starting garagecontrol");
            },
            Ok(LogEvent::Activated) => {
                let _ = logger.info("Activated garage door opener");
            },
            _ => {
                let _ = logger.info("Exiting garagecontrol");
                return;
            }
        }
    }
}

pub static LOG_TX: LazyLock<LogTx> = LazyLock::new(|| {
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: LOG_PROCESS.into(),
        pid: process::id(),
    };
    let logger = syslog::unix(formatter)
        .expect("Could not set up syslog logging");
    let (tx, rx) = channel::<LogEvent>();
    let handler_tx = tx.clone();

    ctrlc::set_handler(move || {
        let _ = handler_tx.send(LogEvent::Exiting);
    })
    .expect("Can't set signal handler");
    thread::spawn(|| {
        logger_thread(logger, rx);
        process::exit(0);
    });
    tx.send(LogEvent::Starting)
    .expect("Could not log to syslog");
    tx
});
