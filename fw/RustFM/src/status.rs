use embassy_stm32::{
    gpio::{AnyPin, Level, Output, Speed},
    Peri,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::Timer;

pub enum SystemStatus {
    OK,
    FAIL,
}

pub type SharedStatus = Mutex<ThreadModeRawMutex, SystemStatus>;

#[embassy_executor::task]
pub async fn status_leds(
    led_ok_pin: Peri<'static, AnyPin>,
    led_fail_pin: Peri<'static, AnyPin>,
    status: &'static SharedStatus,
) {
    let mut led_ok = Output::new(led_ok_pin, Level::Low, Speed::Low);
    let mut led_fail = Output::new(led_fail_pin, Level::Low, Speed::Low);

    loop {
        led_ok.set_high();
        led_fail.set_low();
        Timer::after_millis(500).await;
        led_ok.set_low();
        led_fail.set_high();
        Timer::after_millis(500).await;
    }
}
