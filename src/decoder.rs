use std::{fmt::Display, fs::File, io::{self, BufReader}, path::Path};

use std::error::Error;

use self::raw::RawDecoder;

mod raw;
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

/// Specification decsribing how to decode some raw audio samples.
#[derive(Debug, Clone)]
pub struct RawAudioSpec {
    /// The sample rate of the audio.
    pub sample_rate: u32,
    /// The numbers of channels in the audio.
    pub channels: usize,
    /// The format of the sample data.
    pub sample_format: RawSampleFormat,
    /// The endianness of the samples.
    pub endianness: Endian,
    /// The byte offset at which to start reading samples.
    pub start_offset: usize,
    /// The maximum number of frames to read.
    pub max_frames: Option<usize>,
}

/// Represents endianness.
#[derive(Debug, Copy, Clone)]
pub enum Endian {
    /// Big Endian.
    Big,
    /// Little Endian.
    Little,
}

/// Represents supported sample formats for raw audio decoding.
#[derive(Debug, Copy, Clone)]
pub enum RawSampleFormat {
    /// 32-bit IEEE floating-point sample format.
    Float32,
    /// 64-bit IEEE floating-point sample format.
    Float64,
    /// Unsignbed 8-bit integer sample format.
    Unsigned8,
    /// Signed 8-bit integer sample format.
    Signed8,
    /// Unsigned 16-bit integer sample format.
    Unsigned16,
    /// Signed 16-bit integer sample format.
    Signed16,
    /// Unsigned 24-bit integer sample format.
    Unsigned24,
    /// Signed 24-bit integer sample format.
    Signed24,
    /// Unsigned 32-bit integer sample format.
    Unsigned32,
    /// Signed 32-bit integer sample format.
    Signed32,
    /// Unsigned 64-bit integer sample format.
    Unsigned64,
    /// signed 64-bit integer sample format.
    Signed64
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
    /// WAV format.
    Wav,
    /// Ogg Vorbis format.
    Vorbis,
    /// MPEG Layer 3 format.
    Mp3,
    /// FLAC format.
    Flac,
    /// Raw audio samples.
    Raw,
}

impl Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioFormat::Wav => write!(f, "WAV"),
            AudioFormat::Vorbis => write!(f, "Vorbis"),
            AudioFormat::Mp3 => write!(f, "MP3"),
            AudioFormat::Flac => write!(f, "FLAC"),
            AudioFormat::Raw => write!(f, "Raw"),
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
    #[inline]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, DecoderError> {
        Ok(Self {
            decoder: FormatDecoder::open(path)?
        })
    }

    /// Attempts to open the specified audio file for raw sample decoding.
    ///
    /// The format of the source samples is determined from the `RawAudioSpec` passed to the function.
    #[inline]
    pub fn open_raw<P: AsRef<Path>>(path: P, spec: RawAudioSpec) -> Result<Self, DecoderError> {
        let f = File::open(path).map_err(|err| DecoderError::IOError(err))?;
        Ok(Self {
            decoder: FormatDecoder::Raw(RawDecoder::new(BufReader::new(f), spec)?)
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
    Raw(self::raw::RawDecoder<BufReader<File>>),
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
            FormatDecoder::Raw(d) => Ok(SampleIterator(d.into_samples()?)),
            #[cfg(feature = "wav")]
            FormatDecoder::Wav(d) => Ok(SampleIterator(d.into_samples()?)),
            #[cfg(feature = "vorbis")]
            FormatDecoder::Vorbis(d) => Ok(SampleIterator(d.into_samples()?)),
            #[cfg(feature = "mp3")]
            FormatDecoder::Mp3(d) => Ok(SampleIterator(d.into_samples()?)),
            #[cfg(feature = "flac")]
            FormatDecoder::Flac(d) => Ok(SampleIterator(d.into_samples()?))
        }
    }

    #[inline]
    pub fn info(&self) -> AudioInfo {
        match self {
            FormatDecoder::Raw(d) => d.info(),
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
    /// The decoder could not read a complete frame or sample, possibly due to an EOF.
    IncompleteData,
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
            DecoderError::IncompleteData => write!(f, "incomplete data"),
        }
    }
}