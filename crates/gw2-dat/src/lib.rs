//! Reader for the Guild Wars 2 `Gw2.dat` archive.
//!
//! * [`archive`] — AN header, MFT, file-id index and entry reads.
//! * [`decompress`] — the Gw2 Huffman/LZ decompressor and its [`decompress::BitReader`].
//! * [`packfile`] — the `PF` chunk container found inside many decompressed assets.
//!
//! All parsing is bounds-checked: malformed input yields an [`std::io::Error`]
//! (usually `InvalidData` / `UnexpectedEof`) and never panics.

pub mod archive;
pub mod decompress;
pub mod packfile;
pub mod pfread;
pub mod pfschema;

#[cfg(test)]
mod synthetic;
