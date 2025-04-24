extern crate ctrlc;
extern crate syslog;
extern crate tokio_gpiod;

use actix_web::rt::time::sleep;
use std::{fs, process, sync::{mpsc::{channel, Receiver, Sender}, LazyLock}, thread, time::Duration};
use syslog::{Facility, Formatter3164, Logger, LoggerBackend};
use tokio_gpiod::{Active, Chip, Lines, Options, Output};

const GPIO_DEV: &str = "gpiochip0";
const GPIO_LINE: u32 = 4;
const DELAY_MILLIS: u64 = 500;
const LOG_PROCESS: &str = "garagemon";

pub enum LogEvent {
    Starting, Activated, Exiting
}
pub type LogTx = Sender<LogEvent>;
pub type Syslogger = Logger<LoggerBackend, Formatter3164>;

#[derive(Clone)]
pub struct AppImpl {
    gpio_device: String,
    gpio_line: u32,
    status_filename: String,
    log_tx: LogTx,
}

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

static LOG_TX: LazyLock<Sender<LogEvent>> = LazyLock::new(|| {
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

type OutputResult<'a> = Result<Lines<Output>, &'a str>;
type ActivateResult<'a> = Result<(), &'a str>;

async fn get_outputs(dev: String, line: u32) -> OutputResult<'static> {
    let opts = Options::output([line]) // configure lines offsets
        .active(Active::Low) // configure active to low to operate
        .consumer("garagecontrol"); // optionally set consumer string
    let chip = match Chip::new(dev).await {
        Ok(c) => c,
        Err(_) => return OutputResult::Err("Could not connect to GPIO device"),
    };
    match chip.request_lines(opts).await {
        Err(_) => OutputResult::Err("Could not access GPIO line"),
        Ok(lines) => Ok(lines),
    }
}

impl AppImpl {
    pub fn new(filename: &String) -> Self {
        Self {
            gpio_device: GPIO_DEV.to_string(),
            gpio_line: GPIO_LINE,
            status_filename: filename.clone(),
            log_tx: LOG_TX.to_owned(),
        }
    }

    pub fn get_status(self) -> String {
        match fs::read_to_string(&self.status_filename) {
            Ok(status) => status,
            _ => "Invalid".to_string(),
        }
    }

    pub async fn activate(self) -> ActivateResult<'static> {
        let lines = match get_outputs(self.gpio_device, self.gpio_line).await {
            Ok(output) => output,
            Err(msg) => return ActivateResult::Err(msg),
        };
        let _1 = match lines.set_values([true]).await {
            Err(_) => return ActivateResult::Err("Could not trigger opener"),
            _ => 0,
        };
        let _2 = self.log_tx.send(LogEvent::Activated);
        sleep(Duration::from_millis(DELAY_MILLIS)).await;
        let _3 = match lines.set_values([false]).await {
            Err(_) => return ActivateResult::Err("Could not release opener"),
            _ => 0,
        };
        Ok(())
    }
}
