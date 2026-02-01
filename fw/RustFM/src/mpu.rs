use defmt::trace;
use embassy_stm32::{
    exti::ExtiInput,
    i2c::{I2c, Master},
    mode::Async,
    usart::Uart,
};
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{Receiver, Sender},
};
use embassy_time::{Delay, Instant};
use mpu6050_dmp::{
    calibration::CalibrationParameters, quaternion::Quaternion, sensor_async::Mpu6050,
    yaw_pitch_roll::YawPitchRoll,
};

pub const BUFFERED_QUATERNIONS: usize = 5;

#[embassy_executor::task]
pub async fn telemetry_sender(
    mut telemetry_port: Uart<'static, Async>,
    channel: Receiver<'static, ThreadModeRawMutex, Quaternion, BUFFERED_QUATERNIONS>,
) {
    let mut previous: Instant = Instant::now();
    let mut now: Instant;

    loop {
        // await until we receive a new quaternion packet from the sync channel
        let next_quaternion_value = channel.receive().await.normalize();
        // let ypr_format = YawPitchRoll::from(next_quaternion_value);

        now = Instant::now();
        let elapsed = now - previous;
        defmt::info!("Time elapsed since last data: {} ms", elapsed.as_millis());
        previous = now;

        let w = &next_quaternion_value.w;
        let x = &next_quaternion_value.x;
        let y = &next_quaternion_value.y;
        let z = &next_quaternion_value.z;

        let _send_result_w = telemetry_port.write(&w.to_le_bytes()).await;
        let _send_result_x = telemetry_port.write(&x.to_le_bytes()).await;
        let _send_result_y = telemetry_port.write(&y.to_le_bytes()).await;
        let _send_result_z = telemetry_port.write(&z.to_le_bytes()).await;

        telemetry_port.flush().await.unwrap();
        defmt::info! {"Data: {} {} {} {}", w, x, y, z};
    }
}

#[embassy_executor::task]
pub async fn read_mpu(
    iic: I2c<'static, Async, Master>,
    mut ext: ExtiInput<'static>,
    channel: Sender<'static, ThreadModeRawMutex, Quaternion, BUFFERED_QUATERNIONS>,
) {
    trace! {"Entering MPU thread"};
    let mut mpu = Mpu6050::new(iic, mpu6050_dmp::address::Address::default())
        .await
        .unwrap();
    // initialize the DMP processor for the MPU
    mpu.initialize_dmp(&mut Delay).await.unwrap();

    // Configure calibration parameters
    // let calibration_params = CalibrationParameters::new(
    //     mpu6050_dmp::accel::AccelFullScale::G2,
    //     mpu6050_dmp::gyro::GyroFullScale::Deg2000,
    //     mpu6050_dmp::calibration::ReferenceGravity::ZN,
    // );
    // trace!("Calibrating Sensor");
    // mpu
    //     .calibrate(&mut Delay, &calibration_params)
    //     .await
    //     .unwrap();
    // trace!("Sensor Calibrated");
    mpu.set_clock_source(mpu6050_dmp::clock_source::ClockSource::Xgyro).await.unwrap();
    mpu.enable_dmp().await.unwrap();
    mpu.load_firmware().await.unwrap();
    mpu.boot_firmware().await.unwrap();
    mpu.set_sample_rate_divider(4).await.unwrap();
    mpu.set_digital_lowpass_filter(mpu6050_dmp::config::DigitalLowPassFilter::Filter0)
        .await
        .unwrap();

    let mut fifo: [u8; 28] = [0; 28];

    // set up the interrupt so we receive data from the internal mpu6050 dmp,
    // combining gyro and accel
    mpu.enable_fifo().await.unwrap();
    trace! {"Enabling FIFO interrupt"};
    mpu.interrupt_fifo_oflow_en().await.unwrap();

    loop {
        // block until we get an interrupt from the MPU line
        ext.wait_for_rising_edge().await;
        // read the combined data from the MPU fifo. It sends 28 byte packets, of which the first
        // 16 are the quaternion data.
        mpu.read_fifo(&mut fifo).await.unwrap();
        // obtain the first 16 quaternion packets
        let quaternion_packet = Quaternion::from_bytes(&fifo[..16]).unwrap().normalize();
        // send quaternion value to sync channel
        channel.send(quaternion_packet).await;

        // clear the pending interrupt and wait for the next
        mpu.reset_fifo().await.unwrap();
        mpu.interrupt_read_clear().await.unwrap();
    }
}
