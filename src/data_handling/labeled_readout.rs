pub trait LabeledReadout<const N_BYTES: usize>{
    type Data;

    fn get_label(&self) -> u8;
    fn get_data(&self) -> Self::Data;
    fn get_bytes(&self) -> [u8; N_BYTES];
}

pub mod consts{
    pub const THERMOMETER_SENSOR_TYPE: u8 = 0b0000;
    pub const HIGROMETER_SENSOR_TYPE: u8 = 0b0001;
    pub const BAROMETER_SENSOR_TYPE: u8 = 0b0010;
    pub const LUXMETER_SENSOR_TYPE: u8 = 0b0011;

    pub const SENSOR_ID_OFFSET: u8 = 6;
    pub const UNIT_SCALE_OFFSET: u8 = 4;
    pub const SENSOR_TYPE_OFFSET: u8 = 0;
}