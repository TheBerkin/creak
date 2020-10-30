use std::{fmt::Display, io, path::Path};

use std::error::Error;

#[cfg(feature = "wav")] mod wav;
#[cfg(feature = "vorbis")] mod vorbis;
#[cfg(feature = "mp3")] mod mp3;
#[cfg(feature = "flac")] mod flac;

/// The type of decoded audio samples.
pub type Sample = f32;

/// An audio decoder.
///
/// Use `Decoder::open` to open an audio file and read samples.
pub struct Decoder {
    decoder: FormatDecoder
}

/// Information about an opened audio file.
#[derive(Debug, Clone)]
pub struct AudioInfo {
    sample_rate: u32,
    channels: usize,
    format: AudioFormat,
}

impl AudioInfo {
    /// Gets the sample rate of the audio.
    #[inline]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Gets the number of channels in the audio.
    #[inline]
    pub fn channels(&self) -> usize {
        self.channels
    }

    /// Gets the original format of the audio.
    #[inline] 
    pub fn format(&self) -> AudioFormat {
        self.format
    }
}

/// Indicates the format of an audio stream.
#[derive(Debug, Copy, Clone)]
pub enum AudioFormat {
    Wav,
    Vorbis,
    Mp3,
    Flac,
}

impl Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioFormat::Wav => write!(f, "WAV"),
            AudioFormat::Vorbis => write!(f, "Vorbis"),
            AudioFormat::Mp3 => write!(f, "MP3"),
            AudioFormat::Flac => write!(f, "FLAC"),
        }
    }
}

impl Decoder {
    /// Attempts to open the specified audio file for decoding.
    ///
    /// Creak uses the file's extension to determine what kind of format it is.
    /// The currently recognized extensions are:
    ///
    /// * **.wav** - WAV.
    /// * **.ogg** - Ogg Vorbis.
    /// * **.mp3** - MP3.
    /// * **.flac** - FLAC.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, DecoderError> {
        Ok(Self {
            decoder: FormatDecoder::open(path)?
        })
    }
}

impl Decoder {
    /// Gets information about the audio, such as channel count and sample rate.
    #[inline]
    pub fn info(&self) -> AudioInfo {
        self.decoder.info()
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
    #[cfg(feature = "mp3")]
    Mp3(self::mp3::Mp3Decoder),
    #[cfg(feature = "flac")]
    Flac(self::flac::FlacDecoder),
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
                "ogg" => requires "vorbis" for FormatDecoder::Vorbis(self::vorbis::VorbisDecoder::open(path)?),
                "mp3" => requires "mp3" for FormatDecoder::Mp3(self::mp3::Mp3Decoder::open(path)?),
                "flac" => requires "flac" for FormatDecoder::Flac(self::flac::FlacDecoder::open(path)?)
            )
        }
        Err(DecoderError::NoExtension)
    }

    #[inline]
    pub fn into_samples(self) -> Result<SampleIterator, DecoderError> {
        match self {
            #[cfg(feature = "wav")]
            FormatDecoder::Wav(d) => Ok(SampleIterator(d.into_samples()?)),
            #[cfg(feature = "vorbis")]
            FormatDecoder::Vorbis(d) => Ok(SampleIterator(d.into_samples()?)),
            #[cfg(feature = "mp3")]
            FormatDecoder::Mp3(d) => Ok(SampleIterator(d.into_samples()?)),
            #[cfg(feature = "flac")]
            FormatDecoder::Flac(d) => Ok(SampleIterator(d.into_samples()?)),
        }
    }

    #[inline]
    pub fn info(&self) -> AudioInfo {
        match self {
            #[cfg(feature = "wav")]
            FormatDecoder::Wav(d) => d.info(),
            #[cfg(feature = "vorbis")]
            FormatDecoder::Vorbis(d) => d.info(),
            #[cfg(feature = "mp3")]
            FormatDecoder::Mp3(d) => d.info(),
            #[cfg(feature = "flac")]
            FormatDecoder::Flac(d) => d.info(),
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

impl Error for DecoderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
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