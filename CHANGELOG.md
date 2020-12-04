# Changelog

## 0.3.0

### New
* Added support for reading raw samples via `Decoder::open_raw()`

### Fixes
* Fixed 24-bit WAV samples not being decoded properly

## 0.2.0

### New
* Added `AudioInfo` and `AudioFormat` types.

### Changes
* (Breaking) Replaced `channels()` and `sample_rate()` methods in `Decoder` with `info()` method that returns `AudioInfo`.

## 0.1.0

* Initial release.