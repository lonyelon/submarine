use ahrs::{Ahrs, Madgwick};
use nalgebra::Vector3;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use std::error::Error;
use std::thread;
use std::time::Duration;

struct SensorData {
    gyro: Vector3<f64>,
    accel: Vector3<f64>,
    mag: Vector3<f64>,
    temp: f64,
}

fn normalize_sensor_value(a: u8, b: u8) -> f64 {
    (((a as i16) << 8 | b as i16) as f64) / ((std::i16::MAX) as f64)
}

fn init_sensor_mag(spi: &Spi, cs: &mut rppal::gpio::OutputPin) -> Result<(), Box<dyn Error>> {
    let write_buffer = [0x6A & 0x7F, 0x20];
    let mut read_buffer = [0; 2];

    cs.set_low();
    spi.transfer(&mut read_buffer, &write_buffer)?;
    cs.set_high();

    let write_buffer = [0x24 & 0x7F, 0x0D, 0x8C, 0x02, 0x88];
    let mut read_buffer = [0; 5];

    cs.set_low();
    spi.transfer(&mut read_buffer, &write_buffer)?;
    cs.set_high();

    Ok(())
}

fn read_sensor_data(spi: &Spi, cs: &mut rppal::gpio::OutputPin) -> Result<SensorData, Box<dyn Error>> {
    let mut read_buffer = [0; 21];
    let mut write_buffer = [0; 21];
    write_buffer[0] = 0x3B | 0x80;

    cs.set_low();
    spi.transfer(&mut read_buffer, &write_buffer)?;
    cs.set_high();

    Ok(SensorData {
        accel: Vector3::new(
            normalize_sensor_value(read_buffer[1], read_buffer[2]),
            normalize_sensor_value(read_buffer[3], read_buffer[4]),
            normalize_sensor_value(read_buffer[5], read_buffer[6]),
        ) * 2.0 * std::f64::consts::PI,
        temp: normalize_sensor_value(read_buffer[7], read_buffer[8]),
        gyro: Vector3::new(
            normalize_sensor_value(read_buffer[9], read_buffer[10]),
            normalize_sensor_value(read_buffer[11], read_buffer[12]),
            normalize_sensor_value(read_buffer[13], read_buffer[14]),
        ),
        mag: Vector3::new(
            normalize_sensor_value(read_buffer[15], read_buffer[16]),
            normalize_sensor_value(read_buffer[17], read_buffer[18]),
            normalize_sensor_value(read_buffer[19], read_buffer[20]),
        ),
    })
}

fn sensor_whoami(spi: &Spi, cs: &mut rppal::gpio::OutputPin) -> Result<u8, Box<dyn Error>> {
    let write_buffer = [0x75 | 0x80, 0];
    let mut read_buffer = [0; 2];

    cs.set_low();
    spi.transfer(&mut read_buffer, &write_buffer)?;
    cs.set_high();

    Ok(read_buffer[1])
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut ahrs = Madgwick::default();
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 100_000, Mode::Mode3)?;
    let gpio = rppal::gpio::Gpio::new()?;
    let mut cs = gpio.get(25)?.into_output();
    cs.set_high();

    // Check if we are using a mpu9250 sensor (that has a magnetometer).
    let main_sensor_id = sensor_whoami(&spi, &mut cs)?;
    if main_sensor_id == 0x71 {
        init_sensor_mag(&spi, &mut cs)?;
    } else {
        eprintln!("ERR Gyroscope is not mpu9250 (0x71), got {}",
            main_sensor_id);
    }

    loop {
        let mut data = read_sensor_data(&spi, &mut cs)?;

        // If we have no mag data then we need to add something to it since the
        // Madgwick filter uses it's norm in a division.
        if main_sensor_id != 0x71 {
            data.mag = Vector3::new(0.5, 0.0, 0.0);
        }

        let quat = ahrs.update(
            &data.gyro,
            &data.accel,
            &data.mag,
        ).unwrap();
        let val = quat.euler_angles();

        println!("ACCEL: [{:2.4}, {:2.4}, {:2.4}]",
            data.accel.x, data.accel.y, data.accel.z);
        println!("GYRO:  [{:2.4}, {:2.4}, {:2.4}]",
            data.gyro.x, data.gyro.y, data.gyro.z);
        println!("MAG:   [{:2.4}, {:2.4}, {:2.4}]",
            data.mag.x, data.mag.y, data.mag.z);
        println!("TEMP:  {:2.4}", data.temp);
        println!("ANG:   [{:2.4}, {:2.4}, {:2.4}]", val.0, val.1, val.2);
        println!();

        thread::sleep(Duration::from_millis(100));
    }
}
