//! # Creak
//!
//! Creak is a simple library for decoding popular audio formats into raw `f32` samples.
//!
//! ## Usage
//! 
//! The `Decoder` struct handles opening, parsing, and decoding audio files. Use `Decoder::open()` to open a file.
//! When you want to read samples, call `Decoder.into_samples()` to get an iterator over the samples.
//!
//! The audio file is closed when the `Decoder` or `SampleIterator` is dropped.
//! 
//! Currently supported formats are WAV, Ogg Vorbis, MP3, and FLAC.

#![allow(dead_code)]

mod decoder;

pub use decoder::*;