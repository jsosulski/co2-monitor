//! The actual code for converting the HID reports between byte values and useful numbers.
#![no_std]
#![warn(missing_docs)]

pub mod device;
#[cfg(feature = "pc")]
pub mod pc;

/// Contains the individual parts that can be read from the monitor.
///
/// Use this to read from the device, and write whatever value is coming in, to this struct.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MonitorReadingParts {
    /// Temperature in degrees celsius if set.
    pub temperature: Option<f32>,
    /// Co2 PPM if set.
    pub co2_value: Option<u16>,
    /// Co2 sanity check value if set.
    pub co2_sanity_check: Option<u16>,
}

/// Contains the read out values as u16, if the opcode was unknown, it was returned as well.
#[allow(missing_docs)]
pub enum MonitorReportRaw {
    Temperature(u16),
    Co2Value(u16),
    Co2SanityCheck(u16),
    Unknown(u8, u16),
}

/// Reported opcode when the HID report is a temperature.
pub const OPCODE_TEMPERATURE: u8 = 0x42;
/// Reported opcode when the HID report is a co2 value.
pub const OPCODE_CO2_VALUE: u8 = 0x50;
/// Reported opcode when the HID report is a co2 sanity check value. (At least as far as i know)
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
    /// Given the opcode as `u8` and the value as `u16`, sets the corresponding field.
    pub fn set_op_val(&mut self, op: u8, val: u16) {
        let raw_report = MonitorReportRaw::from((op, val));

        match raw_report {
            MonitorReportRaw::Temperature(val) => {
                let temperature_in_kelvin = f32::from(val) / 16.0;
                let temperature_in_c = temperature_in_kelvin - 273.15;
                self.temperature = Some(temperature_in_c);
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
                self.co2_sanity_check = Some(MAGIC_OFFSET_THAT_NEEDS_BETTER_ESTIMATE - val);
            }
            MonitorReportRaw::Unknown(_, _) => (),
        }
    }

    /// If all values are available, returns a complete `MonitorReading`. Otherwise returns `None`.
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

    /// Create a new container with no values set.
    pub fn new() -> Self {
        Self {
            temperature: None,
            co2_value: None,
            co2_sanity_check: None,
        }
    }

    /// Reset all values.
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

/// A complete reading from the co2 monitor device.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MonitorReading {
    /// Temperature in degrees Celsius.
    pub temperature: f32,
    /// A valid/invalid co2 reading in ppm.
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

/// A Co2Value that knows whether it is/was out of spec.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Co2Value {
    /// A valid Co2 reading
    Valid(u16),
    /// The sanity check variable indicated that this reading was too high. You might still want to see the actually
    /// read value tho. You do you.
    TooHigh(u16),
}

impl Co2Value {
    /// Get the CO2 as a PPM u16 and a bool flag that indicates whether the readout was valid.
    pub fn as_num_and_bool(&self) -> (u16, bool) {
        match self {
            Co2Value::Valid(n) => (*n, true),
            Co2Value::TooHigh(n) => (*n, false),
        }
    }
}

impl core::fmt::Display for Co2Value {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Co2Value::Valid(n) => write!(f, "{}", n),
            Co2Value::TooHigh(_) => f.write_str("too high"),
        }
    }
}
