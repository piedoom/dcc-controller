use dcc_rs::packets::{self, Direction, SerializeBuffer};
use embassy_time::Timer;
use esp_hal::rmt::{Channel, PulseCode, TxChannelAsync};

use crate::{
    devices::{Global, types},
    operations,
};

/// Transmit instructions and write them to the buffer to be bit-banged
#[embassy_executor::task]
pub async fn dcc_operations_transmit(driver: &'static Global<types::DccDriver<'static>>) {
    let mut buffer = SerializeBuffer::default();
    loop {
        // If buffer is empty (all transmit by the DCC step interrupt), continue
        if critical_section::with(|cs| operations::TX_BUFFER.borrow(cs).borrow().is_none()) {
            // Send a packet to be serialized
            let pkt = packets::SpeedAndDirection::builder()
                .address(3)
                .unwrap()
                .speed(3)
                .unwrap()
                .direction(Direction::Forward)
                .build();

            let len = pkt.serialize(&mut buffer).unwrap();

            critical_section::with(|cs| {
                // // Write data to the DCC driver
                // driver
                //     .borrow_ref_mut(cs)
                //     .as_mut()
                //     .unwrap()
                //     .write(buffer.get(0..len).unwrap())
                //     .unwrap();
                // Add the buffer
                operations::TX_BUFFER.replace(cs, Some((buffer, len)));
            });
        }
        Timer::after_millis(15).await; // Retry/Retransmit after minimum amount of time in spec
    }
}

#[embassy_executor::task]
pub(crate) async fn dcc_operations_step_v3(mut rmt: Channel<esp_hal::Async, 0>) {
    loop {
        let tx_buffer = critical_section::with(|cs| operations::TX_BUFFER.take(cs));
        let mut pulse_buffer = [PulseCode::empty(); packets::MAX_BITS + 1];
        if let Some((new_data, len)) = tx_buffer {
            for (bit, pulse) in new_data[..len].iter().zip(pulse_buffer[..len].iter_mut()) {
                *pulse = match *bit {
                    true => PulseCode::new(
                        false,
                        dcc_rs::ONE_MICROS as u16,
                        true,
                        dcc_rs::ONE_MICROS as u16,
                    ),
                    false => PulseCode::new(
                        false,
                        dcc_rs::ZERO_MICROS as u16,
                        true,
                        dcc_rs::ZERO_MICROS as u16,
                    ),
                };
            }
            rmt.transmit(&pulse_buffer[..len + 1]).await.unwrap();
        } else {
            let mut d = [PulseCode::new(
                false,
                dcc_rs::ZERO_MICROS as u16,
                true,
                dcc_rs::ZERO_MICROS as u16,
            ); 16];
            d[15] = PulseCode::empty();
            rmt.transmit(&d).await.unwrap();
        }
    }
}
