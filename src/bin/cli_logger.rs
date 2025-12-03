//! Whenver a new complete reading is available, logs it *as a whole* to the cli. When the reading didn't change, a
//! simple '.' is appended to the line.
use co2::{MonitorReading, MonitorReadingParts, device::Co2MonitorCommunication, pc::PcCo2Monitor};
use std::io::Write;

fn main() {
    let monitor = PcCo2Monitor::init_and_connect();
    let mut prev_reading = MonitorReading::default();
    let mut partial_reading = MonitorReadingParts::default();
    let program_start = std::time::Instant::now();
    loop {
        if let Ok(Some(reading)) = monitor.read_to_part(&mut partial_reading) {
            if reading != prev_reading {
                println!();
                print!("{:>10.1?} -- {:.1}", program_start.elapsed(), reading);
                prev_reading = reading;
            } else {
                print!(".");
            }
            std::io::stdout().flush().unwrap();
        }
    }
}
