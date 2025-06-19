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
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let sda = peripherals.pins.gpio10;
    let scl = peripherals.pins.gpio8;

    let config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &config)?;

    // ANCHOR: bus_manager
    // 1. Create a bus manager that uses a critical section mutex for safe sharing.
    let bus = shared_bus::new_std!(I2cDriver<'_>).unwrap();
    // ANCHOR_END: bus_manager

    // 2. Acquire two independent proxies to the bus.
    let proxy_1 = bus.acquire_i2c();
    let proxy_2 = bus.acquire_i2c();

    // ANCHOR: shtc3_driver
    // 3. Instantiate the SHTC3 driver using its `new` method.
    let mut sht = Shtc3::new(proxy_1);
    // ANCHOR_END: shtc3_driver

    // 4. Read and print the device ID from the SHTC3.
    let device_id = sht.device_identifier().unwrap();
    println!("Device ID SHTC3: {:#04x}", device_id);

    // 5. Create an instance of the ICM42670p sensor (this part remains the same).
    let mut imu = Icm42670::new(proxy_2, Address::Primary).unwrap();
    let device_id = imu.device_id().unwrap();
    println!("Device ID ICM42670p: {:#02x}", device_id);
    
    imu.set_power_mode(imuPowerMode::GyroLowNoise).unwrap();

    loop {
        // 6. Read gyro data (unchanged).
        let gyro_data = imu.gyro_norm().unwrap();

        // ANCHOR: shtc3_measurement
        // 7. The shtc3 driver performs a measurement in a single blocking call.
        // It requires a delay provider to wait for the measurement to complete.
        let measurement = sht.measure(shtPowerMode::NormalMode, &mut FreeRtos).unwrap();
        // ANCHOR_END: shtc3_measurement

        // 8. Print all values.
        println!(
            "TEMP: {:.2} °C | HUM: {:.2} % | GYRO: X= {:.2}  Y= {:.2}  Z= {:.2}",
            measurement.temperature.as_degrees_celsius(),
            measurement.humidity.as_percent(),
            gyro_data.x,
            gyro_data.y,
            gyro_data.z,
        );

        FreeRtos.delay_ms(500u32);
    }
}
