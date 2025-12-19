//! Contains device specific handling code and the trait definition for the `Co2Monitor`.
use crate::{MonitorReading, MonitorReadingParts};

const VID: u16 = 0x04d9;
const PID: u16 = 0xa052;

/// Most errors should be safely assumed to be skippable.
pub enum MonitorError {
    /// Something during the read failed.
    ReadFailed,
    /// The read report doesn't contain the terminator byte in 5-th position: 0x0d.
    MissingTerminatorByte,
    /// Bytes 1, 2 and 3, don't sum to byte 4 (in the lowest byte).
    ChecksumInvalid,
    /// A timeout interrupted the USB-HID read.
    Timeout,
}

/// Implement this trait for your struct that handles talking over USB-HID. See `pc.rs` for an example implementation
/// that uses the hidapi rust crate.
pub trait Co2MonitorCommunication {
    /// This method should create your managing struct and set up the necessary connection.
    fn init_and_connect() -> Self;

    /// This rarely needs to be called directly, use `read_to_part` instead.
    /// It should read a single 8-byte HID report to the `read_buffer`.
    fn read(&self, read_buffer: &mut [u8; 8]) -> Result<usize, MonitorError>;

    /// Online resources have some key or magic table in here, but for my co2 device it works with just zeroes...
    /// Sending the feature report is still necessary. Otherwise no HID data will be available.
    fn get_feature_report() -> &'static [u8; 9] {
        &[0u8; 9]
    }

    /// The vendor ID of the used ZYG-01
    fn get_vid() -> u16 {
        VID
    }

    /// The product ID of the used ZYG-01
    fn get_pid() -> u16 {
        PID
    }

    /// If `read` is implemented, this function reads a single HID report and, if a correct op-code was read, fills
    /// the passed partial reading. If all parts have been read, it returns Some(...) with a complete reading.
    fn read_to_part(
        &self,
        part: &mut MonitorReadingParts,
    ) -> Result<Option<MonitorReading>, MonitorError> {
        let mut read_buffer = [0u8; 8];
        let read_len = self.read(&mut read_buffer);
        match read_len {
            Ok(8) => {
                if read_buffer[4] != 0x0d {
                    return Err(MonitorError::MissingTerminatorByte);
                }
                if ((read_buffer[0] as u16 + read_buffer[1] as u16 + read_buffer[2] as u16) & 0xff)
                    as u8
                    != read_buffer[3]
                {
                    return Err(MonitorError::ChecksumInvalid);
                }

                let op = read_buffer[0];
                let val = ((read_buffer[1] as u16) << 8) | read_buffer[2] as u16;
                // This will fill once the report values container is saturated.
                // let _ = self.report_values.insert(op, val);
                part.set_op_val(op, val);
            }

            // Too few bytes read. Even though we only need the first 5, it should've been 8.
            Ok(_) => (),
            Err(_e) => {
                // eprintln!("read error: {}", e);
            }
        }
        Ok(part.to_reading())
    }
}
