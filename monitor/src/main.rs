extern crate gpiod;

use std::env;

mod status;
mod persist;
use persist::Updater;
mod sensor;
use sensor::Sensor;

const DOOR_OPEN_PIN: u32 = 12;
const DOOR_CLOSED_PIN: u32 = 16;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = match args.len() {
        0 => None,
        1 => Some(&args[0]),
        _ => panic!("Too many arguments!"),
    };
    let mut updater = Updater::new(filename);
    let mut sensor = Sensor::new(DOOR_OPEN_PIN, DOOR_CLOSED_PIN);

    updater.update(sensor.get_status());
    loop {
        sensor.get_event().and_then(|status| Some(updater.update(status)));
    }
}