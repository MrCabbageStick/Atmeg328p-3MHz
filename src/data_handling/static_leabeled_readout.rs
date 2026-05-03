use core::marker::PhantomData;

use crate::data_handling::labeled_readout::LabeledReadout;

pub struct TypedLabelReadout<ID, SCALE, TYPE>{
    data: u32,
    _label: PhantomData<(ID, SCALE, TYPE)>,
}

impl<ID: SensorId, SCALE: UnitScale, TYPE: SensorType> TypedLabelReadout<ID, SCALE, TYPE> {
    const LABEL: u8 = ID::BITS << 6
        | (SCALE::BITS & 0b11) << 4
        | (TYPE::BITS & 0b1111);

    pub fn new(data: u32) -> Self { Self {data, _label: PhantomData} }
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

impl SensorType for Thermometer{ const BITS: u8 = 0b0000; }
impl SensorType for Higrometer{ const BITS: u8 = 0b0001; }
impl SensorType for Barometer{ const BITS: u8 = 0b0010; }
impl SensorType for Luxmeter{ const BITS: u8 = 0b0011; }