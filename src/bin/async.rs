#![no_std]
#![no_main]

use embassy_executor::Spawner;

use esp_backtrace as _;
use esp_hal::i2s::asynch::I2sWriteDmaAsync;
use esp_hal::{
    clock::ClockControl,
    dma::{Dma, DmaPriority},
    dma_circular_buffers,
    gpio::Io,
    i2s::{DataFormat, I2s, Standard},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
};

// This is the only 'good' value as it seems
const NUM_SAMPLES: usize = 128;
const NUM_CHANNELS: usize = 2;
const DMA_BUFFER_SIZE: usize = NUM_SAMPLES * NUM_CHANNELS * core::mem::size_of::<i16>();
const SAMPLE_RATE: f32 = 44100.0;

#[main]
async fn main(_spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    log::info!("Init (async) - {NUM_SAMPLES} samples - DMA buffer size: {DMA_BUFFER_SIZE}");

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
        (SAMPLE_RATE as u32).Hz(),
        dma_channel.configure_for_async(
            false,
            &mut tx_descriptors,
            &mut rx_descriptors,
            DmaPriority::Priority0,
        ),
        &clocks,
    );

    let i2s_tx = i2s
        .i2s_tx
        .with_bclk(io.pins.gpio1)
        .with_ws(io.pins.gpio2)
        .with_dout(io.pins.gpio3)
        .build();

    let mut sample_buffer = [0_i16; NUM_SAMPLES * NUM_CHANNELS];
    let sample_bytes = unsafe { core::slice::from_raw_parts(&sample_buffer as *const _ as *const u8, sample_buffer.len() * core::mem::size_of::<i16>()) };

    let buffer = tx_buffer;

    let freq = 440_f32;
    let mod_freq = 1_f32;

    let mut proc = s3box_sound::SampleProcessor::new(SAMPLE_RATE, freq, mod_freq);

    let mut transfer = i2s_tx.write_dma_circular_async(buffer).unwrap();
    loop {
        transfer
            .push_with(|dma_buf| {
                let num_samples = dma_buf.len() / core::mem::size_of::<i16>();
                let num_samples_even = num_samples - num_samples % NUM_CHANNELS;

                proc.process_samples(&mut sample_buffer[0..num_samples_even], NUM_CHANNELS);

                let num_bytes = num_samples_even * core::mem::size_of::<i16>();
                for i in 0..num_bytes {
                    dma_buf[i] = sample_bytes[i];
                }

                num_bytes
            })
            .await
            .unwrap();
    }
}
