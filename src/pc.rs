//! Implement the Co2 monitor reading for a PC using the `hidapi` crate.
use crate::device::{Co2MonitorCommunication, MonitorError};
use hidapi::{HidApi, HidDevice};

/// This struct holds the `HidDevice` from hidapi crate, that is needed for communication.
pub struct PcCo2Monitor {
    device: HidDevice,
}

impl Co2MonitorCommunication for PcCo2Monitor {
    fn init_and_connect() -> Self {
        let api = HidApi::new().expect("Could not initialize Hid Api.");
        let device = api
            .open(Self::get_vid(), Self::get_pid())
            .expect("Unable to open HID device. Is it connected to this computer? Do you have sufficient permissions?");

        // This tells the monitor to actually start sending data over HID.
        device
            .send_feature_report(Self::get_feature_report())
            .expect("Could not send feature report.");

        Self { device }
    }

    fn read(&self, read_buffer: &mut [u8; 8]) -> Result<usize, MonitorError> {
        self.device
            .read_timeout(read_buffer, 1000)
            .map_err(|_| MonitorError::ReadFailed)
    }
}
