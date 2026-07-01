use serialport::SerialPort;
use std::io::{Read, Write};
use std::time::Duration;
use crate::fit::Fit;
use crate::utilities::{COMMAND, Config, FieldType};
use std::collections::HashMap;

/// Handles serial port connection for Geiger reader
#[allow(non_snake_case, dead_code)]
pub struct SerialConnection {
    port: Box<dyn SerialPort>,
    config: HashMap<Config, String>,
    cpm_to_nSv_fit: Option<Fit>,
}

impl SerialConnection {
    /// Opens a serial connection to the specified port
    pub fn new(port_name: &str, baud_rate: u32) -> Result<SerialConnection, Box<dyn std::error::Error>> {
        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_secs(1))
            .open()?;

        let connection = SerialConnection { port, config: HashMap::new(), cpm_to_nSv_fit: None };
        Ok(connection)
    }

    pub fn drain(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut drain = [0u8; 1024];
        while self.port.read(&mut drain).unwrap_or(0) > 0 {}
        Ok(())
    }

    pub fn extract_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let buffer = self.run_command::<256>(COMMAND::CONFIG.as_bytes())?;
        self.config.clear();

        for config in Config::all() {
            let field = config.field();
            let start = field.index;
            let end = start + field.size;

            if end > buffer.len() {
                return Err(format!("Config field {:?} is out of bounds", config).into());
            }

            let value = &buffer[start..end];
            let formatted_value = match field.field_type {
                Some(FieldType::LittleEndianFloat) if field.size == 4 => {
                    let bytes = [value[0], value[1], value[2], value[3]];
                    format!("{}", f32::from_le_bytes(bytes))
                }
                Some(FieldType::HighEndianUnsignedInt) if field.size == 2 => {
                    let bytes = [value[0], value[1]];
                    format!("{}", u16::from_be_bytes(bytes))
                }
                _ =>format!("{}", u8::from_le_bytes([value[0]]))
            };

            self.config.insert(*config, formatted_value);
        }
        Ok(())
    }

    pub fn print_config(&mut self) {
        for config in Config::all() {
            if let Some(value) = self.config.get(config) {
                println!("{:?}: {}", config, value);
            }
        }
    }

    pub fn usv_calibration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let x = vec![0.0, self.config.get(&Config::CalibrationCpm0).unwrap().parse::<f64>().unwrap(), self.config.get(&Config::CalibrationCpm1).unwrap().parse::<f64>().unwrap(), self.config.get(&Config::CalibrationCpm2).unwrap().parse::<f64>().unwrap()];
        let y = vec![0.0, self.config.get(&Config::CalibrationUSv0).unwrap().parse::<f64>().unwrap(), self.config.get(&Config::CalibrationUSv1).unwrap().parse::<f64>().unwrap(), self.config.get(&Config::CalibrationUSv2).unwrap().parse::<f64>().unwrap()];
        
        self.cpm_to_nSv_fit = Some(Fit::new(&x, &y, 2)?);
        Ok(())
    }
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
        let buffer = self.run_command::<14>(COMMAND::VERSION.as_bytes())?;
        let response = String::from_utf8_lossy(&buffer).to_string();
        Ok(response)
    }

    pub fn get_cpm(&mut self) -> Result<u16, Box<dyn std::error::Error>> {
        let buffer = self.run_command::<2>(COMMAND::CPM.as_bytes())?;
        let cpm = u16::from_be_bytes(buffer);
        Ok(cpm)
    }

    pub fn get_nSv(&mut self, cpm: Option<u16>) -> Result<f32, Box<dyn std::error::Error>> {
        let mut current_cpm: u16 = 0;

        if cpm.is_none() {
            current_cpm = cpm.unwrap_or(self.get_cpm()?);
        }
        else
        {
            current_cpm = cpm.unwrap();
        }
        if let Some(fit) = &self.cpm_to_nSv_fit {
            return Ok(fit.predict(current_cpm as f64) as f32);
        }    
        Err("CPM to nSv fit not calibrated".into())}  

    pub fn get_gyro(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let buffer = self.run_command::<7>(COMMAND::GYRO.as_bytes())?;
        let x = u16::from_le_bytes([buffer[0], buffer[1]]);
        let y = u16::from_le_bytes([buffer[2], buffer[3]]);
        let z = u16::from_le_bytes([buffer[4], buffer[5]]);
        let response = format!("X: {}, Y: {}, Z: {}", x, y, z);
        Ok(response)
    }

    pub fn get_voltage(&mut self) -> Result<f32, Box<dyn std::error::Error>> {
        let buffer = self.run_command::<1>(COMMAND::VOLT.as_bytes())?;
        let voltage = u8::from_be_bytes(buffer) as f32 / 10.0;
        Ok(voltage)
    }

    pub fn get_temperature(&mut self) -> Result<f32, Box<dyn std::error::Error>> {
        let buffer = self.run_command::<4>(COMMAND::TEMP.as_bytes())?;
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