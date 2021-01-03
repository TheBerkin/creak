use super::*;
use minimp3::{Decoder as Mp3Reader, Error as Mp3Error, Frame};

pub struct Mp3Decoder<R> {
    reader: Mp3Reader<R>,
    first_frame: Frame,
    sample_rate: u32,
    channels: usize,
}

impl<R: Read> Mp3Decoder<R> {
    #[inline]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Mp3Decoder<File>, DecoderError> {
        let f = File::open(path).map_err(|err| DecoderError::IOError(err))?;
        Mp3Decoder::from_reader(f)
    }

    #[inline]
    pub fn try_decode(reader: &mut R) -> Result<bool, DecoderError> {
        Ok(match Mp3Decoder::from_reader(reader) {
            Ok(_) => true,
            Err(DecoderError::FormatError(_)) => false,
            Err(e) => return Err(e),
        })
    }

    #[inline]
    pub fn from_reader(reader: R) -> Result<Self, DecoderError> {
        let mut reader = Mp3Reader::new(reader);
        let first_frame = loop {
            match reader.next_frame() {
                Ok(frame) => break frame,
                Err(Mp3Error::SkippedData) => continue,
                Err(Mp3Error::Eof) => {
                    return Err(DecoderError::FormatError("mp3: no audio data".to_owned()))
                }
                Err(other) => return Err(mp3_err_to_decoder_err(other)),
            }
        };

        Ok(Self {
            sample_rate: first_frame.sample_rate as _,
            channels: first_frame.channels as _,
            first_frame,
            reader,
        })
    }
}

impl<'reader, R: 'reader + Read> Mp3Decoder<R> {
    #[inline]
    pub fn info(&self) -> AudioInfo {
        AudioInfo {
            format: AudioFormat::Mp3,
            sample_rate: self.sample_rate,
            channels: self.channels,
        }
    }

    #[inline]
    pub fn into_samples(
        self,
    ) -> Result<Box<dyn 'reader + Iterator<Item = Result<crate::Sample, DecoderError>>>, DecoderError>
    {
        Ok(Box::new(Mp3SampleIterator {
            expected_channels: self.channels,
            expected_sample_rate: self.sample_rate,
            cur_frame: self.first_frame,
            frame_cursor: 0,
            reader: self.reader,
        }))
    }
}

struct Mp3SampleIterator<R: Read> {
    reader: Mp3Reader<R>,
    expected_channels: usize,
    expected_sample_rate: u32,
    cur_frame: Frame,
    frame_cursor: usize,
}

impl<R: Read> Iterator for Mp3SampleIterator<R> {
    type Item = Result<crate::Sample, DecoderError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Read next frame in if current frame is exhausted
        if self.frame_cursor >= self.cur_frame.data.len() {
            self.frame_cursor = 0;
            self.cur_frame = loop {
                match self.reader.next_frame() {
                    Ok(frame) => {
                        // Skip empty frames
                        if frame.data.len() == 0 {
                            continue;
                        }
                        // Make sure the sample rates match
                        if frame.sample_rate as u32 != self.expected_sample_rate {
                            return Some(Err(DecoderError::FormatError(
                                "mp3: streams with variable sample rates are not supported"
                                    .to_owned(),
                            )));
                        }
                        // Make sure the channel counts match
                        if frame.channels != self.expected_channels {
                            return Some(Err(DecoderError::FormatError(
                                "mp3: streams with variable channel counts are not supported"
                                    .to_owned(),
                            )));
                        }
                        break frame;
                    }
                    Err(Mp3Error::SkippedData) => continue,
                    Err(Mp3Error::Eof) => return None,
                    Err(other) => return Some(Err(mp3_err_to_decoder_err(other))),
                }
            };
        }

        let sample_float = self.cur_frame.data[self.frame_cursor] as f32 / i16::MAX as f32;
        self.frame_cursor += 1;
        Some(Ok(sample_float))
    }
}

#[inline]
fn mp3_err_to_decoder_err(error: minimp3::Error) -> DecoderError {
    match error {
        minimp3::Error::Io(ioerr) => DecoderError::IOError(ioerr),
        minimp3::Error::InsufficientData => {
            DecoderError::FormatError(format!("mp3: insufficient data"))
        }
        _ => unimplemented!(),
    }
}
