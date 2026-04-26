/*
VEML7700 Ambient Light Sensor configuration 
is defined by values in registers 0x00-0x03.
Module can act as a threshold switch, 
but raw light value can be accessed too.

---

## Register breakdown

### 0x00 — CONFIGURATION REGISTER
Controls measurement behavior, interrupts, and power state.
BIT(s)   FIELD        DESCRIPTION
15:13    reserved     Set to 000b
12:11    ALS_SM       Sensitivity mode
10       reserved     Set to 0b
9:6      ALS_IT       Integration time (measurement duration)
5:4      ALS_PERS     Interrupt persistence (consecutive threshold hits)
3:2      reserved     Set to 00b
1        ALS_INT_EN   ALS interrupt enable
0        ALS_SD       Shutdown mode


### 0x01 — ALS_WH (HIGH THRESHOLD)
BIT(s)   FIELD        DESCRIPTION
15:8     MSB          High threshold value
7:0      LSB          High threshold value


### 0x02 — ALS_WL (LOW THRESHOLD)
BIT(s)   FIELD        DESCRIPTION
15:8     MSB          Low threshold value
7:0      LSB          Low threshold value


### 0x03 — POWER SAVING

BIT(s)   FIELD        DESCRIPTION
15:3     reserved     Set to 0
2:1      PSM          Power saving mode (measurement interval)
0        PSM_EN       Power saving enable
*/

// For more customizable naming disable some code linting
// Without it `const BITS_0x00` would be `const BITS_0X0`
// and `struct AlsSm_x1_8` would be `AlsSm1Over8` or something similar
#![allow(non_camel_case_types, non_upper_case_globals)]

// 0x00 register offsets
const SM_OFFSET: u16 = 11;
const IT_OFFSET: u16 = 6;
const SD_OFFSET: u16 = 0;
// 0x03 register offsets
const PSM_OFFSET: u16 = 1;
const PSM_EN_OFFSET: u16 = 0;

/// Veml7700 configuration without interrupt settings
pub trait Config{
    type Sm: AlsSm;
    type It: AlsIt;
    type Psm: AlsPsm;

    /// Values of registers
    const BITS_0x00: u16 = Self::Sm::BITS << SM_OFFSET | Self::It::BITS << IT_OFFSET;
    const BITS_0x03: u16 = Self::Psm::BITS << PSM_OFFSET | 0b1 << PSM_EN_OFFSET;
    // Registers 0x01 and 0x02 are unused
    const BITS_0x01: u16 = 0;
    const BITS_0x02: u16 = 0;

    /// Lux calculation numerator
    const LUX_NUM: u32;
    /// Lux calculation denominator
    const LUX_DEN: u32;
}

// Default configurations
pub struct ConfigFastLowPower;

impl Config for ConfigFastLowPower{
    type It = AlsIt25ms;
    type Sm = AlsSm_x2;
    type Psm = AlsPsm1;

    // Sensitivity is scaled by 1000
    const LUX_NUM: u32 = 10 * 1000;
    // Sesitivity of 0.042 scaled scaled by 1000
    // and refresh time of 600ms
    const LUX_DEN: u32 = 42 * 600;
}


/// Sensitivity mode
pub trait AlsSm{
    /// Bits for sensitivity mode with **no** offset
    const BITS: u16;
}

/// Sensitivity x1
pub struct AlsSm_x1;
/// Sensitivity x2
pub struct AlsSm_x2;
/// Sensitivity x1/4
pub struct AlsSm_x1_4;
/// Sensitivity x1/8
pub struct AlsSm_x1_8;

impl AlsSm for AlsSm_x1{   const BITS: u16 = 0b00; }
impl AlsSm for AlsSm_x2{   const BITS: u16 = 0b01; }
impl AlsSm for AlsSm_x1_4{ const BITS: u16 = 0b10; }
impl AlsSm for AlsSm_x1_8{ const BITS: u16 = 0b11; }



/// Integration time
pub trait AlsIt{
    /// Bits for integration time with **no** offset
    const BITS: u16;
}

pub struct AlsIt25ms;
pub struct AlsIt50ms;
pub struct AlsIt100ms;
pub struct AlsIt200ms;
pub struct AlsIt400ms;
pub struct AlsIt800ms;

impl AlsIt for AlsIt25ms{  const BITS: u16 = 0b1100; }
impl AlsIt for AlsIt50ms{  const BITS: u16 = 0b1000; }
impl AlsIt for AlsIt100ms{ const BITS: u16 = 0b0000; }
impl AlsIt for AlsIt200ms{ const BITS: u16 = 0b0001; }
impl AlsIt for AlsIt400ms{ const BITS: u16 = 0b0010; }
impl AlsIt for AlsIt800ms{ const BITS: u16 = 0b0011; }



/// Power saving mode
pub trait AlsPsm{
    /// Bits for power saving mode with **no** offset
    const BITS: u16;
}

pub struct AlsPsm1;
pub struct AlsPsm2;
pub struct AlsPsm3;
pub struct AlsPsm4;

impl AlsPsm for AlsPsm1{ const BITS: u16 = 0b00; }
impl AlsPsm for AlsPsm2{ const BITS: u16 = 0b01; }
impl AlsPsm for AlsPsm3{ const BITS: u16 = 0b10; }
impl AlsPsm for AlsPsm4{ const BITS: u16 = 0b11; }