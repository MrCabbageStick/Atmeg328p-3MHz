
// Register 0xf4 (ctrl_meas)
const TEMP_OVERSAMPLING_OFFSET: u8 = 5;
const PRESS_OVERSAMPLING_OFFSET: u8 = 2;
const POWER_MODE_OFFSET: u8 = 0;

// Register 0xf5 (config)
const STANDBY_OFFSET: u8 = 5;
const IIR_CONSTANT_OFFSET: u8 = 2;
const EN_SPI_OFFSET: u8 = 0;

pub trait Config{
    type TempOversampling: Oversampling;
    type PressOversampling: Oversampling;
    type PowerMode: PowerMode;
    type Standby: Standby;
    type IIR: IIRConstant;

    const CTRL_MEAS_VALUE: u8 = 
        Self::TempOversampling::BITS << TEMP_OVERSAMPLING_OFFSET
        | Self::PressOversampling::BITS << PRESS_OVERSAMPLING_OFFSET
        | Self::PowerMode::BITS << POWER_MODE_OFFSET;

    const CONFIG_VALUE: u8 =
        Self::Standby::BITS << STANDBY_OFFSET
        | Self::IIR::BITS << IIR_CONSTANT_OFFSET
        // Disable SPI mode
        | 0b0 << EN_SPI_OFFSET;
}

pub struct DefaultConfig;

impl Config for DefaultConfig{
    type TempOversampling = OversamplingX1;
    type PressOversampling = OversamplingX2;
    type PowerMode = PowerModeForced;

    type Standby = Standby0_5ms;
    type IIR = IIROff;
}


/// Temperature and pressure oversampling
pub trait Oversampling{ const BITS: u8; }

pub struct OversamplingSkipped;
pub struct OversamplingX1;
pub struct OversamplingX2;
pub struct OversamplingX4;
pub struct OversamplingX8;
pub struct OversamplingX16;

impl Oversampling for OversamplingSkipped{ const BITS: u8 = 0b000; }
impl Oversampling for OversamplingX1{  const BITS: u8 = 0b001; }
impl Oversampling for OversamplingX2{  const BITS: u8 = 0b010; }
impl Oversampling for OversamplingX4{  const BITS: u8 = 0b011; }
impl Oversampling for OversamplingX8{  const BITS: u8 = 0b100; }
impl Oversampling for OversamplingX16{ const BITS: u8 = 0b101; }


/// Power mode
pub trait PowerMode{ const BITS: u8; }

pub struct PowerModeSleep;
pub struct PowerModeForced;
pub struct PowerModeNormal;

impl PowerMode for PowerModeSleep{ const BITS: u8 = 0b00; }
impl PowerMode for PowerModeForced{ const BITS: u8 = 0b10; }
impl PowerMode for PowerModeNormal{ const BITS: u8 = 0b11; }


/// Normal mode standby
pub trait Standby{ const BITS: u8; }

pub struct Standby0_5ms;
pub struct Standby62_5ms;
pub struct Standby125ms;
pub struct Standby250ms;
pub struct Standby500ms;
pub struct Standby1000ms;
pub struct Standby2000ms;
pub struct Standby4000ms;

impl Standby for Standby0_5ms{   const BITS: u8 = 0b000; }
impl Standby for Standby62_5ms{  const BITS: u8 = 0b001; }
impl Standby for Standby125ms{   const BITS: u8 = 0b010; }
impl Standby for Standby250ms{   const BITS: u8 = 0b011; }
impl Standby for Standby500ms{   const BITS: u8 = 0b100; }
impl Standby for Standby1000ms{  const BITS: u8 = 0b101; }
impl Standby for Standby2000ms{  const BITS: u8 = 0b110; }
impl Standby for Standby4000ms{  const BITS: u8 = 0b111; }


/// IIR filter time constant
pub trait IIRConstant{ const BITS: u8; }

pub struct IIROff;
pub struct IIR2;
pub struct IIR4;
pub struct IIR8;
pub struct IIR16;

impl IIRConstant for IIROff{ const BITS: u8 = 0b000; }
impl IIRConstant for IIR2{   const BITS: u8 = 0b001; }
impl IIRConstant for IIR4{   const BITS: u8 = 0b010; }
impl IIRConstant for IIR8{   const BITS: u8 = 0b011; }
impl IIRConstant for IIR16{  const BITS: u8 = 0b100; }