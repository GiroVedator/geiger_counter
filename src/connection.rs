use serialport::SerialPort;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::time::Duration;
use polyfit::MonomialFit;

struct Fit{
    coeffs: Vec<f64>,
}

impl Fit {
    fn new(x: &[f64], y: &[f64], degree: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let data: Vec<(f64, f64)> = x.iter().zip(y.iter()).map(|(x, y)| (*x, *y)).collect();
        let fit = MonomialFit::new(&data, degree)?;
        Ok(Fit { coeffs: fit.coefficients().to_vec() })
    }

    pub fn predict(&self, x: f64) -> f64 {
        let mut sum = 0.0;
        sum = 1000.0*(self.coeffs[1]*x + self.coeffs[0]);
        sum
    }
}

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
    HighEndianUnsignedInt
}

#[derive(Debug, Clone, Copy)]
struct ConfigField {
    index: usize,
    size: usize,
    description: &'static str,
    field_type: Option<FieldType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Config {
    CalibrationUSv0,
    CalibrationUSv1,
    CalibrationUSv2,
    IdleTextState,
    AlarmValueUSv,
    Baudrate,
    ThresholdUSv,
    CalibrationCpm0,
    CalibrationCpm1,
    CalibrationCpm2,
    ThresholdCpm,
}

impl Config {
    const ALL: [Config; 11] = [
        Config::CalibrationCpm0,
        Config::CalibrationUSv0,
        Config::CalibrationCpm1,
        Config::CalibrationUSv1,
        Config::CalibrationCpm2,
        Config::CalibrationUSv2,
        Config::IdleTextState,
        Config::AlarmValueUSv,
        Config::Baudrate,
        Config::ThresholdUSv,
        Config::ThresholdCpm,
    ];

    fn all() -> &'static [Config] {
        &Self::ALL
    }

    fn field(&self) -> ConfigField {
        match self {
            Config::CalibrationCpm0 => ConfigField {
                index: 8,
                size: 2,
                description: "",
                field_type: Some(FieldType::HighEndianUnsignedInt),
            },
            Config::CalibrationUSv0 => ConfigField {
                index: 10,
                size: 4,
                description: "",
                field_type: Some(FieldType::LittleEndianFloat),
            },
            Config::CalibrationCpm1 => ConfigField {
                index: 14,
                size: 2,
                description: "",
                field_type: Some(FieldType::HighEndianUnsignedInt),
            },
            Config::CalibrationUSv1 => ConfigField {
                index: 16,
                size: 4,
                description: "",
                field_type: Some(FieldType::LittleEndianFloat),
            },
            Config::CalibrationCpm2 => ConfigField {
                index: 20,
                size: 2,
                description: "",
                field_type: Some(FieldType::HighEndianUnsignedInt),
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
            Config::ThresholdCpm => ConfigField {
                index: 62,
                size: 2,
                description: "",
                field_type: Some(FieldType::HighEndianUnsignedInt),
            },
        }
    }
}

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

    pub fn get_nSv(&mut self) -> Result<f32, Box<dyn std::error::Error>> {
        let cpm = self.get_cpm()?;
        if let Some(fit) = &self.cpm_to_nSv_fit {
            return Ok(fit.predict(cpm as f64) as f32);
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