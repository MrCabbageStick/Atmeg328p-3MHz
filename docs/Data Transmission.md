# Data transmission
This project is supposed to transmit data wirelessly using cheap 433Mhz FM radio modules.\
Data will be gathered from all sensors and sent all at once.

## Data Labeling
Data "frame" will consist of all sensor readouts labeled with `sensor_id`, `unit_scale`, and `sensor_type`.
- `sensor_id` id of the sensor for given `sensor_type`. 
- `unit_scale` fraction of the unit data represent.
- `sensor_type` type of sensor used for the measurement (eg. thermometer, higrometer)

`unit_scale` will probably be read as: 10<sup>-(unit_scale + 1)</sup>*unit*, where *unit* represents unit for the given `sensor_type`. So if we wanted to represent hundredth part of the °C, `unit_scale` would have a value of `1`.

Data label will be an 8bit value:
```
Bit no: 8      7      6      5      4      3      2      1      0
        ┌─────────────┬─────────────┬───────────────────────────┐
        │  sensor_id  |  unit_sale  |       sensor_type         │
        └─────────────┴─────────────┴───────────────────────────┘
```

