use super::*;

pub struct RawDecoder<R: Read + Seek> {
    reader: R,
    spec: RawAudioSpec,
    info: AudioInfo,
}

impl<R: Read + Seek> RawDecoder<R> {
    pub fn new(mut reader: R, spec: RawAudioSpec) -> Result<Self, DecoderError> {
        // Attempt to seek to requested starting position
        if let Err(err) = reader.seek(SeekFrom::Start(spec.start_offset as _)) {
            return Err(DecoderError::IOError(err));
        }

        let info = AudioInfo {
            channels: spec.channels,
            format: AudioFormat::Raw,
            sample_rate: spec.sample_rate,
        };

        Ok(Self { reader, spec, info })
    }
}

impl<R: Read + Seek> RawDecoder<R> {
    #[inline]
    pub fn info(&self) -> AudioInfo {
        self.info.clone()
    }

    #[inline]
    pub fn into_samples<'a>(
        self,
    ) -> Result<Box<dyn 'a + Iterator<Item = Result<crate::Sample, DecoderError>>>, DecoderError>
    where
        R: 'a,
    {
        let endian = self.spec.endianness;

        macro_rules! sample_iterator {
            ($func:expr) => {
                Box::new(RawSampleIterator {
                    reader: self.reader,
                    read_func: $func,
                })
            };
            (unsigned $sample_type:ty) => {
                Box::new(RawSampleIterator {
                    reader: self.reader,
                    read_func: move |reader: &mut R| {
                        const SIZE_BYTES: usize = std::mem::size_of::<$sample_type>();
                        const MAX_VAL: f32 = <$sample_type>::MAX as f32; //(1 << $sample_bits) + ((1 << $sample_bits) - 1);
                        let mut buf = [0; SIZE_BYTES];
                        match reader.read(&mut buf) {
                            Ok(0) => None,
                            Ok(SIZE_BYTES) => Some(Ok(match endian {
                                Endian::Big => <$sample_type>::from_be_bytes(buf) as f32 / MAX_VAL * 2.0 - 1.0,
                                Endian::Little => <$sample_type>::from_le_bytes(buf) as f32 / MAX_VAL * 2.0 - 1.0,
                            })),
                            Ok(_) => Some(Err(DecoderError::IncompleteData)),
                            Err(err) => Some(Err(DecoderError::IOError(err))),
                        }
                    }
                })
            };
            (signed $sample_type:ty) => {
                Box::new(RawSampleIterator {
                    reader: self.reader,
                    read_func: move |reader: &mut R| {
                        const SIZE_BYTES: usize = std::mem::size_of::<$sample_type>();
                        const MAX_VAL: f32 = <$sample_type>::MAX as f32; //(1 << $sample_bits) + ((1 << $sample_bits) - 1);
                        let mut buf = [0; SIZE_BYTES];
                        match reader.read(&mut buf) {
                            Ok(0) => None,
                            Ok(SIZE_BYTES) => Some(Ok(match endian {
                                Endian::Big => <$sample_type>::from_be_bytes(buf) as f32 / MAX_VAL,
                                Endian::Little => <$sample_type>::from_le_bytes(buf) as f32 / MAX_VAL,
                            })),
                            Ok(_) => Some(Err(DecoderError::IncompleteData)),
                            Err(err) => Some(Err(DecoderError::IOError(err))),
                        }
                    }
                })
            };
        }
        Ok(match self.spec.sample_format {
            RawSampleFormat::Float32 => sample_iterator!(move |reader: &mut R| {
                const SIZEOF_FLOAT: usize = 4;
                let mut buf = [0; SIZEOF_FLOAT];
                match reader.read(&mut buf) {
                    Ok(0) => None,
                    Ok(SIZEOF_FLOAT) => Some(Ok(match endian {
                        Endian::Big => f32::from_be_bytes(buf),
                        Endian::Little => f32::from_le_bytes(buf),
                    })),
                    Ok(_) => Some(Err(DecoderError::IncompleteData)),
                    Err(err) => Some(Err(DecoderError::IOError(err))),
                }
            }),
            RawSampleFormat::Float64 => sample_iterator!(move |reader: &mut R| {
                const SIZEOF_DOUBLE: usize = 8;
                let mut buf = [0; SIZEOF_DOUBLE];
                match reader.read(&mut buf) {
                    Ok(0) => None,
                    Ok(SIZEOF_DOUBLE) => Some(Ok(match endian {
                        Endian::Big => f64::from_be_bytes(buf) as f32,
                        Endian::Little => f64::from_le_bytes(buf) as f32,
                    })),
                    Ok(_) => Some(Err(DecoderError::IncompleteData)),
                    Err(err) => Some(Err(DecoderError::IOError(err))),
                }
            }),
            RawSampleFormat::Unsigned24 => sample_iterator!(move |reader: &mut R| {
                const MAX_U24: usize = (1 << 24) + ((1 << 24) - 1);
                const SIZE: usize = 3;
                const BUFFER_SIZE: usize = 5;
                let mut buf = [0; BUFFER_SIZE];
                match reader.read(&mut buf[1..SIZE]) {
                    Ok(0) => None,
                    Ok(SIZE) => Some(Ok(match endian {
                        // I am so sorry.
                        Endian::Big => {
                            u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as f32
                                / MAX_U24 as f32
                                * 2.0
                                - 1.0
                        }
                        Endian::Little => {
                            u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]) as f32
                                / MAX_U24 as f32
                                * 2.0
                                - 1.0
                        }
                    })),
                    Ok(_) => Some(Err(DecoderError::IncompleteData)),
                    Err(err) => Some(Err(DecoderError::IOError(err))),
                }
            }),
            RawSampleFormat::Signed24 => sample_iterator!(move |reader: &mut R| {
                const MAX_I24: usize = (1 << 23) + ((1 << 23) - 1);
                const SIZE: usize = 3;
                const BUFFER_SIZE: usize = 5;
                let mut buf = [0; BUFFER_SIZE];
                match reader.read(&mut buf[1..SIZE]) {
                    Ok(0) => None,
                    Ok(SIZE) => Some(Ok(match endian {
                        Endian::Big => {
                            i32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as f32
                                / MAX_I24 as f32
                        }
                        Endian::Little => {
                            i32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]) as f32
                                / MAX_I24 as f32
                        }
                    })),
                    Ok(_) => Some(Err(DecoderError::IncompleteData)),
                    Err(err) => Some(Err(DecoderError::IOError(err))),
                }
            }),
            RawSampleFormat::Unsigned8 => sample_iterator!(unsigned u8),
            RawSampleFormat::Signed8 => sample_iterator!(signed i8),
            RawSampleFormat::Unsigned16 => sample_iterator!(unsigned u16),
            RawSampleFormat::Signed16 => sample_iterator!(signed i16),
            RawSampleFormat::Unsigned32 => sample_iterator!(unsigned u32),
            RawSampleFormat::Signed32 => sample_iterator!(signed i32),
            RawSampleFormat::Unsigned64 => sample_iterator!(unsigned u64),
            RawSampleFormat::Signed64 => sample_iterator!(signed i64),
        })
    }
}

struct RawSampleIterator<
    R: Read + Seek,
    F: Fn(&mut R) -> Option<Result<crate::Sample, DecoderError>>,
> {
    reader: R,
    read_func: F,
}

impl<R: Read + Seek, F: Fn(&mut R) -> Option<Result<crate::Sample, DecoderError>>> Iterator
    for RawSampleIterator<R, F>
{
    type Item = Result<crate::Sample, DecoderError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        (self.read_func)(&mut self.reader)
    }
}

#[inline]
fn io_err_to_decoder_err(error: std::io::Error) -> DecoderError {
    DecoderError::IOError(error)
}
