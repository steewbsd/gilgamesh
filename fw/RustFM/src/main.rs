#![no_std]
#![no_main]

mod mpu;
mod rf;

use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex},
    rwlock::RwLock,
};

use heapless::pool::arc::Arc;
use mpu::read_mpu;
use mpu::BUFFERED_QUATERNIONS;
use mpu6050_dmp::quaternion::Quaternion;
use rf::transmit;

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
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1_EV => i2c::EventInterruptHandler<embassy_stm32::peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<embassy_stm32::peripherals::I2C1>;
    USART3 => usart::InterruptHandler<embassy_stm32::peripherals::USART3>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();

    config.rcc.hse = Some(rcc::Hse {
        freq: embassy_stm32::time::Hertz(12_000_000),
        mode: rcc::HseMode::Oscillator,
    });
    let p = embassy_stm32::init(config);

    // let iic = I2c::new_blocking(p.I2C1, p.PA9, p.PA10, i2c::Config::default());
    let iic = I2c::new(
        p.I2C1,
        p.PA9,
        p.PA10,
        Irqs,
        p.DMA1_CH6,
        p.DMA1_CH7,
        i2c::Config::default(),
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
    tmtry_uart.set_baudrate(9600).unwrap();

    // dedicated task for the RF transmitter
    spawner.spawn(transmit(txpin.into())).unwrap();
    // dedicated task for MPU data readings
    spawner
        .spawn(read_mpu(iic, imu_int.into(), QUATERNION_CHANNEL.sender()))
        .unwrap();
    // dedicated task for UART telemetry
    spawner.spawn(telemetry_sender(tmtry_uart, QUATERNION_CHANNEL.receiver()));
    // dedicated task for physical status leds
    // TODO: replace with timer interrupt
    spawner.spawn(status_leds(ok_pin.into())).unwrap();
    Timer::after_millis(500).await;
    spawner.spawn(status_leds(fail_pin.into())).unwrap();
}

#[embassy_executor::task(pool_size = 2)]
async fn status_leds(p: Peri<'static, AnyPin>) {
    let mut led = Output::new(p, Level::Low, Speed::Low);

    loop {
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
        Timer::after_millis(500).await;
    }
}
