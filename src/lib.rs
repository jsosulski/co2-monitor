#![no_std]
pub mod device;

use heapless::LinearMap;
use hidapi::{HidApi, HidDevice};

pub const VID: u16 = 0x04d9;
pub const PID: u16 = 0xa052;

pub trait MonitorComms {
    fn init();
    fn read_timeout(buffer: &mut [u8; 8]);
}

pub enum MonitorValue {
    Temperature(f32),
    Co2(Co2Value),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MonitorReadingParts {
    pub temperature: Option<f32>,
    pub co2_value: Option<u16>,
    pub co2_sanity_check: Option<u16>,
}

impl MonitorReadingParts {
    pub fn set_op_val(&mut self, op: u8, val: u16) {
        const OPCODE_TEMPERATURE: u8 = 0x42;
        const OPCODE_CO2_VALUE: u8 = 0x50;
        const OPCODE_CO2_SANITY_CHECK: u8 = 0x6e;

        match op {
            OPCODE_TEMPERATURE => {
                let temperature_in_kelvin = (val as f32) / 16.0;
                let temperature_in_c = temperature_in_kelvin - 273.15;
                self.temperature = Some(temperature_in_c)
            }
            OPCODE_CO2_VALUE => {
                self.co2_value = Some(val);
            }
            OPCODE_CO2_SANITY_CHECK => {
                // For very large values, sometimes the "actual" co2 code simply reports 1065, even though
                // the diplay indicates "HI". However, there's a second number that decreases with in-
                // creasing CO2 values. It is not quite 1:1, there is some small-ish factor involved,
                // but for now this offset should be enough.
                const MAGIC_OFFSET_THAT_NEEDS_BETTER_ESTIMATE: u16 = 12811;
                self.co2_sanity_check = Some(MAGIC_OFFSET_THAT_NEEDS_BETTER_ESTIMATE - val)
            }
            _ => (),
        }
    }

    pub fn to_reading(&self) -> Option<MonitorReading> {
        if let (Some(t), Some(c), Some(cs)) =
            (self.temperature, self.co2_value, self.co2_sanity_check)
        {
            const SPEC_MAX_CO2_THRESHOLD: u16 = 3000;
            let co2_value = if cs > SPEC_MAX_CO2_THRESHOLD || c > SPEC_MAX_CO2_THRESHOLD {
                Co2Value::TooHigh(c)
            } else {
                Co2Value::Valid(c)
            };
            let mr = MonitorReading {
                temperature: t,
                co2_value,
            };
            return Some(mr);
        }
        None
    }

    fn new() -> Self {
        Self {
            temperature: None,
            co2_value: None,
            co2_sanity_check: None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MonitorReading {
    pub temperature: f32,
    pub co2_value: Co2Value,
}

impl MonitorReading {
    pub fn empty() -> MonitorReading {
        Self {
            temperature: 0.0,
            co2_value: Co2Value::TooHigh(0),
        }
    }
}

impl core::fmt::Display for MonitorReading {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "T:{:.1} Co2:{}", self.temperature, self.co2_value)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Co2Value {
    /// A valid Co2 reading
    Valid(u16),
    /// The sanity check variable indicated that this reading was too high. You might still want to see the actually
    /// read value tho. You do you.
    TooHigh(u16),
}

impl core::fmt::Display for Co2Value {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Co2Value::Valid(n) => write!(f, "{}", n),
            Co2Value::TooHigh(_) => f.write_str("too high"),
        }
    }
}

pub struct Co2Monitor {
    device: HidDevice,
    read_buffer: [u8; 8],
    report_values: LinearMap<u8, u16, 10>,
}

impl Co2Monitor {
    pub fn new() -> Self {
        let api = HidApi::new().expect("Could not initialize Hid Api.");
        let device = api
            .open(VID, PID)
            .expect("Unable to open HID device. Is it connected to this computer?");

        // Online resources have some key or magic table in here, but for my co2 device it works with just zeroes...
        // Sending the feature report is still necessary. Otherwise no HID data will be available.
        let feature_report = [0u8; 9];
        device
            .send_feature_report(&feature_report)
            .expect("Could not send feature report.");

        let read_buffer = [0u8; 8];
        let report_values = LinearMap::new();
        Self {
            device,
            read_buffer,
            report_values,
        }
    }

    pub fn poll(&mut self) -> MonitorReading {
        let mut parts = MonitorReadingParts::new();

        loop {
            match self.device.read_timeout(&mut self.read_buffer, 1000) {
                Ok(8) => {
                    if self.read_buffer[4] != 0x0d {
                        // println!(
                        //     "{} 4-th byte is not 0x0d. But it should be.",
                        //     hex_pretty(&self.read_buffer)
                        // );
                    }
                    if ((self.read_buffer[0] as u16
                        + self.read_buffer[1] as u16
                        + self.read_buffer[2] as u16)
                        & 0xff) as u8
                        != self.read_buffer[3]
                    {
                        // println!("Checksum error. Skipping.");
                    }

                    let op = self.read_buffer[0];
                    let val = ((self.read_buffer[1] as u16) << 8) | self.read_buffer[2] as u16;
                    // This will fill once the report values container is saturated.
                    let _ = self.report_values.insert(op, val);
                    parts.set_op_val(op, val);
                }

                // Too few bytes read. Even though we only need the first 5, it should've been 8.
                Ok(_) => (),
                Err(_e) => {
                    // eprintln!("read error: {}", e);
                }
            }
            if let Some(r) = parts.to_reading() {
                return r;
            }
        }
    }
}

impl Default for Co2Monitor {
    fn default() -> Self {
        Self::new()
    }
}

// pub fn hex_pretty(bytes: &[u8]) -> String {
//     let hex = hex::encode(bytes);
//     let mut count = 0;
//     let mut s = String::with_capacity(64);
//     for c in hex.chars() {
//         s.push(c);
//         count += 1;
//         if count % 2 == 0 {
//             s.push(' ');
//             count = 0;
//         }
//     }
//     s
// }
