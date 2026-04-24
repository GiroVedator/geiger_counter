mod connection;
use std::io::{Read, Write};
use serialport::{SerialPort;
//use std::time::Duration;


pub struct Device {
	name: String,
	port: Option<Box<dyn SerialPort>>,
}

impl Device {
	pub fn new(name: &str) -> Result<Self, Box<dyn std::error::Error>> 
    {
		Ok(Self {
			name: name.to_string(),
			port: None,
		})
	}

    pub fn initialize(&mut self, port: Box<dyn SerialPort>) -> Result<(), Box<dyn std::error::Error>> 
    {
		self.port = Some(port);
        Ok(())
    }

	fn send_command(&mut self, command: &[u8], buffer_size: usize) -> Result<String, Box<dyn std::error::Error>> {
		self.port.write(command);

		let mut buf = Vec::with_capacity(buffer_size);
		buf.resize(buffer_size, 0);
		let bytes_read = self.port.read(&mut buf)?;
		Ok(String::from_utf8_lossy(&buf[..bytes_read]).to_string())
	}

	pub fn get_version(&mut self) -> Result<String, Box<dyn std::error::Error>> {
		self.send_command(VERSION_COMMAND, 8)
	}

	pub fn get_cpm(&mut self, buffer_size: usize) -> Result<String, Box<dyn std::error::Error>> {
		self.send_command(CPM_COMMAND, buffer_size)
	}

	pub fn get_gyro(&mut self, buffer_size: usize) -> Result<String, Box<dyn std::error::Error>> {
		self.send_command(GYRO_COMMAND, buffer_size)
	}

	pub fn get_volt(&mut self, buffer_size: usize) -> Result<String, Box<dyn std::error::Error>> {
		self.send_command(VOLT_COMMAND, buffer_size)
	}

	pub fn get_datetime(&mut self, buffer_size: usize) -> Result<String, Box<dyn std::error::Error>> {
		self.send_command(DATE_COMMAND, buffer_size)
	}

	pub fn get_config(&mut self, buffer_size: usize) -> Result<String, Box<dyn std::error::Error>> {
		self.send_command(CONFIG_COMMAND, buffer_size)
	}

	pub fn get_temp(&mut self, buffer_size: usize) -> Result<String, Box<dyn std::error::Error>> {
		self.send_command(TEMP_COMMAND, buffer_size)
	}

	pub fn power_off(&mut self, buffer_size: usize) -> Result<String, Box<dyn std::error::Error>> {
		self.send_command(POWOFF_COMMAND, buffer_size)
	}

	pub fn power_on(&mut self, buffer_size: usize) -> Result<String, Box<dyn std::error::Error>> {
		self.send_command(POWON_COMMAND, buffer_size)
	}

	pub fn set_datetime(&mut self, buffer_size: usize) -> Result<String, Box<dyn std::error::Error>> {
		self.send_command(SETTIME_COMMAND, buffer_size)
	}

	pub fn reboot(&mut self, buffer_size: usize) -> Result<String, Box<dyn std::error::Error>> {
		self.send_command(REBOOT_COMMAND, buffer_size)
	}
}
