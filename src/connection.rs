use serialport::SerialPort;
use std::io::{Read, Write};
use std::time::Duration;

#[allow(dead_code)]
const VERSION: &[u8] = b"<GETVER>>";
//const SERIAL: &[u8] = b"<GETSERIAL>>";
const CPM: &[u8] = b"<GETCPM>>";
const GYRO: &[u8] = b"<GETGYRO>>";
const VOLT: &[u8] = b"<GETVOLT>>";
//const DATE: &[u8] = b"<GETDATETIME>>";
const CONFIG: &[u8] = b"<GETCFG>>";
const TEMP: &[u8] = b"<GETTEMP>>";
//const POWOFF: &[u8] = b"<POWEROFF>>";
//const POWON: &[u8] = b"<POWERON>>";
//const SETTIME: &[u8] = b"<SETDATETIME>>";
//const REBOOT: &[u8] = b"<REBOOT>>";


/// Handles serial port connection for Geiger reader
pub struct SerialConnection {
    port: Box<dyn SerialPort>
}

impl SerialConnection {
    /// Opens a serial connection to the specified port
    pub fn new(port_name: &str, baud_rate: u32) -> Result<SerialConnection, Box<dyn std::error::Error>> {
        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_secs(1))
            .open()?;

        Ok(SerialConnection { port })
    }

    // pub fn drain(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    //     let mut drain = [0u8; 1024];
    //     while self.port.read(&mut drain).unwrap_or(0) > 0 {}
    //     Ok(())
    // }

    fn run_command<const N: usize>(&mut self, command: &[u8]) -> Result<[u8; N], Box<dyn std::error::Error>> {
        self.port.write_all(command)?;
        self.port.flush()?;

        std::thread::sleep(Duration::from_millis(100));
        let mut buf = [0u8; N];
        let _bytes_read = self.port.read(&mut buf)?;
        if _bytes_read != N {
            return Err(format!("Expected {} bytes, got {}", N, _bytes_read).into());
        }
        Ok(buf)
    }

    pub fn get_version(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let buffer = self.run_command::<14>(VERSION)?;
        let response = String::from_utf8_lossy(&buffer).to_string();
        Ok(response)
    }

    pub fn get_cpm(&mut self) -> Result<u16, Box<dyn std::error::Error>> {
        let buffer = self.run_command::<2>(CPM)?;
        let cpm = u16::from_be_bytes(buffer);
        Ok(cpm)
    }

    pub fn get_gyro(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let buffer = self.run_command::<7>(GYRO)?;
        let x = u16::from_be_bytes([buffer[0], buffer[1]]);
        let y = u16::from_be_bytes([buffer[2], buffer[3]]);
        let z = u16::from_be_bytes([buffer[4], buffer[5]]);
        let response = format!("X: {}, Y: {}, Z: {}", x, y, z);
        Ok(response)
    }

    pub fn get_voltage(&mut self) -> Result<f32, Box<dyn std::error::Error>> {
        let buffer = self.run_command::<1>(VOLT)?;
        let voltage = u8::from_be_bytes(buffer) as f32 / 10.0;
        Ok(voltage)
    }

    //pub fn get_config(&mut self) -> Result<String, Box<dyn std::error::Error>> {
    //    self.run_command(CONFIG)
    // }

    pub fn get_temperature(&mut self) -> Result<f32, Box<dyn std::error::Error>> {
        let buffer = self.run_command::<4>(TEMP)?;
        let mut sign: f32 = 1.0;
        let sign_byte = u8::from_be_bytes([buffer[2]]);
        if sign_byte != 0 {
            sign = -1.0;
        }
        let int_temp = u8::from_be_bytes([buffer[0]]);
        let frac_temp = u8::from_be_bytes([buffer[1]]);
        let temp: f32 = sign * (int_temp as f32 + frac_temp as f32 / 10.0);
        Ok(temp)
    }

    /// Closes the connection (automatically called on drop)
    pub fn close(&mut self) -> Result<(), Box<dyn std::error::Error>> 
    {
        Ok(())
    }
}