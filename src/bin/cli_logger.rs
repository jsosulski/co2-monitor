use co2::{Co2Monitor, MonitorReading};

fn main() {
    let mut monitor = Co2Monitor::new();
    let mut prev_reading = MonitorReading::empty();
    let program_start = std::time::Instant::now();
    loop {
        let reading = monitor.poll();
        if reading != prev_reading {
            println!("{:>10.1?} -- {:.1}", program_start.elapsed(), reading);
            prev_reading = reading;
        }
    }
}
