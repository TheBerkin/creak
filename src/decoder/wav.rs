use super::*;
use hound::{WavReader, WavSpec};

pub struct WavDecoder<R> {
    reader: WavReader<R>,
    spec: WavSpec,
}

impl<R: Read> WavDecoder<R> {
    #[inline]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<WavDecoder<File>, DecoderError> {
        let f = File::open(path).map_err(|err| DecoderError::IOError(err))?;
        WavDecoder::from_reader(f)
    }

    #[inline]
    pub fn try_decode(reader: &mut R) -> Result<bool, DecoderError> {
        Ok(match WavDecoder::from_reader(reader) {
            Ok(_) => true,
            Err(DecoderError::FormatError(_)) => false,
            Err(e) => return Err(e),
        })
    }

    #[inline]
    pub fn from_reader(reader: R) -> Result<Self, DecoderError> {
        let reader = WavReader::new(reader).map_err(hound_err_to_decoder_err)?;
        Ok(Self {
            spec: reader.spec(),
            reader,
        })
    }

    #[inline]
    pub fn info(&self) -> AudioInfo {
        let spec = self.spec;
        AudioInfo {
            format: AudioFormat::Wav,
            sample_rate: spec.sample_rate,
            channels: spec.channels as usize,
        }
    }
}

impl<'reader, R: 'reader + Read> WavDecoder<R> {
    pub fn into_samples(
        self,
    ) -> Result<Box<dyn 'reader + Iterator<Item = Result<crate::Sample, DecoderError>>>, DecoderError>
    {
        let spec = self.spec;
        Ok(match (spec.bits_per_sample, spec.sample_format) {
            (8, hound::SampleFormat::Int) => {
                let iter = self.reader.into_samples::<i8>().map(|sample| {
                    sample
                        .map(|sample| sample as f32 / i8::MAX as f32)
                        .map_err(hound_err_to_decoder_err)
                });
                Box::new(iter)
            }
            (16, hound::SampleFormat::Int) => {
                let iter = self.reader.into_samples::<i16>().map(|sample| {
                    sample
                        .map(|sample| sample as f32 / i16::MAX as f32)
                        .map_err(hound_err_to_decoder_err)
                });
                Box::new(iter)
            }
            (24, hound::SampleFormat::Int) => {
                const MAX_I24: i32 = 0x7fffff;
                let iter = self.reader.into_samples::<i32>().map(|sample| {
                    sample
                        .map(|sample| sample as f32 / MAX_I24 as f32)
                        .map_err(hound_err_to_decoder_err)
                });
                Box::new(iter)
            }
            (32, hound::SampleFormat::Int) => {
                let iter = self.reader.into_samples::<i32>().map(|sample| {
                    sample
                        .map(|sample| sample as f32 / i32::MAX as f32)
                        .map_err(hound_err_to_decoder_err)
                });
                Box::new(iter)
            }
            (32, hound::SampleFormat::Float) => {
                let iter = self
                    .reader
                    .into_samples::<f32>()
                    .map(|sample| sample.map_err(hound_err_to_decoder_err));
                Box::new(iter)
            }
            (other_bps, other_format) => {
                return Err(DecoderError::FormatError(format!(
                    "wav: format '{}-bit {:?}' is not supported",
                    other_bps, other_format
                )))
            }
        })
    }
}

fn hound_err_to_decoder_err(error: hound::Error) -> DecoderError {
    match error {
        hound::Error::IoError(ioerr) => DecoderError::IOError(ioerr),
        hound::Error::FormatError(fmterr) => DecoderError::FormatError(format!("wav: {}", fmterr)),
        hound::Error::Unsupported => {
            DecoderError::FormatError("wav: unsupported format".to_owned())
        }
        hound::Error::InvalidSampleFormat => {
            DecoderError::FormatError("wav: invalid sample format".to_owned())
        }
        hound::Error::TooWide => DecoderError::FormatError(
            "wav: decoded samples are too wide for destination type".to_owned(),
        ),
        _ => unreachable!(),
    }
}
