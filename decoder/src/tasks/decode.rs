use esp_hal::{
    Async,
    rmt::{self, PulseCode, RxChannelAsync},
};

/// Receive the DCC signal
#[embassy_executor::task]
pub async fn receive_data(mut rmt: rmt::Channel<Async, 2>) {
    loop {
        let mut buffer: [u32; 48] = [PulseCode::empty(); 48];
        rmt.receive(&mut buffer).await.unwrap();
        for code in buffer.iter() {
            if *code != PulseCode::empty() {
                esp_println::println!("hi");
            }
        }
    }
}
