use super::*;
use lewton::inside_ogg::OggStreamReader;

pub struct VorbisDecoder<R: Read + Seek> {
    reader: OggStreamReader<R>,
    channels: usize,
    sample_rate: u32,
}

impl<R: Read + Seek> VorbisDecoder<R> {
    #[inline]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<VorbisDecoder<File>, DecoderError> {
        let f = File::open(path).map_err(|err| DecoderError::IOError(err))?;
        VorbisDecoder::from_reader(f)
    }

    #[inline]
    pub fn try_decode(reader: &mut R) -> Result<bool, DecoderError> {
        Ok(match VorbisDecoder::from_reader(reader) {
            Ok(_) => true,
            Err(DecoderError::FormatError(_)) => false,
            Err(e) => return Err(e),
        })
    }

    #[inline]
    pub fn from_reader(reader: R) -> Result<Self, DecoderError> {
        let reader = match OggStreamReader::new(reader) {
            Ok(reader) => reader,
            Err(err) => return Err(vorbis_err_to_decoder_err(err)),
        };

        Ok(Self {
            channels: reader.ident_hdr.audio_channels as usize,
            sample_rate: reader.ident_hdr.audio_sample_rate,
            reader,
        })
    }

    #[inline]
    pub fn info(&self) -> AudioInfo {
        AudioInfo {
            format: AudioFormat::Vorbis,
            sample_rate: self.sample_rate,
            channels: self.channels,
        }
    }
}

impl<'reader, R: 'reader + Read + Seek> VorbisDecoder<R> {
    #[inline]
    pub fn into_samples(
        mut self,
    ) -> Result<Box<dyn 'reader + Iterator<Item = Result<crate::Sample, DecoderError>>>, DecoderError>
    {
        Ok(Box::new(OggSampleIterator {
            cur_packet: self
                .reader
                .read_dec_packet_itl()
                .map_err(vorbis_err_to_decoder_err)?,
            reader: self.reader,
            packet_cursor: 0,
        }))
    }
}

struct OggSampleIterator<T: Read + Seek> {
    reader: OggStreamReader<T>,
    cur_packet: Option<Vec<i16>>,
    packet_cursor: usize,
}

impl<T: Read + Seek> OggSampleIterator<T> {
    #[inline(always)]
    fn next_packet(&mut self) -> Result<(), DecoderError> {
        self.packet_cursor = 0;
        self.cur_packet = match self.reader.read_dec_packet_itl() {
            Ok(packet) => packet,
            Err(err) => return Err(vorbis_err_to_decoder_err(err)),
        };
        Ok(())
    }
}

impl<T: Read + Seek> Iterator for OggSampleIterator<T> {
    type Item = Result<crate::Sample, DecoderError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(packet) = self.cur_packet.as_ref() {
            match packet.get(self.packet_cursor) {
                Some(sample) => {
                    // Increment the cursor
                    self.packet_cursor += 1;

                    let sample = *sample;

                    // Get the next packet if done reading this one
                    if self.packet_cursor >= packet.len() {
                        if let Err(err) = self.next_packet() {
                            return Some(Err(err));
                        }
                    }

                    // Convert the sample and return it
                    let sample_float = sample as f32 / i16::MAX as f32;
                    return Some(Ok(sample_float));
                }
                None => {
                    if let Err(err) = self.next_packet() {
                        return Some(Err(err));
                    }
                    continue;
                }
            }
        }
        None
    }
}

fn vorbis_err_to_decoder_err(error: lewton::VorbisError) -> DecoderError {
    match error {
        lewton::VorbisError::BadAudio(err) => {
            DecoderError::FormatError(format!("ogg: bad audio: {}", err))
        }
        lewton::VorbisError::BadHeader(err) => {
            DecoderError::FormatError(format!("ogg: bad header: {}", err))
        }
        lewton::VorbisError::OggError(err) => DecoderError::FormatError(format!("ogg: {}", err)),
    }
}
