#![no_std]
#![no_main]

mod mpu;
mod rf;
mod status;

use defmt::trace;
use embassy_stm32::time::Hertz;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;

use embassy_sync::mutex::Mutex;
use mpu::read_mpu;
use mpu::BUFFERED_QUATERNIONS;
use mpu6050_dmp::quaternion::Quaternion;
use rf::transmit;
use status::SystemStatus;

use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    exti::ExtiInput,
    gpio::{AnyPin, Level, Output, Speed},
    i2c::{self, I2c},
    rcc::{self},
    usart::{self, Uart},
    Config, Peri,
};
use embassy_sync::channel::Channel;

use crate::mpu::telemetry_sender;
use crate::status::status_leds;
use crate::status::SharedStatus;
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<embassy_stm32::peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<embassy_stm32::peripherals::I2C1>;
    USART3 => usart::InterruptHandler<embassy_stm32::peripherals::USART3>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    trace! {"Starting system up..."};
    let mut config = Config::default();

    config.rcc.hse = Some(rcc::Hse {
        freq: embassy_stm32::time::Hertz(12_000_000),
        mode: rcc::HseMode::Oscillator,
    });

    config.rcc.sys = rcc::Sysclk::PLL1_R;
    config.rcc.pll = Some(rcc::Pll {
        source: rcc::PllSource::HSE,
        prediv: rcc::PllPreDiv::DIV2,
        mul: rcc::PllMul::MUL8,
        divp: None,
        divq: Some(rcc::PllQDiv::DIV2), // PLL1_Q clock (32 / 2 * 6 / 2), used for RNG
        divr: Some(rcc::PllRDiv::DIV2), // sysclk 48Mhz clock (32 / 2 * 6 / 2)
    });

    let p = embassy_stm32::init(config);
    let mut i2c_config = i2c::Config::default();
    i2c_config.frequency = Hertz::khz(400);
    i2c_config.gpio_speed = Speed::VeryHigh;

    let iic = I2c::new(
        p.I2C1, p.PA9, p.PA10, Irqs, p.DMA1_CH6, p.DMA1_CH7, i2c_config,
    );
    let _lsb_pin = Output::new(p.PA11, Level::Low, Speed::Low);

    let ok_pin = p.PC14;
    let fail_pin = p.PC15;
    let txpin = p.PA6;

    let _rxen = Output::new(p.PB0, Level::High, Speed::Low);
    let _txen = Output::new(p.PB1, Level::Low, Speed::Low);

    core::mem::forget(_rxen);
    core::mem::forget(_txen);
    core::mem::forget(_lsb_pin);

    let imu_int = ExtiInput::new(p.PA12, p.EXTI12, embassy_stm32::gpio::Pull::Down);

    // create shared channel between the MPU polling thread and the UART sender for the quaternion data buffer.
    static QUATERNION_CHANNEL: Channel<ThreadModeRawMutex, Quaternion, BUFFERED_QUATERNIONS> =
        Channel::<ThreadModeRawMutex, Quaternion, BUFFERED_QUATERNIONS>::new();
    static SHARED_STATUS: SharedStatus = Mutex::new(SystemStatus::FAIL);

    let tmtry_uart = Uart::new(
        p.USART3,
        p.PC5,
        p.PC4,
        Irqs,
        p.DMA1_CH2,
        p.DMA1_CH3,
        usart::Config::default(),
    )
    .unwrap();
    tmtry_uart.set_baudrate(115200).unwrap();

    trace! {"Initializing tasks..."};

    // dedicated task for the RF transmitter
    spawner.spawn(transmit(txpin.into())).unwrap();
    // dedicated task for MPU data readings
    spawner
        .spawn(read_mpu(iic, imu_int, QUATERNION_CHANNEL.sender()))
        .unwrap();
    // dedicated task for UART telemetry
    spawner
        .spawn(telemetry_sender(tmtry_uart, QUATERNION_CHANNEL.receiver()))
        .unwrap();
    // dedicated task for physical status leds
    // TODO: replace with timer interrupt
    // spawner.spawn(status_leds(ok_pin.into())).unwrap();
    // Timer::after_millis(500).await;
    // spawner.spawn(status_leds(fail_pin.into())).unwrap();
    spawner
        .spawn(status_leds(ok_pin.into(), fail_pin.into(), &SHARED_STATUS))
        .unwrap();
}
