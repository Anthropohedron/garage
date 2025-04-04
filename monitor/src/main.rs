extern crate syslog;
extern crate gpiod;

use std::process;
use syslog::{Facility, Formatter3164, Logger, LoggerBackend};
use gpiod::{Chip, Options, Edge, EdgeDetect, Lines, Input};

const PROGRAM_NAME: &str = "garagemon";

fn init_logger() -> Logger<LoggerBackend, Formatter3164> {
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: PROGRAM_NAME.into(),
        pid: process::id(),
    };
    return syslog::unix(formatter).expect("Could not set up syslog logging");
}

const DOOR_OPEN_PIN: u32 = 12;
const DOOR_CLOSED_PIN: u32 = 16;

fn init_gpio() -> Lines<Input> {
    let chip = Chip::new("gpiochip0")
        .expect("Could not start monitoring GPIO");
    let opts = Options::input([DOOR_OPEN_PIN, DOOR_CLOSED_PIN]) // configure lines offsets
        .edge(EdgeDetect::Both) // configure edges to detect
        .consumer("garage-door"); // optionally set consumer string
    return chip.request_lines(opts).expect("Could not access GPIO");
}

fn status_change(last: &mut Edge, current: Edge) -> bool {
    if *last == current {
        return false;
    }
    *last = current;
    return true;
}

fn main() -> std::io::Result<()> {
    let mut logger = init_logger();
    let mut inputs = init_gpio();
    let mut last_edge_open = if inputs.get_values([false; 2])?[0] { Edge::Falling } else { Edge::Rising };
    let mut last_edge_closed: Edge = if inputs.get_values([false; 2])?[1] { Edge::Falling } else { Edge::Rising };

    logger.info("Started garagemon").expect("Logging failed");
    loop {
        let event = inputs.read_event()?;
        let changed: bool = match event.line {
           0 => status_change(&mut last_edge_open, event.edge),
           1 => status_change(&mut last_edge_closed, event.edge),
           _ => false
        };
        if changed {
            let _ = match (last_edge_open, last_edge_closed) {
                (Edge::Rising, Edge::Falling) => logger.info("Door closed"),
                (Edge::Falling, Edge::Rising) => logger.info("Door open"),
                (Edge::Falling, Edge::Falling) => logger.info("Door in motion"),
                (Edge::Rising, Edge::Rising) => logger.info("Confusingly open and closed simultaneously")
            };
        }
    }
}
