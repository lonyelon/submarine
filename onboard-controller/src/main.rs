use std::error::Error;
use std::thread;
use std::time::Duration;

mod mpu9250;

fn main() -> Result<(), Box<dyn Error>> {
    let mut main_sensor = mpu9250::Mpu9250::new(25)?;

    loop {
        let data = main_sensor.read_data()?;

        println!("ACCEL: [{:2.4}, {:2.4}, {:2.4}]",
            data.accel_d.x, data.accel_d.y, data.accel_d.z);
        println!("GYRO:  [{:2.4}, {:2.4}, {:2.4}]",
            data.gyro_d.x, data.gyro_d.y, data.gyro_d.z);
        println!("MAG:   [{:2.4}, {:2.4}, {:2.4}]",
            data.mag_d.x, data.mag_d.y, data.mag_d.z);
        println!("TEMP:  {:2.4}", data.temp);
        println!("ANG:   [{:2.4}, {:2.4}, {:2.4}]",
            data.yaw, data.pitch, data.roll);
        println!();

        thread::sleep(Duration::from_millis(100));
    }
}
