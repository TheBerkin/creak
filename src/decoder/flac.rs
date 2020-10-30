use std::{fs::File, io::Read, path::Path};

use claxon::FlacReader;

use crate::DecoderError;

pub struct FlacDecoder {
    reader: FlacReader<File>,
    sample_rate: u32,
    channels: usize,
}

impl FlacDecoder {
    #[inline]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, DecoderError> {
        let reader = FlacReader::open(path).map_err(flac_err_as_decoder_err)?;
        let (sample_rate, channels) = (reader.streaminfo().sample_rate, reader.streaminfo().channels);
        Ok(Self {
            sample_rate,
            channels: channels as _,
            reader
        })
    }
}

impl FlacDecoder {
    #[inline]
    pub fn channels(&self) -> usize {
        self.channels
    }

    #[inline]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[inline]
    pub fn into_samples(self) -> Result<Box<dyn Iterator<Item = Result<crate::Sample, DecoderError>>>, DecoderError> {
        Ok(Box::new(FlacSampleIterator::new(self.reader)))
    }
}

struct FlacSampleIterator<R: Read> {
    reader: FlacReader<R>,
    cur_block: Vec<i32>,
    cur_block_len: usize,
    max_sample_value: f32,
    block_cursor: usize,
}

impl<'a, R: Read + 'a> FlacSampleIterator<R> {
    fn new(reader: FlacReader<R>) -> Self {
        let info = reader.streaminfo();
        
        Self {
            cur_block: Vec::with_capacity(info.max_block_size as usize * info.channels as usize),
            max_sample_value: (i32::MAX >> (32 - info.bits_per_sample)) as f32,
            reader,
            cur_block_len: 0,
            block_cursor: 0
        }
    }
}

impl<R: Read> Iterator for FlacSampleIterator<R> {
    type Item = Result<crate::Sample, DecoderError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.block_cursor < self.cur_block_len {
                let sample_float = self.cur_block[self.block_cursor] as f32 / self.max_sample_value;
                self.block_cursor += 1;
                return Some(Ok(sample_float));
            }

            self.block_cursor = 0;
            let block_buffer = std::mem::replace(&mut self.cur_block, vec![]);
            match self.reader.blocks().read_next_or_eof(block_buffer) {
                Ok(Some(block)) => {
                    self.cur_block_len = block.len() as _;
                    self.cur_block = block.into_buffer();
                }
                _ => return None,
            }
        }
    }
}

fn flac_err_as_decoder_err(error: claxon::Error) -> DecoderError {
    match error {
        claxon::Error::IoError(ioerr) => DecoderError::IOError(ioerr),
        claxon::Error::FormatError(fmterr) => DecoderError::FormatError(format!("flac: format error: {}", fmterr)),
        claxon::Error::Unsupported(what) => DecoderError::FormatError(format!("flac: unsupported: {}", what)),
    }
}