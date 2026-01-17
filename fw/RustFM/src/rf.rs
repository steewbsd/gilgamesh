use embassy_stm32::{
    gpio::{AnyPin, Level, Output, Speed},
    Peri,
};
use embassy_time::Timer;

#[embassy_executor::task]
pub async fn transmit(p: Peri<'static, AnyPin>) {
    let mut txpin = Output::new(p, Level::Low, Speed::VeryHigh);
    // txpin.set_low();
    // Timer::after_millis(1).await;
    // txpin.set_high();
    // Timer::after_millis(1).await;
    // txpin.set_low();

    loop {
        txpin.toggle();
        Timer::after_millis(1).await;
    }
}
