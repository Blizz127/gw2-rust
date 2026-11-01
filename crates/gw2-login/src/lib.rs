//! GW2 portal / login server pieces.
//!
//! The offline build-209026 observer has exercised patch manifests, StartTls,
//! TLS and explicit synthetic portal replies; see `docs/re/209026-login-tls.md`
//! and the subsequent portal-message notes. This remains an observation harness,
//! without credential validation or a persistent account service. Older protocol
//! assumptions still require verification against this frozen client.

pub mod assetcdn;
pub mod dns;
pub mod sts;
pub mod tls;
