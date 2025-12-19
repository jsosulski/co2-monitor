//! Whenver a new complete reading is available, logs it *as a whole* to the cli. When the reading didn't change, a
//! simple '.' is appended to the line. In addition writes readings to a csv file.
use chrono::{Local, NaiveDateTime};
use co2::{MonitorReading, MonitorReadingParts, device::Co2MonitorCommunication, pc::PcCo2Monitor};
use serde::Serialize;
use std::io::Write;

#[derive(Serialize)]
struct Row {
    timestamp: NaiveDateTime,
    temperature: f32,
    co2_ppm: usize,
    co2_is_valid: bool,
}

fn main() {
    let monitor = PcCo2Monitor::init_and_connect();
    let mut prev_reading = MonitorReading::default();
    let mut partial_reading = MonitorReadingParts::default();
    let program_start = std::time::Instant::now();
    let mut csv_writer = csv::Writer::from_path("log.csv").unwrap();
    loop {
        if let Ok(Some(reading)) = monitor.read_to_part(&mut partial_reading) {
            let (ppm, valid) = reading.co2_value.as_num_and_bool();
            let now = Local::now().naive_local();
            let row = Row {
                temperature: reading.temperature,
                co2_ppm: ppm as usize,
                co2_is_valid: valid,
                timestamp: now,
            };
            csv_writer.serialize(&row).unwrap();
            if reading != prev_reading {
                println!();
                print!("{:>10.1?} -- {:.1}", program_start.elapsed(), reading);
                prev_reading = reading;
            } else {
                print!(".");
            }
            csv_writer.flush().unwrap();
            std::io::stdout().flush().unwrap();
        }
    }
}
