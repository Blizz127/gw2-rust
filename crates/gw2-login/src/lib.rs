//! GW2 portal / login server pieces.
//!
//! **UNVERIFIED.** Everything here comes from public descriptions of the NCsoft
//! STS protocol and the 2012–13 GW2 notes (`docs/re/public-protocol-notes.md`).
//! None of it is REA-confirmed for our client build. The observer
//! (`gw2-login-observe`) exists to capture what the real client actually sends.

pub mod assetcdn;
pub mod dns;
pub mod sts;
pub mod tls;
