use crate::data_handling::labeled_readout::{LabeledReadout, consts::{BAROMETER_SENSOR_TYPE, HIGROMETER_SENSOR_TYPE, LUXMETER_SENSOR_TYPE, SENSOR_ID_OFFSET, SENSOR_TYPE_OFFSET, THERMOMETER_SENSOR_TYPE, UNIT_SCALE_OFFSET}};

pub struct DynamicLabeledReadout{
    data: u32,
    sensor_id: u8,
    unit_scale: u8,
    label: u8,
    sensor_type: SensorType,
}

impl DynamicLabeledReadout{
    pub fn new_labeled_data(label: u8, data: u32) -> Option<Self>{
        let sensor_type = SensorType::from_bits((label & 0b1111) >> SENSOR_TYPE_OFFSET)?;
        let sensor_id = (label & 0b11) >> SENSOR_ID_OFFSET;
        let unit_scale = (label & 0b11) >> UNIT_SCALE_OFFSET;

        Some(Self{sensor_id, sensor_type, unit_scale, data, label})
    }

    pub fn sensor_id(&self) -> u8{ self.sensor_id }
    pub fn unit_scale(&self) -> u8{ self.unit_scale }
    pub fn sensor_type(&self) -> &SensorType{ &self.sensor_type }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self>{
        if bytes.len() < 5{
            return None;
        }

        Self::new_labeled_data(
            bytes[0], 
            u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]])
        )
    }
}

impl LabeledReadout<5> for DynamicLabeledReadout{
    type Data = u32;

    fn get_bytes(&self) -> [u8; 5] {
        let data = self.data.to_le_bytes();

        [self.label, data[0], data[1], data[2], data[3]]
    }

    fn get_data(&self) -> Self::Data {
        self.data
    }

    fn get_label(&self) -> u8 {
        self.label
    }
}

pub enum SensorType{
    Thermometer,
    Higrometer,
    Barometer,
    Luxmeter,
}

impl SensorType{
    pub fn get_bits(&self) -> u8{
        match self {
            Self::Thermometer => THERMOMETER_SENSOR_TYPE,
            Self::Higrometer => HIGROMETER_SENSOR_TYPE,
            Self::Barometer => BAROMETER_SENSOR_TYPE,
            Self::Luxmeter => LUXMETER_SENSOR_TYPE,
        }
    }

    pub fn from_bits(bits: u8) -> Option<Self>{
        match bits{
            THERMOMETER_SENSOR_TYPE => Some(Self::Thermometer),
            HIGROMETER_SENSOR_TYPE => Some(Self::Higrometer),
            BAROMETER_SENSOR_TYPE => Some(Self::Barometer),
            LUXMETER_SENSOR_TYPE => Some(Self::Luxmeter),
            _ => None,
        }
    }
}