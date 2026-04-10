use serialport::SerialPort;
use std::io::{Read, Write};
use std::time::Duration;
#[allow(dead_code)]
/// Handles serial port connection for Geiger reader
pub struct Connection {
    port: Box<dyn SerialPort>,
}

impl Connection {
    /// Opens a serial connection to the specified port
    pub fn new(port_name: &str, baud_rate: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_secs(1))
            .open()?;

        Ok(Connection { port })
    }

    /// Reads data from the serial port
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(self.port.read(buf)?)
    }

    /// Writes data to the serial port
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(self.port.write(buf)?)
    }

    /// Closes the connection (automatically called on drop)
    pub fn close(&mut self) -> Result<(), Box<dyn std::error::Error>> 
    {
        Ok(())
    }
}