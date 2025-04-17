extern crate ctrlc;
extern crate gpiod;

use std::{env, process::exit, sync::mpsc::{self, Receiver}, thread};

mod status;
mod persist;
use persist::Updater;
mod sensor;
use sensor::Sensor;
use status::DoorStatus;

const DOOR_OPEN_PIN: u32 = 12;
const DOOR_CLOSED_PIN: u32 = 16;

fn logger_thread(mut updater: Updater, rx: Receiver<Option<DoorStatus>>) {
    loop {
        match rx.recv() {
            Ok(Some(status)) => {
                updater.update(status);
            }
            _ => { return }
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = match args.len() {
        0 => None,
        1 => None,
        2 => Some(&args[1]),
        _ => panic!("Too many arguments!"),
    };
    let mut updater = Updater::new(filename);
    let mut sensor = Sensor::new(DOOR_OPEN_PIN, DOOR_CLOSED_PIN);
    let (tx, rx) = mpsc::channel::<Option<DoorStatus>>();
    let handler_tx = tx.clone();

    updater.update(sensor.get_status());

    ctrlc::set_handler(move || { let _ = handler_tx.send(None); })
        .expect("Can't set signal handler");
    thread::spawn(move || { logger_thread(updater, rx); exit(0); });
    loop {
        sensor.get_event().and_then(|status| Some(tx.send(Some(status))));
    }
}