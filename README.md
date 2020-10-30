# Creak

Creak is a barebones, unified interface for decoding audio of various formats into 32-bit float samples.

## Supported formats

All listed formats will eventually be implemented.

|Format |Feature  |Backend                                     |Status|
|-------|---------|--------------------------------------------|:----:|
|WAV    |`wav`    |[hound](https://crates.io/crates/hound)     |✅
|MP3    |`mp3`    |[minimp3](https://crates.io/crates/minimp3) |🛠
|Vorbis |`vorbis` |[lewton](https://crates.io/crates/lewton)   |🛠
|FLAC   |`flac`   |[claxon](https://crates.io/crates/claxon)   |🛠
