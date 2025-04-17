extern crate syslog;

use std::{fs, process};
use syslog::{Facility, Formatter3164, Logger, LoggerBackend};
use crate::status::{STATUS, DoorStatus};

const PROGRAM_NAME: &str = "garagemon";
const DEFAULT_STATUS_FILENAME: &str = "/var/run/garagemon_status";

pub struct Updater {
    status_filename: String,
    status_filename_new: String,
    logger: Syslogger,
}
pub type Syslogger = Logger<LoggerBackend, Formatter3164>;

impl Updater {
    pub fn new(status_filename: Option<&String>) -> Self {
        let formatter = Formatter3164 {
            facility: Facility::LOG_USER,
            hostname: None,
            process: PROGRAM_NAME.into(),
            pid: process::id(),
        };
        let mut logger = syslog::unix(formatter).expect("Could not set up syslog logging");
        logger
            .info(format!("Started {PROGRAM_NAME}"))
            .expect("Logging failed");
        let default_filename = DEFAULT_STATUS_FILENAME.to_string();
        let filename = status_filename.unwrap_or(&default_filename).clone();
        Updater {
            status_filename_new: format!("{filename}.new"),
            status_filename: filename,
            logger: logger,
        }
    }

    fn write_file(&self, status: &DoorStatus) {
        let _ = fs::write(&self.status_filename_new, STATUS[status])
            .and(fs::rename(&self.status_filename_new, &self.status_filename));
    }

    pub fn update(&mut self, status: DoorStatus) {
        self.write_file(&status);
        let _ = match status {
            DoorStatus::Closed => self.logger.info("Door closed"),
            DoorStatus::Open => self.logger.info("Door open"),
            DoorStatus::Indeterminate => self.logger.info("Door in motion"),
            DoorStatus::Invalid => self
                .logger
                .info("Confusingly open and closed simultaneously"),
        };
    }
}

impl Drop for Updater {
    fn drop(&mut self) {
        let _ = self.logger.info(format!("Exiting {PROGRAM_NAME}"));
    }
}