use std::{io::BufReader, fs::File, path::Path};

use hound::{WavReader, WavSpec};

use crate::DecoderError;

pub struct WavDecoder {
    reader: WavReader<BufReader<File>>,
    spec: WavSpec
}

impl WavDecoder {
    #[inline]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, DecoderError> {
        let reader = WavReader::open(path).map_err(hound_err_to_decoder_err)?;
        Ok(Self {
            spec: reader.spec(),
            reader
        })
    }

    #[inline]
    pub fn sample_rate(&self) -> u32 {
        self.spec.sample_rate
    }

    #[inline]
    pub fn channels(&self) -> u32 {
        self.spec.channels as u32
    }

    pub fn into_samples(self) -> Result<Box<dyn Iterator<Item = Result<crate::Sample, DecoderError>>>, DecoderError> {
        let spec = self.spec;
        Ok(match (spec.bits_per_sample, spec.sample_format) {
            (8, hound::SampleFormat::Int) => {
                let iter = self.reader.into_samples::<i8>()
                    .map(|sample| 
                        sample
                        .map(|sample| sample as f32 / i8::MAX as f32)
                        .map_err(hound_err_to_decoder_err)
                    );
                Box::new(iter)
            },
            (16, hound::SampleFormat::Int) => {
                let iter = self.reader.into_samples::<i16>()
                    .map(|sample| 
                        sample
                        .map(|sample| sample as f32 / i16::MAX as f32)
                        .map_err(hound_err_to_decoder_err)
                    );
                Box::new(iter)
            },
            (24, hound::SampleFormat::Int) | (32, hound::SampleFormat::Int) => {
                let iter = self.reader
                    .into_samples::<i32>()
                    .map(|sample| 
                        sample
                        .map(|sample| sample as f32 / i32::MAX as f32)
                        .map_err(hound_err_to_decoder_err)
                    );
                Box::new(iter)
            },
            (32, hound::SampleFormat::Float) => {
                let iter = self.reader
                    .into_samples::<f32>()
                    .map(|sample| sample.map_err(hound_err_to_decoder_err));
                Box::new(iter)
            },
            (other_bps, other_format) => return Err(DecoderError::FormatError(format!("wav: format '{}-bit {:?}' is not supported", other_bps, other_format)))
        })
    }
}

fn hound_err_to_decoder_err(error: hound::Error) -> DecoderError {
    match error {
        hound::Error::IoError(ioerr) => DecoderError::IOError(ioerr),
        hound::Error::FormatError(fmterr) => DecoderError::FormatError(format!("wav: {}", fmterr)),
        hound::Error::Unsupported => DecoderError::FormatError("wav: unsupported format".to_owned()),
        hound::Error::InvalidSampleFormat => DecoderError::FormatError("wav: invalid sample format".to_owned()),
        hound::Error::TooWide => DecoderError::FormatError("wav: decoded samples are too wide for destination type".to_owned()),
        _ => unreachable!()
    }
}