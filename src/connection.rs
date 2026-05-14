use serialport::SerialPort;
use std::io::{Read, Write};
use std::time::Duration;

enum COMMAND {
    VERSION,
    CPM,
    USV,
    GYRO,
    VOLT,
    CONFIG,
    TEMP,
    SERIAL,
    DATE,
    POWOFF,
    POWON,
    SETTIME,
    REBOOT,
}

impl COMMAND {
    fn as_bytes(&self) -> &'static [u8] {
        match self {
            COMMAND::VERSION => b"<GETVER>>",
            COMMAND::CPM => b"<GETCPM>>",
            COMMAND::USV => b"<GETUSV>>",
            COMMAND::GYRO => b"<GETGYRO>>",
            COMMAND::VOLT => b"<GETVOLT>>",
            COMMAND::CONFIG => b"<GETCFG>>",
            COMMAND::TEMP => b"<GETTEMP>>",
            COMMAND::SERIAL => b"<GETSERIAL>>",
            COMMAND::DATE => b"<GETDATETIME>>",
            COMMAND::POWOFF => b"<POWEROFF>>",
            COMMAND::POWON => b"<POWERON>>",
            COMMAND::SETTIME => b"<SETDATETIME>>",
            COMMAND::REBOOT => b"<REBOOT>>",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FieldType {
    LittleEndianFloat,
    Byte,
}

#[derive(Debug, Clone, Copy)]
struct ConfigField {
    index: usize,
    size: usize,
    description: &'static str,
    field_type: Option<FieldType>,
}

#[derive(Debug, Clone, Copy)]
enum Config {
    CalibrationUSv0,
    CalibrationUSv1,
    CalibrationUSv2,
    IdleTextState,
    AlarmValueUSv,
    Baudrate,
    ThresholdUSv,
    CalibrationCPM_0,
    CalibrationCPM_1,
    CalibrationCPM_2,
}

impl Config {
    const ALL: [Config; 10] = [
        Config::CalibrationCPM_0,
        Config::CalibrationUSv0,
        Config::CalibrationCPM_1,
        Config::CalibrationUSv1,
        Config::CalibrationCPM_2,
        Config::CalibrationUSv2,
        Config::IdleTextState,
        Config::AlarmValueUSv,
        Config::Baudrate,
        Config::ThresholdUSv,
    ];

    fn all() -> &'static [Config] {
        &Self::ALL
    }

    fn field(&self) -> ConfigField {
        match self {
            Config::CalibrationCPM_0 => ConfigField {
                index: 8,
                size: 2,
                description: "",
                field_type: Some(FieldType::LittleEndianFloat),
            },
            Config::CalibrationUSv0 => ConfigField {
                index: 10,
                size: 4,
                description: "",
                field_type: Some(FieldType::LittleEndianFloat),
            },
            Config::CalibrationCPM_1 => ConfigField {
                index: 14,
                size: 2,
                description: "",
                field_type: Some(FieldType::LittleEndianFloat),
            },
            Config::CalibrationUSv1 => ConfigField {
                index: 16,
                size: 4,
                description: "",
                field_type: Some(FieldType::LittleEndianFloat),
            },
            Config::CalibrationCPM_2 => ConfigField {
                index: 20,
                size: 2,
                description: "",
                field_type: Some(FieldType::LittleEndianFloat),
            },
            Config::CalibrationUSv2 => ConfigField {
                index: 22,
                size: 4,
                description: "",
                field_type: Some(FieldType::LittleEndianFloat),
            },
            Config::IdleTextState => ConfigField {
                index: 26,
                size: 1,
                description: "??",
                field_type: None,
            },
            Config::AlarmValueUSv => ConfigField {
                index: 27,
                size: 4,
                description: "",
                field_type: Some(FieldType::LittleEndianFloat),
            },
            Config::Baudrate => ConfigField {
                // see https://www.gqelectronicsllc.com/forum/topic.asp?TOPIC_ID=4948 reply#12
                index: 57,
                size: 1,
                description: "64=1200,160=2400,208=4800,232=9600,240=14400,\
                               244=19200,248=28800,250=38400,252=57600,254=115200",
                field_type: None,
            },
            Config::ThresholdUSv => ConfigField {
                index: 65,
                size: 4,
                description: "",
                field_type: Some(FieldType::LittleEndianFloat),
            },
        }
    }
}

/// Handles serial port connection for Geiger reader
#[allow(non_snake_case, dead_code)]
pub struct SerialConnection {
    port: Box<dyn SerialPort>,
    config: Vec<String>,
}

impl SerialConnection {
    /// Opens a serial connection to the specified port
    pub fn new(port_name: &str, baud_rate: u32) -> Result<SerialConnection, Box<dyn std::error::Error>> {
        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_secs(1))
            .open()?;

        let mut connection = SerialConnection { port, config: Vec::new() };
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
                _ => String::from_utf8_lossy(value).trim().to_string(),
            };

            self.config.push(format!("{:?}: {}", config, formatted_value));
        }
        Ok(())
    }

    pub fn print_config(&mut self) {
        for (i, line) in self.config.iter().enumerate() {
            println!("{}: {}", i, line);
        }
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

    pub fn get_usV(&mut self) -> Result<f32, Box<dyn std::error::Error>> {
        let buffer = self.run_command::<4>(COMMAND::USV.as_bytes())?;
        let usv = u32::from_be_bytes(buffer) as f32 / 100.0;

        // conversion
        Ok(usv)
    }  

    pub fn get_gyro(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let buffer = self.run_command::<7>(COMMAND::GYRO.as_bytes())?;
        let x = u16::from_be_bytes([buffer[0], buffer[1]]);
        let y = u16::from_be_bytes([buffer[2], buffer[3]]);
        let z = u16::from_be_bytes([buffer[4], buffer[5]]);
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