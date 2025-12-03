//! The actual code for converting the HID reports between byte values and useful numbers.
#![no_std]

// use heapless::{String, Vec};
pub mod device;
#[cfg(feature = "pc")]
pub mod pc;

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

pub enum MonitorReportRaw {
    Temperature(u16),
    Co2Value(u16),
    Co2SanityCheck(u16),
    Unknown(u8, u16),
}

pub const OPCODE_TEMPERATURE: u8 = 0x42;
pub const OPCODE_CO2_VALUE: u8 = 0x50;
pub const OPCODE_CO2_SANITY_CHECK: u8 = 0x6e;

impl From<(u8, u16)> for MonitorReportRaw {
    fn from((op, val): (u8, u16)) -> Self {
        match op {
            OPCODE_TEMPERATURE => Self::Temperature(val),
            OPCODE_CO2_VALUE => Self::Co2Value(val),
            OPCODE_CO2_SANITY_CHECK => Self::Co2SanityCheck(val),
            _ => Self::Unknown(op, val),
        }
    }
}

impl MonitorReadingParts {
    pub fn set_op_val(&mut self, op: u8, val: u16) {
        let raw_report = MonitorReportRaw::from((op, val));

        match raw_report {
            MonitorReportRaw::Temperature(val) => {
                let temperature_in_kelvin = (val as f32) / 16.0;
                let temperature_in_c = temperature_in_kelvin - 273.15;
                self.temperature = Some(temperature_in_c)
            }
            MonitorReportRaw::Co2Value(val) => {
                self.co2_value = Some(val);
            }
            MonitorReportRaw::Co2SanityCheck(val) => {
                // For very large values, sometimes the "actual" co2 code simply reports 1065, even though
                // the diplay indicates "HI". However, there's a second number that decreases with in-
                // creasing CO2 values. It is not quite 1:1, there is some small-ish factor involved,
                // but for now this offset should be enough.
                const MAGIC_OFFSET_THAT_NEEDS_BETTER_ESTIMATE: u16 = 12811;
                self.co2_sanity_check = Some(MAGIC_OFFSET_THAT_NEEDS_BETTER_ESTIMATE - val)
            }
            MonitorReportRaw::Unknown(_, _) => (),
        }
    }

    pub fn to_reading(&mut self) -> Option<MonitorReading> {
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
            self.clear();
            return Some(mr);
        }
        None
    }

    pub fn new() -> Self {
        Self {
            temperature: None,
            co2_value: None,
            co2_sanity_check: None,
        }
    }
    pub fn clear(&mut self) {
        self.temperature = None;
        self.co2_value = None;
        self.co2_sanity_check = None;
    }
}

impl Default for MonitorReadingParts {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MonitorReading {
    pub temperature: f32,
    pub co2_value: Co2Value,
}

impl MonitorReading {
    /// Creates a new empty monitor reading.
    pub fn new() -> MonitorReading {
        Self {
            temperature: 0.0,
            co2_value: Co2Value::TooHigh(0),
        }
    }
}

impl Default for MonitorReading {
    fn default() -> Self {
        Self::new()
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

// pub fn hex_pretty<const N: usize>(bytes: &[u8]) -> String<N> {
//     let mut out_slice = Vec::<u8, N>::new();
//     hex::encode_to_slice(bytes, &mut out_slice).unwrap();
//     let mut count = 0;
//     let mut s = String::<N>::new();
//     for c in out_slice {
//         let _ = s.push(c as char);
//         count += 1;
//         if count % 2 == 0 {
//             let _ = s.push(' ');
//             count = 0;
//         }
//     }
//     s
// }
