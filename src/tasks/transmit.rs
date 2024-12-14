use dcc_rs::packets::{self, Direction, SerializeBuffer};
use embassy_time::Timer;
use esp_hal::prelude::*;

use crate::operations;

/// Transmit instructions and write them to the buffer to be bit-banged
#[embassy_executor::task]
pub async fn dcc_operations_transmit() {
    loop {
        // If buffer is empty (all transmit by the DCC step interrupt), continue
        if critical_section::with(|cs| operations::TX_BUFFER.borrow(cs).borrow().is_none()) {
            let mut buffer = SerializeBuffer::default();

            // Send a packet to be serialized
            let pkt = packets::SpeedAndDirection::builder()
                .address(3)
                .unwrap()
                .speed(14)
                .unwrap()
                .direction(Direction::Forward)
                .build();

            let len = pkt.serialize(&mut buffer).unwrap();

            critical_section::with(|cs| {
                // Write data to the DCC driver
                operations::DRIVER
                    .borrow_ref_mut(cs)
                    .as_mut()
                    .unwrap()
                    .write(buffer.get(0..len).unwrap())
                    .unwrap();
                // Add the buffer
                operations::TX_BUFFER.replace(cs, Some((buffer, len)));
            });
        }
        Timer::after_millis(15).await; // Retry/Retransmit after minimum amount of time in spec
    }
}

/// Transmit DCC signal via bitbanging
#[handler]
pub fn dcc_operations_step() {
    let tx_buffer = critical_section::with(|cs| operations::TX_BUFFER.take(cs));
    let mut dcc_handler = critical_section::with(|cs| operations::DRIVER.take(cs).unwrap());

    // Only write to the handler if we have new data in the buffer
    if let Some((new_data, len)) = tx_buffer {
        dcc_handler.write(&new_data[..len]).unwrap();
    }

    // Set the delay until the next level change
    let new_delay = dcc_handler.tick().unwrap();

    let timer = critical_section::with(|cs| operations::TIMER.take(cs).unwrap());

    timer.load_value((new_delay as u64).micros()).unwrap();
    timer.start();

    timer.clear_interrupt();

    // Replace devices
    critical_section::with(|cs| operations::DRIVER.replace(cs, Some(dcc_handler)));
    critical_section::with(|cs| operations::TIMER.replace(cs, Some(timer)));
}
