use core::marker::PhantomData;

use crate::data_handling::labeled_readout::{LabeledReadout, consts::{BAROMETER_SENSOR_TYPE, HIGROMETER_SENSOR_TYPE, LUXMETER_SENSOR_TYPE, SENSOR_ID_MASK, SENSOR_ID_OFFSET, SENSOR_TYPE_MASK, SENSOR_TYPE_OFFSET, THERMOMETER_SENSOR_TYPE, UNIT_SCALE_MASK, UNIT_SCALE_OFFSET}};

pub struct TypedLabelReadout<ID, SCALE, TYPE>{
    data: u32,
    _label: PhantomData<(ID, SCALE, TYPE)>,
}

impl<ID: SensorId, SCALE: UnitScale, TYPE: SensorType> TypedLabelReadout<ID, SCALE, TYPE> {
    const LABEL: u8 = 
        (ID::BITS & SENSOR_ID_MASK) << SENSOR_ID_OFFSET
        | (SCALE::BITS & UNIT_SCALE_MASK) << UNIT_SCALE_OFFSET
        | (TYPE::BITS & SENSOR_TYPE_MASK) << SENSOR_TYPE_OFFSET;

    pub fn new(data: u32) -> Self { Self {data, _label: PhantomData} }

    pub fn from_bytes(bytes: [u8; 5]) -> Result<Self, ()>{
        let label = bytes[0];

        // Check if label is correct
        if !(
            (label >> SENSOR_ID_OFFSET) & SENSOR_ID_MASK == ID::BITS
            && (label >> UNIT_SCALE_OFFSET) & UNIT_SCALE_MASK == SCALE::BITS
            && (label >> SENSOR_TYPE_OFFSET) & SENSOR_TYPE_MASK == TYPE::BITS
        ){
            return Err(())
        }

        Ok(Self::new(u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]])))
    }
}

impl<ID: SensorId, SCALE: UnitScale, TYPE: SensorType> LabeledReadout<5> for TypedLabelReadout<ID, SCALE, TYPE>{
    type Data = u32;

    fn get_label(&self) -> u8 {
        Self::LABEL
    }

    fn get_data(&self) -> Self::Data {
        self.data
    }
    
    fn get_bytes(&self) -> [u8; 5] {
        let data = self.data.to_le_bytes();

        [Self::LABEL, data[0], data[1], data[2], data[3]]
    }
}



pub trait SensorId{ const BITS: u8; }

pub struct SensorId0;
pub struct SensorId1;
pub struct SensorId2;
pub struct SensorId3;

impl SensorId for SensorId0{ const BITS: u8 = 0b00; }
impl SensorId for SensorId1{ const BITS: u8 = 0b01; }
impl SensorId for SensorId2{ const BITS: u8 = 0b10; }
impl SensorId for SensorId3{ const BITS: u8 = 0b11; }


pub trait UnitScale{ const BITS: u8; }

pub struct UnitScale1;
/// Unit scale of 0.1
pub struct UnitScale1_10;
/// Unit scale of 0.01
pub struct UnitScale1_100;
/// Unit scale of 0.001
pub struct UnitScale1_1000;

impl UnitScale for UnitScale1{const BITS: u8 = 0b00;}
impl UnitScale for UnitScale1_10{const BITS: u8 = 0b01;}
impl UnitScale for UnitScale1_100{const BITS: u8 = 0b10;}
impl UnitScale for UnitScale1_1000{const BITS: u8 = 0b11;}


pub trait SensorType{ const BITS: u8; }

pub struct Thermometer;
pub struct Higrometer;
pub struct Barometer;
pub struct Luxmeter;

impl SensorType for Thermometer{ const BITS: u8 = THERMOMETER_SENSOR_TYPE; }
impl SensorType for Higrometer{ const BITS: u8 = HIGROMETER_SENSOR_TYPE; }
impl SensorType for Barometer{ const BITS: u8 = BAROMETER_SENSOR_TYPE; }
impl SensorType for Luxmeter{ const BITS: u8 = LUXMETER_SENSOR_TYPE; }


#[cfg(test)]
mod tests{
    use crate::data_handling::{labeled_readout::LabeledReadout, static_labeled_readout::{SensorId0, Thermometer, TypedLabelReadout, UnitScale1}};

    #[test]
    fn from_bytes(){
        let data_in = TypedLabelReadout::new::<SensorId0, UnitScale1, Thermometer>(0xffeeddcc);
        let bytes = data_in.get_bytes();

        let data_out = TypedLabelReadout::<SensorId0, UnitScale1, Thermometer>::from_bytes(bytes);

        assert!(matches!(data_out, Ok(_)), "Correctly byte encoded readout cannot be parsed");
        
        let unwrapped = data_out.unwrap();
        assert!(unwrapped.data == data_in.data, "Data in decoded readout is different from encoded one: ({:x} vs {:x})", unwrapped.data, data_in.data);
    }
}