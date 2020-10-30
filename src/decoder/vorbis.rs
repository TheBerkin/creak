use std::{fs::File, io::Read, path::Path, io::Seek};

use lewton::inside_ogg::OggStreamReader;

use crate::DecoderError;

pub struct VorbisDecoder {
    reader: OggStreamReader<File>,
    channels: u32,
    sample_rate: u32,
}

impl VorbisDecoder {
    #[inline]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, DecoderError> {
        let f = File::open(path).map_err(|err| DecoderError::IOError(err))?;
        let reader = match OggStreamReader::new(f) {
            Ok(reader) => reader,
            Err(err) => {
                return Err(vorbis_err_to_decoder_err(err))
            }
        };

        Ok(Self {
            channels: reader.ident_hdr.audio_channels as u32,
            sample_rate: reader.ident_hdr.audio_sample_rate,
            reader,
        })
    }

    #[inline]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[inline]
    pub fn channels(&self) -> usize {
        self.channels as _
    }

    #[inline]
    pub fn into_samples(mut self) -> Result<Box<dyn Iterator<Item = Result<crate::Sample, DecoderError>>>, DecoderError> {
        Ok(Box::new(OggSampleIterator {
            cur_packet: self.reader.read_dec_packet_itl().map_err(vorbis_err_to_decoder_err)?,
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
            Err(err) => return Err(vorbis_err_to_decoder_err(err))
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
                            return Some(Err(err))
                        }
                    }

                    // Convert the sample and return it
                    let sample_float = sample as f32 / i16::MAX as f32;
                    return Some(Ok(sample_float))
                },
                None => {                        
                    if let Err(err) = self.next_packet() {
                        return Some(Err(err))
                    }
                    continue
                }
            }
        }
        None
    }
}

fn vorbis_err_to_decoder_err(error: lewton::VorbisError) -> DecoderError {
    match error {
        lewton::VorbisError::BadAudio(err) => DecoderError::FormatError(format!("ogg: bad audio: {}", err)),
        lewton::VorbisError::BadHeader(err) => DecoderError::FormatError(format!("ogg: bad header: {}", err)),
        lewton::VorbisError::OggError(err) => DecoderError::FormatError(format!("ogg: {}", err)),
    }
}