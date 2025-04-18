extern crate tokio_gpiod;

use actix_web::rt::time::sleep;
use tokio_gpiod::{Active, Chip, Lines, Options, Output};
use std::{fs, time::Duration};

const GPIO_DEV: &str = "gpiochip0";
const GPIO_LINE: u32 = 4;
const DELAY_MILLIS: u64 = 500;

#[derive(Clone)]
pub struct AppImpl {
    gpio_device: String,
    gpio_line: u32,
    status_filename: String
}

type OutputResult<'a> = Result<Lines<Output>, &'a str>;
type ActivateResult<'a> = Result<(), &'a str>;

async fn get_outputs(dev: String, line: u32) -> OutputResult<'static> {
    let opts = Options::output([line]) // configure lines offsets
        .active(Active::Low) // configure active to low to operate
        .consumer("garagecontrol"); // optionally set consumer string
    let chip = match Chip::new(dev).await {
        Ok(c) => c,
        Err(_) => return OutputResult::Err("Could not connect to GPIO device")
    };
    match chip.request_lines(opts).await {
        Err(_) => OutputResult::Err("Could not access GPIO line"),
        Ok(lines) => Ok(lines)
    }
}

impl AppImpl {
    pub fn new(filename: &String) -> Self {
        Self {
            gpio_device: GPIO_DEV.to_string(),
            gpio_line: GPIO_LINE,
            status_filename: filename.clone()
        }
    }
    
    pub fn get_status(self) -> String {
        match fs::read_to_string(&self.status_filename) {
            Ok(status) => status,
            _ => "Invalid".to_string()
        }
    }

    pub async fn activate(self) -> ActivateResult<'static> {
        let lines = match get_outputs(self.gpio_device, self.gpio_line).await {
            Ok(output) => output,
            Err(msg) => return ActivateResult::Err(msg)
        };
        let _1 = match lines.set_values([true]).await {
            Err(_) => return ActivateResult::Err("Could not trigger opener"),
            _ => 0
        };
        sleep(Duration::from_millis(DELAY_MILLIS)).await;
        let _2 = match lines.set_values([false]).await {
            Err(_) => return ActivateResult::Err("Could not release opener"),
            _ => 0
        };
        Ok(())
    }
}
