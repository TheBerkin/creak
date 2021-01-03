// Simple program that reads an audio file and dumps its samples in 32-bit float to stdout

use creak;
use std::{
    env,
    fs::File,
    io::{self, Write},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get a file name from the cmdline args
    let file_name = match env::args().nth(1) {
        Some(arg) => arg,
        None => {
            eprintln!("No audio file specified!");
            return Ok(());
        }
    };

    let reader = File::open(file_name)?;

    // Open an audio file of any supported format with one function call
    let decoder = creak::Decoder::from_reader(reader)?;

    // Print basic audio info to stderr
    let info = decoder.info();
    eprintln!(
        "Format: {}; Channels: {}; Sample Rate: {}Hz",
        info.format(),
        info.channels(),
        info.sample_rate()
    );

    let mut stdout = io::stdout();
    let mut num_samples: usize = 0;

    // Dump all samples to stdout
    for sample in decoder.into_samples()? {
        stdout.write(&sample?.to_le_bytes())?;
        num_samples += 1;
    }

    eprintln!("{} samples(s) read.", num_samples);
    Ok(())
}
