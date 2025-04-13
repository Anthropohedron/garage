extern crate gpiod;

use gpiod::{Bias, Chip, Edge, EdgeDetect, Input, Lines, Options};
use crate::status::DoorStatus;

pub struct Sensor {
    inputs: Lines<Input>,
    last_edge_open: Edge,
    last_edge_closed: Edge,
    last_status: DoorStatus,
}

use std::{collections::HashMap, sync::LazyLock};

pub static STATUS_MAP: LazyLock<HashMap<(Edge, Edge), DoorStatus>> = LazyLock::new(|| {
    HashMap::from([
        ((Edge::Rising, Edge::Falling), DoorStatus::Closed),
        ((Edge::Falling, Edge::Rising), DoorStatus::Open),
        ((Edge::Falling, Edge::Falling), DoorStatus::Indeterminate),
        ((Edge::Rising, Edge::Rising), DoorStatus::Invalid),
    ])
});

fn status_change(last: &mut Edge, current: Edge) -> bool {
    if *last == current {
        return false;
    }
    *last = current;
    return true;
}

impl Sensor {
    pub fn new(door_open_pin: u32, door_closed_pin: u32) -> Self {
        let chip = Chip::new("gpiochip0").expect("Could not start monitoring GPIO");
        let opts = Options::input([door_open_pin, door_closed_pin]) // configure lines offsets
            .bias(Bias::PullUp) // configure bias to maintain current
            .edge(EdgeDetect::Both) // configure edges to detect
            .consumer("garage-door"); // optionally set consumer string
        let lines = chip.request_lines(opts).expect("Could not access GPIO");
        let values = lines.get_values([false; 2]).expect("Cannot read GPIO values");
        let last_edge_open = if values[0] {
            Edge::Falling
        } else {
            Edge::Rising
        };
        let last_edge_closed = if values[1] {
            Edge::Falling
        } else {
            Edge::Rising
        };
        let last_status = STATUS_MAP[&(last_edge_open, last_edge_closed)];
        return Self {
            inputs: lines,
            last_edge_open: last_edge_open,
            last_edge_closed: last_edge_closed,
            last_status: last_status
        }
    }

    pub fn get_status(&self) -> DoorStatus {
        self.last_status
    }

    pub fn get_event(&mut self) -> Option<DoorStatus> {
        let event = match self.inputs.read_event() {
            Ok(e) => e,
            _ => return None
        };
        let changed: bool = match event.line {
            0 => status_change(&mut self.last_edge_open, event.edge),
            1 => status_change(&mut self.last_edge_closed, event.edge),
            _ => false,
        };
        if changed {
            self.last_status = STATUS_MAP[&(self.last_edge_open, self.last_edge_closed)];
            return Some(self.last_status);
        }
        return None;
    }
}