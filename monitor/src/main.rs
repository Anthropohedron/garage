extern crate gpiod;

use gpiod::{Chip, Options, Edge, EdgeDetect, Lines, Input};

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
    let mut inputs = init_gpio();
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
            let _ = match (last_edge_open, last_edge_closed) {
                (Edge::Rising, Edge::Falling) => println!("Status: Door closed"),
                (Edge::Falling, Edge::Rising) => println!("Status: Door open"),
                (Edge::Falling, Edge::Falling) => println!("Status: Door in motion"),
                (Edge::Rising, Edge::Rising) => println!("Confusingly open and closed simultaneously"),
            };
        }
    }

    //Ok(())
}