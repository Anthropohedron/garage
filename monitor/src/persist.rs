extern crate syslog;

use std::process;
use syslog::{Facility, Formatter3164, Logger, LoggerBackend};

const PROGRAM_NAME: &str = "garagemon";

pub enum DoorStatus {
    Closed, Open, Indeterminate, Invalid
}

pub struct Updater {
    logger: Syslogger,
}
pub type Syslogger = Logger<LoggerBackend, Formatter3164>;

impl Updater {
    pub fn new() -> Self {
        let formatter = Formatter3164 {
            facility: Facility::LOG_USER,
            hostname: None,
            process: PROGRAM_NAME.into(),
            pid: process::id(),
        };
        let mut logger = syslog::unix(formatter)
            .expect("Could not set up syslog logging");
        logger.info("Started {PROGRAM_NAME}").expect("Logging failed");
        Updater {
            logger: logger
        }
    }
    
    pub fn update(&mut self, status: DoorStatus) {
        let _ = match status {
            DoorStatus::Closed => {
                self.logger.info("Door closed")
            },
            DoorStatus::Open => {
                self.logger.info("Door open")
            },
            DoorStatus::Indeterminate => {
                self.logger.info("Door in motion")
            },
            DoorStatus::Invalid => {
                self.logger.info("Confusingly open and closed simultaneously")
            }
        };
    }
}