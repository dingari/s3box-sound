# Audio processing on the ESP32-S3 (WIP)

Demos/experiments with audio processing on the ESP32-S3 via I2S, forked from [BjoernQ's async implementation](https://github.com/bjoernQ/s3box-sound/tree/async).

My hope is to be able to synthesize audio on the ESP32-S3. For these examples I've been using a T-Display-S3 by LilyGo and a PCM5102A DAC.

The crate has two flavors:

* `cargo run --release --features async --bin async` will use the `esp_hal::asynch::i2s` async API
* `cargo run --release --bin sync` will use the default `esp_hal::i2s` non-async API
