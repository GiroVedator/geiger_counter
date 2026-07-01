
#[allow(unused)]
pub enum COMMAND {
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
    pub fn as_bytes(&self) -> &'static [u8] {
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
pub enum FieldType {
    LittleEndianFloat,
    HighEndianUnsignedInt
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigField {
    pub index: usize,
    pub size: usize,
    pub description: &'static str,
    pub field_type: Option<FieldType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Config {
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

    pub fn all() -> &'static [Config] {
        &Self::ALL
    }

    pub fn field(&self) -> ConfigField {
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