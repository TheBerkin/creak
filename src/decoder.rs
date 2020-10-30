use std::{fmt::Display, io, path::Path};

#[cfg(feature = "wav")] mod wav;
#[cfg(feature = "vorbis")] mod vorbis;

pub type Sample = f32;

/// An audio decoder.
///
/// Use `Decoder::open` to open an audio file and read samples.
pub struct Decoder {
    decoder: FormatDecoder
}

impl Decoder {
    /// Attempts to open the specified audio file for decoding.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, DecoderError> {
        Ok(Self {
            decoder: FormatDecoder::open(path)?
        })
    }
}

impl Decoder {
    /// Gets the sample rate of the audio.
    #[inline]
    pub fn sample_rate(&self) -> u32 {
        self.decoder.sample_rate()
    }

    /// Gets the channel count of the audio.
    #[inline]
    pub fn channels(&self) -> u32 {
        self.decoder.channels()
    }

    /// Consumes the `Decoder` and returns an iterator over the samples.
    /// Channels are interleaved.
    #[inline]
    pub fn into_samples(self) -> Result<SampleIterator, DecoderError> {
        self.decoder.into_samples()
    }
}

/// Iterates over decoded audio samples. Channels are interleaved.
pub struct SampleIterator(Box<dyn Iterator<Item = Result<Sample, DecoderError>>>);

impl Iterator for SampleIterator {
    type Item = Result<Sample, DecoderError>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub(crate) enum FormatDecoder {
    #[cfg(feature = "wav")]
    Wav(self::wav::WavDecoder),
    #[cfg(feature = "vorbis")]
    Vorbis(self::vorbis::VorbisDecoder),
}

impl FormatDecoder {
    #[inline]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, DecoderError> {
        macro_rules! get_decoder {
            ($in_ext:expr, $($ext:literal => requires $feature:literal for $init:expr),*) => {
                match $in_ext {
                    $(
                        #[cfg(feature = $feature)]
                        $ext => { return Ok($init) }
                        #[cfg(not(feature = $feature))]
                        $ext => { return Err(DecoderError::DisabledExtension { feature: $feature, extension: $ext }) }
                    )*
                    other => return Err(DecoderError::UnsupportedExtension(other.to_owned()))
                }
            }
        }

        // Check the file extension to see which backend to use
        if let Some(ext) = path.as_ref().extension().and_then(|ext| ext.to_str()) {
            get_decoder!(ext,
                "wav" => requires "wav" for FormatDecoder::Wav(self::wav::WavDecoder::open(path)?),
                "ogg" => requires "vorbis" for FormatDecoder::Vorbis(self::vorbis::VorbisDecoder::open(path)?)
            )
        }
        Err(DecoderError::NoExtension)
    }

    #[inline]
    pub fn into_samples(self) -> Result<SampleIterator, DecoderError> {
        match self {
            #[cfg(feature = "wav")]
            FormatDecoder::Wav(decoder) => Ok(SampleIterator(decoder.into_samples()?)),
            #[cfg(feature = "vorbis")]
            FormatDecoder::Vorbis(decoder) => Ok(SampleIterator(decoder.into_samples()?)),
            _ => unreachable!()
        }
    }

    #[inline]
    pub fn sample_rate(&self) -> u32 {
        match self {
            #[cfg(feature = "wav")]
            FormatDecoder::Wav(d) => d.sample_rate(),
            #[cfg(feature = "vorbis")]
            FormatDecoder::Vorbis(d) => d.sample_rate(),
            _ => unreachable!()
        }
    }

    #[inline]
    pub fn channels(&self) -> u32 {
        match self {
            #[cfg(feature = "wav")]
            FormatDecoder::Wav(d) => d.channels(),
            #[cfg(feature = "vorbis")]
            FormatDecoder::Vorbis(d) => d.channels(),
            _ => unreachable!()
        }
    }
}

/// An error encountered while decoding an audio file.
#[derive(Debug)]
pub enum DecoderError {
    /// I/O error.
    IOError(io::Error),
    /// Error specific to the audio format.
    FormatError(String),
    /// No extension was provided on the input file.
    NoExtension,
    /// The extension on the input file is not supported for decoding.
    UnsupportedExtension(String),
    /// The extension on the input file requires a feature that is not enabled.
    DisabledExtension {
        extension: &'static str,
        feature: &'static str,
    },
}

impl std::error::Error for DecoderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl Display for DecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecoderError::IOError(err) => write!(f, "IO error: {}", err),
            DecoderError::FormatError(err) => write!(f, "format error: {}", err),
            DecoderError::NoExtension => write!(f, "file has no extension"),
            DecoderError::UnsupportedExtension(ext) => write!(f, "extension '{}' is not supported", ext),
            DecoderError::DisabledExtension { extension, feature } => write!(f, "feature '{}' is required to read '{}' files, but is not enabled", feature, extension),
        }
    }
}