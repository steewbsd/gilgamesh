use cortex_m::prelude::_embedded_hal_blocking_serial_Write;
use embassy_stm32::{
    exti::ExtiInput,
    i2c::{I2c, Master},
    mode::Async,
    usart::Uart,
};
use embassy_sync::{blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex, ThreadModeRawMutex}, channel::{Channel, Receiver, Sender}, rwlock::RwLock};
use embassy_time::{Delay, WithTimeout};
use heapless::pool::arc::Arc;
use mpu6050_dmp::{
    calibration::CalibrationParameters, quaternion::Quaternion, sensor_async::Mpu6050,
    yaw_pitch_roll::YawPitchRoll,
};

use defmt::info;

// Holds the current body state
struct MotionState {
    // gyroscope readings
    yaw: f32,
    pitch: f32,
    roll: f32,
    // accelerometer readings
    accel_x: f32,
    accel_y: f32,
    accel_z: f32,
    // magnetometer readings
        
}

pub const BUFFERED_QUATERNIONS: usize = 5;

#[embassy_executor::task]
pub async fn telemetry_sender(
    mut telemetry_port: Uart<'static, Async>,
    channel: Receiver<'static, ThreadModeRawMutex, Quaternion, BUFFERED_QUATERNIONS>,
) {
    loop {
        // await until we receive a new quaternion packet from the sync channel
        let next_quaternion_value = channel.receive().await;
        // write the received values in the telemetry UART
        // TODO: data synchronization for the receiver
        telemetry_port.write(&next_quaternion_value.w.to_le_bytes()).await;
        telemetry_port.write(&next_quaternion_value.x.to_le_bytes()).await;
        telemetry_port.write(&next_quaternion_value.y.to_le_bytes()).await;
        telemetry_port.write(&next_quaternion_value.z.to_le_bytes()).await;
    }
}


#[embassy_executor::task]
pub async fn read_mpu(
    iic: I2c<'static, Async, Master>,
    mut ext: ExtiInput<'static>,
    channel: Sender<'static, ThreadModeRawMutex, Quaternion, BUFFERED_QUATERNIONS>,
) {
    let mut mpu = Mpu6050::new(iic, mpu6050_dmp::address::Address::default())
        .await
        .unwrap();
    // initialize the DMP processor for the MPU
    mpu.initialize_dmp(&mut Delay).await.unwrap();
    
    // Configure calibration parameters
    // let _calibration_params = CalibrationParameters::new(
    //     mpu6050_dmp::accel::AccelFullScale::G2,
    //     mpu6050_dmp::gyro::GyroFullScale::Deg2000,
    //     mpu6050_dmp::calibration::ReferenceGravity::ZN,
    // );
    // info!("Calibrating Sensor");
    // mpu
    //     .calibrate(&mut Delay, &calibration_params)
    //     .await
    //     .unwrap();
    // info!("Sensor Calibrated");
    
    mpu.enable_dmp().await.unwrap();
    mpu.load_firmware().await.unwrap();
    mpu.boot_firmware().await.unwrap();
    mpu.set_sample_rate_divider(9).await.unwrap();
    mpu.set_digital_lowpass_filter(mpu6050_dmp::config::DigitalLowPassFilter::Filter1)
        .await
        .unwrap();

    // mpu.set_clock_source(mpu6050_dmp::clock_source::ClockSource::Xgyro).unwrap();
    let mut fifo: [u8; 28] = [0; 28];

    // set up the interrupt so we receive data from the internal mpu6050 dmp,
    // combining gyro and accel
    mpu.enable_fifo().await.unwrap();
    mpu.interrupt_data_ready_en().await.unwrap();
    
    loop {
        // block until we get an interrupt from the MPU line
        ext.wait_for_rising_edge().await;
        // read the combined data from the MPU fifo. It sends 28 byte packets, of which the first
        // 16 are the quaternion data.
        mpu.read_fifo(&mut fifo).await.unwrap();
        // obtain the first 16 quaternion packets
        let quaternion_packet = Quaternion::from_bytes(&fifo[..16]).unwrap().normalize();
        // send quaternion value to sync channel
        channel.send(quaternion_packet);

        // clear the pending interrupt and wait for the next
        mpu.reset_fifo().await.unwrap();
        mpu.interrupt_read_clear().await.unwrap();
    }
}
