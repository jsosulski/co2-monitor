//! Whenever a new complete reading is available, logs it *as a whole*. When the sensor reading didn't change, a
//! simple '.' is appended to the line. In addition writes readings to a csv file.
use chrono::{Local, NaiveDateTime};
use co2_monitor::{
    MonitorReading, MonitorReadingParts, device::Co2MonitorCommunication, pc::PcCo2Monitor,
};
use serde::Serialize;
use std::{io::Write, time::Duration};

#[derive(Serialize)]
struct Row {
    timestamp: NaiveDateTime,
    temperature: f32,
    co2_ppm: usize,
    co2_is_valid: bool,
}

fn main() {
    let program_start = std::time::Instant::now();

    use std::fs::OpenOptions;
    const LOG_NAME: &str = "log.csv";

    let log_exists = std::path::Path::new(LOG_NAME).exists();
    if log_exists {
        println!("Appending to existing log file.");
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_NAME)
        .unwrap();

    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(!log_exists)
        .from_writer(file);

    loop {
        let mut heartbeat = std::time::Instant::now();
        let monitor = PcCo2Monitor::init_and_connect();
        let mut prev_reading = MonitorReading::default();
        let mut partial_reading = MonitorReadingParts::default();
        loop {
            if heartbeat.elapsed() > Duration::from_secs(60) {
                println!(
                    "WARNING, there were no readings since at least 60 seconds. Re-starting loop in 10 seconds."
                );
                std::thread::sleep(Duration::from_secs(10));
                break;
            };
            std::thread::sleep(Duration::from_millis(200));
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
                heartbeat = std::time::Instant::now();
            }
        }
    }
}
