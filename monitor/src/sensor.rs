extern crate gpiod;

use crate::status::DoorStatus;
use gpiod::{Bias, Chip, Edge, EdgeDetect, Input, Lines, Options};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SensorStates {
    open_sensor: Edge,
    closed_sensor: Edge,
}

#[derive(Debug)]
enum WhichSensor {
    Open,
    Closed,
    Invalid,
}

pub struct Sensor {
    inputs: Lines<Input>,
    last_state: SensorStates,
    last_status: DoorStatus,
}

use std::{collections::HashMap, sync::LazyLock};

pub static STATUS_MAP: LazyLock<HashMap<SensorStates, DoorStatus>> = LazyLock::new(|| {
    let closed = SensorStates {
        open_sensor: Edge::Falling,
        closed_sensor: Edge::Rising,
    };
    let open = SensorStates {
        open_sensor: Edge::Rising,
        closed_sensor: Edge::Falling,
    };
    let invalid = SensorStates {
        open_sensor: Edge::Falling,
        closed_sensor: Edge::Falling,
    };
    let indeterminate = SensorStates {
        open_sensor: Edge::Rising,
        closed_sensor: Edge::Rising,
    };
    HashMap::from([
        (open, DoorStatus::Closed),
        (closed, DoorStatus::Open),
        (invalid, DoorStatus::Invalid),
        (indeterminate, DoorStatus::Indeterminate),
    ])
});

const GPIO_DEV: &str = "gpiochip0";

fn single_status_change(last: &mut Edge, current: Edge) -> bool {
    if *last == current {
        return false;
    }
    *last = current;
    return true;
}

fn status_change(last: &mut SensorStates, sensor: WhichSensor, current: Edge) -> bool {
    match sensor {
        WhichSensor::Open => single_status_change(&mut last.open_sensor, current),
        WhichSensor::Closed => single_status_change(&mut last.closed_sensor, current),
        WhichSensor::Invalid => false,
    }
}

impl Sensor {
    pub fn new(door_open_pin: u32, door_closed_pin: u32) -> Self {
        let chip = Chip::new(GPIO_DEV).expect("Could not connect to GPIO");
        let opts = Options::input([door_open_pin, door_closed_pin]) // configure lines offsets
            .bias(Bias::PullUp) // configure bias to maintain current
            .edge(EdgeDetect::Both) // configure edges to detect
            .consumer("garagemon"); // optionally set consumer string
        let lines = chip.request_lines(opts).expect("Could not access GPIO");
        let values = lines
            .get_values([false; 2])
            .expect("Cannot read GPIO values");
        let last_edge_open = if values[0] {
            Edge::Rising
        } else {
            Edge::Falling
        };
        let last_edge_closed = if values[1] {
            Edge::Rising
        } else {
            Edge::Falling
        };
        let last_state = SensorStates {
            open_sensor: last_edge_open,
            closed_sensor: last_edge_closed,
        };
        let last_status = STATUS_MAP[&last_state];
        return Self {
            inputs: lines,
            last_state: last_state,
            last_status: last_status,
        };
    }

    pub fn get_status(&self) -> DoorStatus {
        self.last_status
    }

    pub fn get_event(&mut self) -> Option<DoorStatus> {
        let event = match self.inputs.read_event() {
            Ok(e) => e,
            _ => return None,
        };
        let which: WhichSensor = match event.line {
            0 => WhichSensor::Open,
            1 => WhichSensor::Closed,
            _ => WhichSensor::Invalid,
        };
        let changed: bool = status_change(&mut self.last_state, which, event.edge);
        if changed {
            self.last_status = STATUS_MAP[&self.last_state];
            return Some(self.last_status);
        }
        return None;
    }
}
