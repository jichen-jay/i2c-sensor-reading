use anyhow::Result;
use embedded_hal::delay::DelayNs; // Note: DelayMs is now part of the DelayNs trait
use esp_idf_svc::hal::{
    delay::FreeRtos,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    prelude::*,
};
use icm42670::{Address, Icm42670, PowerMode as imuPowerMode};

// ANCHOR: new_imports
// Use the modern bus sharing crate
use embedded_hal_bus::i2c::CriticalSectionDevice;
// Use the v1.0.0 compatible SHTC3 driver
use shtc3::{self, PowerMode as shtPowerMode, Shtc3};
// ANCHOR_END: new_imports


fn main() -> Result<()> {
}
