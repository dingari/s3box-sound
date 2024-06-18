#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    dma::{Dma, DmaPriority},
    dma_circular_buffers,
    gpio::Io,
    i2s::{DataFormat, I2s, I2sWriteDma, Standard},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
};

const NUM_SAMPLES: usize = 512;
const NUM_CHANNELS: usize = 2;
const DMA_BUFFER_SIZE: usize = NUM_SAMPLES * NUM_CHANNELS * core::mem::size_of::<i16>();

#[entry]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    log::info!("Init (sync) - {NUM_SAMPLES} samples - DMA buffer size: {DMA_BUFFER_SIZE}");

    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    let (tx_buffer, mut tx_descriptors, _, mut rx_descriptors) = dma_circular_buffers!(DMA_BUFFER_SIZE, 0);

    let i2s = I2s::new(
        peripherals.I2S0,
        Standard::Philips,
        DataFormat::Data16Channel16,
        44100u32.Hz(),
        dma_channel.configure(
            false,
            &mut tx_descriptors,
            &mut rx_descriptors,
            DmaPriority::Priority0,
        ),
        &clocks,
    );

    let mut i2s_tx = i2s
        .i2s_tx
        .with_bclk(io.pins.gpio1)
        .with_ws(io.pins.gpio2)
        .with_dout(io.pins.gpio3)
        .build();

    let mut sample_buffer = [0_i16; NUM_SAMPLES * NUM_CHANNELS];
    let sample_bytes = unsafe { core::slice::from_raw_parts(&sample_buffer as *const _ as *const u8, sample_buffer.len() * core::mem::size_of::<i16>()) };

    let mut buffer = tx_buffer;

    let sample_rate = 44100_f32;
    let freq = 440_f32;
    let mod_freq = 1_f32;

    let mut proc = s3box_sound::SampleProcessor::new(sample_rate, freq, mod_freq);

    let mut transfer = i2s_tx.write_dma_circular(&mut buffer).unwrap();
    loop {
        if transfer.available() > 0 {
            transfer
                .push_with(|dma_buf| {
                    let num_samples = dma_buf.len() / core::mem::size_of::<i16>();
                    let num_channels = 2;

                    proc.process_samples(&mut sample_buffer[0..num_samples], num_channels);

                    for i in 0..dma_buf.len() {
                        dma_buf[i] = sample_bytes[i];
                    }

                    dma_buf.len()
                })
                .unwrap();
        }
    }
}
