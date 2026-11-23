//! Whenever a new complete reading is available, logs it *as a whole*. When the sensor reading didn't change, a
//! simple '.' is appended to the line. In addition writes readings to a csv file.
use chrono::{Local, NaiveDateTime};
use co2_monitor::{
    MonitorReading, MonitorReadingParts, device::Co2MonitorCommunication, pc::PcCo2Monitor,
};
use serde::Serialize;
use std::{
    io::Write,
    process::{Child, Command},
    time::Duration,
};

/// RAII guard that kills the visualization subprocess when the CLI exits.
struct DashGuard(Child);

impl Drop for DashGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn start_dashboard() -> Option<DashGuard> {
    // Anchor to the repo root (the crate's manifest dir) so this works no matter where the CLI is invoked from.
    let visualize_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("visualize");
    let python = visualize_dir.join(".venv/bin/python");
    match Command::new(python)
        .arg("main.py")
        .current_dir(&visualize_dir)
        .spawn()
    {
        Ok(child) => {
            println!("Starting visual dashboard...");
            Some(DashGuard(child))
        }
        Err(e) => {
            eprintln!(
                "Could not start the visualization script: {}. Continuing without it.",
                e
            );
            None
        }
    }
}

#[derive(Serialize)]
struct Row {
    timestamp: NaiveDateTime,
    temperature: f32,
    co2_ppm: usize,
    co2_is_valid: bool,
}

fn main() {
    let program_start = std::time::Instant::now();

    let with_dash = std::env::args().skip(1).any(|arg| arg == "--with-dash");
    let _dash_guard = with_dash.then(start_dashboard).flatten();

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
