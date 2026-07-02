pub struct ClimateSensorDataFrame<'a>{
    pub sensor_id: u8,
    pub readouts: &'a [u8],
}

