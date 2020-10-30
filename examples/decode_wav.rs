use creak::Decoder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sine_decoder = Decoder::open("./examples/sine_i16_44100.wav")?;
    assert_eq!(sine_decoder.channels(), 1);
    assert_eq!(sine_decoder.sample_rate(), 44100);
    for sample in sine_decoder.into_samples()? {
        println!("sample: {}", sample?);
    }
    Ok(())
}