use ahrs::{Ahrs, Madgwick};
use nalgebra::Vector3;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use std::error::Error;

pub struct SensorData {
    // Absolute values.
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
    pub temp: f64,

    // Deltas for last measurement.
    pub gyro_d: Vector3<f64>,
    pub accel_d: Vector3<f64>,
    pub mag_d: Vector3<f64>,
}

pub struct Mpu9250 {
    spi: Spi,
    cs: rppal::gpio::OutputPin,
    ahrs: Madgwick<f64>,
    id: u8,
}

impl Mpu9250 {
    pub fn new(cs_pin: u8) -> Result<Mpu9250, Box<dyn Error>> {
        let gpio = rppal::gpio::Gpio::new()?;
        let mut cs = gpio.get(cs_pin)?.into_output();
        cs.set_high();

        let mut sensor = Mpu9250 {
            spi: Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode3)?,
            cs: cs,
            ahrs: Madgwick::default(),
            id: 0,
        };

        sensor.id = sensor._whoami()?;

        match sensor.id {
            0x71 => {
                sensor._init_mag_sensor()?;
                Ok(sensor)
            },
            0x70 => Ok(sensor),
            _ => panic!("Sensor on pin {} is not an MPU(925[05]|6500)", cs_pin),
        }

    }

    pub fn read_data(&mut self) -> Result<SensorData, Box<dyn Error>> {
        let mut read_buffer = [0; 21];
        let mut write_buffer = [0; 21];
        write_buffer[0] = 0x3B | 0x80;

        self.cs.set_low();
        self.spi.transfer(&mut read_buffer, &write_buffer)?;
        self.cs.set_high();

        let accel = Vector3::new(
            Mpu9250::_normalize_value(read_buffer[1], read_buffer[2]),
            Mpu9250::_normalize_value(read_buffer[3], read_buffer[4]),
            Mpu9250::_normalize_value(read_buffer[5], read_buffer[6]),
        ) * 2.0 * std::f64::consts::PI;

        let gyro = Vector3::new(
            Mpu9250::_normalize_value(read_buffer[9], read_buffer[10]),
            Mpu9250::_normalize_value(read_buffer[11], read_buffer[12]),
            Mpu9250::_normalize_value(read_buffer[13], read_buffer[14]),
        );

        let mut mag = Vector3::new(
            Mpu9250::_normalize_value(read_buffer[15], read_buffer[16]),
            Mpu9250::_normalize_value(read_buffer[17], read_buffer[18]),
            Mpu9250::_normalize_value(read_buffer[19], read_buffer[20]),
        );
        
        // If we have no mag sensor we need to make this up because then the
        // Madgwick filter will not work.
        if self.id == 0x70 {
            mag += Vector3::new(0.5, 0.0, 0.0);
        }

        let quat = self.ahrs.update(
           &gyro,
           &accel,
           &mag,
        )?;
        let (yaw, pitch, roll) = quat.euler_angles();

        Ok(SensorData {
            temp: Mpu9250::_normalize_value(read_buffer[7], read_buffer[8]),
            accel_d: accel,
            gyro_d: gyro,
            mag_d: mag,
            yaw: yaw,
            pitch: pitch,
            roll: roll,
        })
    }
    
    fn _whoami(&mut self) -> Result<u8, Box<dyn Error>> {
        let write_buffer = [0x75 | 0x80, 0];
        let mut read_buffer = [0; 2];

        self.cs.set_low();
        self.spi.transfer(&mut read_buffer, &write_buffer)?;
        self.cs.set_high();

        Ok(read_buffer[1])
    }
    
    fn _init_mag_sensor(&mut self) -> Result<(), Box<dyn Error>> {
        let write_buffer = [0x6A & 0x7F, 0x20];
        let mut read_buffer = [0; 2];

        self.cs.set_low();
        self.spi.transfer(&mut read_buffer, &write_buffer)?;
        self.cs.set_high();

        let write_buffer = [0x24 & 0x7F, 0x0D, 0x8C, 0x02, 0x88];
        let mut read_buffer = [0; 5];

        self.cs.set_low();
        self.spi.transfer(&mut read_buffer, &write_buffer)?;
        self.cs.set_high();

        Ok(())
    }

    fn _normalize_value(a: u8, b: u8) -> f64 {
        (((a as i16) << 8 | b as i16) as f64) / ((std::i16::MAX) as f64)
    }
}
