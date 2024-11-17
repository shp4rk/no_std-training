// To easily test this you can connect GPIO2 and GPIO4
// This way we will receive was we send. (loopback)

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    dma::{Dma, DmaChannel, DmaPriority, DmaRxBuf, DmaTxBuf},
    dma_buffers,
    gpio::Io,
    prelude::*,
    spi::{master::Spi, SpiMode},
};
use esp_println::{print, println};

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let sclk = io.pins.gpio0;
    let miso = io.pins.gpio2;
    let mosi = io.pins.gpio4;
    let cs = io.pins.gpio5;

    // XXX: What are the DMA channels?
    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    // TODO(shpark): How does DMA work on esp32 devices?'
    // - descriptors?
    // - channels?
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000, 32000);
    let mut dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let mut dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer,).unwrap();

    let mut spi = Spi::new(peripherals.SPI2, 100.kHz(), SpiMode::Mode0)
        .with_pins(sclk, mosi, miso, cs)
        .with_dma(dma_channel.configure(false, DmaPriority::Priority0));

    let delay = Delay::new();

    // Prepare data in the TX buffer
    dma_tx_buf.as_mut_slice().fill(0x42);

    loop {
        // ANCHOR: transfer
        // To transfer much larger amounts of data we can use DMA and
        // the CPU can even do other things while the transfer is in progress
        // let mut data = [0x01u8, 0x02, 0x03, 0x04];

        // spi.transfer(&mut data).unwrap();

        // NOTE: Interestingly, this actually iniitates transfer of the data.
        let transfer = spi
            .dma_transfer(dma_rx_buf, dma_tx_buf)
            .map_err(|e| e.0)
            .unwrap();
        // ANCHOR_END: transfer

        // here the CPU could do other things while the transfer is taking done without using the CPU
        while !transfer.is_done() { println!("."); }

        // ANCHOR: transfer-wait
        // NOTE: `dma_transfer` took the ownership of `spi`, `dma_rx_buffer` and `dma_tx_buf`.
        // Here, `.wait()` method is used to get the ownership of these variables back...
        (spi, (dma_rx_buf, dma_tx_buf)) = transfer.wait();
        // ANCHOR_END: transfer-wait

        println!();
        println!(
            "Received {:x?} .. {:x?}",
            &dma_rx_buf.as_slice()[..10],
            &dma_rx_buf.as_slice().last_chunk::<10>().unwrap()
        );

        delay.delay_millis(2500u32);
    }
}
