//! Reader for the Guild Wars 2 `Gw2.dat` archive.
//!
//! * [`archive`] — AN header, MFT, file-id index and entry reads.
//! * [`decompress`] — the Gw2 Huffman/LZ decompressor and its [`decompress::BitReader`].
//! * [`packfile`] — the `PF` chunk container found inside many decompressed assets.
//! * [`pfwrite`] / [`manifest`] — packfile writer and patcher manifests (offline CDN).
//!
//! All parsing is bounds-checked: malformed input yields an [`std::io::Error`]
//! (usually `InvalidData` / `UnexpectedEof`) and never panics.

pub mod archive;
pub mod animation_sequence;
pub mod appearance;
pub mod content;
pub mod content_name;
pub mod creation_equipment_content;
pub mod decompress;
pub mod manifest;
pub mod mapc;
pub mod packfile;
pub mod pfread;
pub mod pfschema;
pub mod pfwrite;
pub mod profession_content;
pub mod progress_content;
pub mod skill_fact_content;
pub mod skill_palette_content;
pub mod skill_recharge_content;
pub mod setting_content;
pub mod scene_bounds;
pub mod terrain;
pub mod surface;
pub mod water;

#[cfg(test)]
mod synthetic;
