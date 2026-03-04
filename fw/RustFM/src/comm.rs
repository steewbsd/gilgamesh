use core::any::Any;

use defmt::{trace, error};

use embassy_stm32::can::{Frame};

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Receiver};
use embassy_time::Timer;

pub const CAN_BUFFER: usize = 10;

#[embassy_executor::task]
pub async fn can_tx_task(mut can_bus: embassy_stm32::can::CanTx<'static>, chan: Receiver<'static, ThreadModeRawMutex, u32, CAN_BUFFER>) {
    
    loop {
        // wait for new messages to transmit
        // let message = chan.receive().await;
        let message = "Test";
        // TODO: check what to send in the message ID
        // let frame = Frame::new_extended(0x123456F, &message.to_le_bytes()).unwrap();
        let frame = Frame::new_extended(0x123456F, message.as_bytes()).unwrap();
        trace!("Writing CAN frame");

        _ = can_bus.write(&frame).await;
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
pub async fn can_rx_task(mut can_bus: embassy_stm32::can::CanRx<'static>) {
    
    loop {
        let message = can_bus.read().await;
        trace!("Receiving CAN...");
        
        let contents: &[u8];
        // watch for possible bus errors
        match message {
            Ok(envelope) => {
                contents = envelope.frame.data();
                trace!("Received CAN message: {}", contents);
            },
            _ => {
                error!("Error receiving CAN bus message!");
            }
        }
    }
}
