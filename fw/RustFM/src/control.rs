use core::ops::{Add, Sub};

use defmt::{info, trace};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Receiver};
use mpu6050_dmp::{quaternion::Quaternion, yaw_pitch_roll::YawPitchRoll};

use crate::mpu::BUFFERED_QUATERNIONS;

// represents the motor's possible turning direction (clockwise and counter-clockwise)
pub enum MotorTurningState {
    CW,
    CCW
}

// holds the motor state, including its angular speed and direction.
pub struct Motor {
    angular_rotation_speed: f64,
    turning: MotorTurningState,
}

// represents the body's total motor count
pub const MOTOR_NUM: usize = 4;

// holds the system state for control, including the motor rotation speeds, the current velocity and altitude,
// and several other data relevant to system control
pub struct Controller {
    motors: [Motor; MOTOR_NUM],
    altitude: f64,
    velocity: f64,
    // stores the last reference frame
    last_frame: YawPitchRoll,
    // stores the target body attitude desired, from a reference frame
    attitude: YawPitchRoll
    // store the last and next quaternion values for the control calculus
}

pub struct Frame {
    yaw: f32,
    pitch: f32,
    roll: f32
}

impl Add for Frame {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            yaw: self.yaw + other.yaw,
            pitch: self.pitch + other.pitch,
            roll: self.roll + other.roll
        }
    }
}

impl Sub for Frame {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            yaw: self.yaw - other.yaw,
            pitch: self.pitch - other.pitch,
            roll: self.roll - other.roll
        }
    }

}

#[embassy_executor::task]
pub async fn update_control_loop(
    quaternion_channel: Receiver<'static, ThreadModeRawMutex, Quaternion, BUFFERED_QUATERNIONS>,
    target_frame: Frame,
) {
    let mut last_yaw: f32 = 0.0;
    let mut last_pitch: f32 = 0.0;
    let mut last_roll: f32 = 0.0;

    const THRESHOLD: f32 = 3.0;
    
    loop {
        let next_quaternion_value = quaternion_channel.receive().await;
        let ypr_format = YawPitchRoll::from(next_quaternion_value);
        
        let yaw_deg = ypr_format.yaw * 180.0 / core::f32::consts::PI;
        let pitch_deg = ypr_format.pitch * 180.0 / core::f32::consts::PI;
        let roll_deg = ypr_format.roll * 180.0 / core::f32::consts::PI;

        let y_diff = yaw_deg - last_yaw;
        let p_diff = pitch_deg - last_pitch;
        let r_diff = roll_deg - last_roll;

        // filter the new value and see if it has differed in more than THRESHOLD than the
        // previous. If not, ignore it.
        if f32::abs(y_diff) > THRESHOLD
            || f32::abs(p_diff) > THRESHOLD
            || f32::abs(r_diff) > THRESHOLD
        {
            info!("YPR differential: y:{}, p:{}, r:{}", y_diff, p_diff, r_diff);
            last_yaw = yaw_deg;
            last_pitch = pitch_deg;
            last_roll = roll_deg;
        }
    }
}
