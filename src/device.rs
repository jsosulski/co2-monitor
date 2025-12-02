use crate::MonitorReadingParts;

pub const VID: u16 = 0x04d9;
pub const PID: u16 = 0xa052;

pub trait MonitorComms {
    fn connect();
    fn read_to_part(read_buffer: &mut [u8; 8], part: &mut MonitorReadingParts);
}
