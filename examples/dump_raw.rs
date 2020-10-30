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
    let info = decoder.info();
    eprintln!("Format: {}; Channels: {}; Sample Rate: {}Hz", 
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