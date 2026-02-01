use defmt::{trace, error};

use embassy_stm32::can::filter::Mask32;
use embassy_stm32::can::{
    Fifo, Frame,
};

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Receiver};
use embassy_time::Timer;

pub const CAN_BUFFER: usize = 10;

#[embassy_executor::task]
pub async fn comm_task(mut can_bus: embassy_stm32::can::Can<'static>, chan: Receiver<'static, ThreadModeRawMutex, u32, CAN_BUFFER>) {
    // enable the fifo 0 bank for storing the CAN message queue
    can_bus.modify_filters().enable_bank(0, Fifo::Fifo0, Mask32::accept_all());
    // enable loopback for testing
    can_bus.modify_config()
        .set_loopback(true) // Receive own frames
        .set_silent(true)
        .set_bitrate(250_000);
    can_bus.enable().await;

    loop {
        // wait for new messages to transmit
        let message = chan.receive().await;
        // TODO: check what to send in the message ID
        let frame = Frame::new_extended(0x123456F, &message.to_le_bytes()).unwrap();
        trace!("Writing CAN frame");

        _ = can_bus.write(&frame).await;
    }
}
