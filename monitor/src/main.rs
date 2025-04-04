extern crate gpiod;
use gpiod::{Chip, Options, Edge, EdgeDetect, Lines, Input};

const DOOR_OPEN_PIN: u32 = 12;
const DOOR_CLOSED_PIN: u32 = 16;

fn initialize() -> Lines<Input> {
    match Chip::new("gpiochip0") {
        Ok(chip ) => {
        let opts = Options::input([DOOR_OPEN_PIN, DOOR_CLOSED_PIN]) // configure lines offsets
            .edge(EdgeDetect::Both) // configure edges to detect
            .consumer("garage-door"); // optionally set consumer string
         match chip.request_lines(opts) {
            Ok(input) => input,
            Err(_) => panic!("Could not start monitoring GPIO")
         }
        }
        Err(_) => panic!("Could not access GPIO"),
    }
}

fn status_change(last: &mut Edge, current: Edge) -> bool {
    if *last == current {
        return false;
    }
    *last = current;
    return true;
}

fn main() -> std::io::Result<()> {
    let mut inputs = initialize();
    let mut last_edge_open = if inputs.get_values([false; 2])?[0] { Edge::Falling } else { Edge::Rising };
    let mut last_edge_closed: Edge = if inputs.get_values([false; 2])?[1] { Edge::Falling } else { Edge::Rising };

    loop {
        let event = inputs.read_event()?;
        let changed: bool = match event.line {
           0 => status_change(&mut last_edge_open, event.edge),
           1 => status_change(&mut last_edge_closed, event.edge),
           _ => false
        };
        if changed {
            match (last_edge_open, last_edge_closed) {
                (Edge::Rising, Edge::Falling) => println!("Status: Door closed"),
                (Edge::Falling, Edge::Rising) => println!("Status: Door open"),
                (Edge::Falling, Edge::Falling) => println!("Status: Door in motion"),
                _ => println!("")
            }
        }

        //println!("event: {:?}", event);
    }

    //Ok(())
}