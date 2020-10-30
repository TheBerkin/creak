# Creak

[![Crates.io](https://img.shields.io/crates/v/creak)](https://crates.io/crates/creak)

Creak is a barebones, unified interface for decoding audio of various formats into 32-bit float samples.

## Supported formats

|Format |Feature  |Backend                                     |Status|
|-------|---------|--------------------------------------------|:----:|
|WAV    |`wav`    |[hound](https://crates.io/crates/hound)     |✅
|Vorbis |`vorbis` |[lewton](https://crates.io/crates/lewton)   |✅
|MP3    |`mp3`    |[minimp3](https://crates.io/crates/minimp3) |✅
|FLAC   |`flac`   |[claxon](https://crates.io/crates/claxon)   |✅

(✅ = Implemented; 🛠 = WIP)

## Example

```rust
// Simple program that reads an audio file and dumps its samples in 32-bit float to stdout

use std::{env, io};
use std::io::Write;
use creak;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    // Get a file name from the cmdline args
    let file_name = match args.first() {
        Some(arg) => arg,
        None => {
            eprintln!("no audio file specified!");
            return Ok(())
        }
    };

    // Open an audio file of any supported format with one function call
    let decoder = creak::Decoder::open(&file_name)?;

    // Print basic audio info
    eprintln!("Channels: {}, {}Hz", decoder.channels(), decoder.sample_rate());

    let mut stdout = io::stdout();
    let mut num_samples: usize = 0;

    // Dump all samples to stdout
    for sample in decoder.into_samples()? {
        stdout.write(&sample?.to_le_bytes())?;
        num_samples += 1;
    }

    eprintln!("Samples read: {}", num_samples);

    Ok(())
}
```

## License

Licensed under either of

* Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.