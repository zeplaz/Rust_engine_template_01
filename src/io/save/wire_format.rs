//! Save wire-format markers — binary bulk remains matrix-owned.
//!
//! **ANIM-03** (gea4v2 13.8.2–13.8.4 · LEGACY `animation_compression` gea_v2_e4 §13.8 p306):
//! book curve quantization / key-omission / curve compression map onto save+telemetry —
//! quantize floats before hash/compare, Identity payload codec until a real compressor lands.
//! No in-engine `AnimationCurve` host; debt: true keyframe curve compression stays deferred.

use std::io;

/// Four-byte artifact envelope tag (`WRST`).
pub const SAVE_ARTIFACT_MAGIC: &[u8; 4] = b"WRST";
pub const SAVE_ARTIFACT_ENVELOPE_VERSION: u8 = 1;

/// ANIM-03 unit-interval quantize scale (matches fire-overlay VT-4 heat hash ×10000).
pub const SAVE_FLOAT_QUANT_SCALE: f32 = 10_000.0;

/// Quantize a unit-interval float to a stable u32 key (gea4v2 13.8.2).
#[must_use]
#[cfg(test)]
fn quantize_unit_interval_f32(v: f32) -> u32 {
    (v.clamp(0.0, 1.0) * SAVE_FLOAT_QUANT_SCALE).round() as u32
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveWireFormat {
    RonTextual,
}

/// Compression codec for chunk artifact bodies (matrix-owned beyond identity).
///
/// ANIM-03: sole active codec is [`SavePayloadCompression::Identity`] — envelope rejects others.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SavePayloadCompression {
    Identity = 0,
}

/// Payload kind inside a chunk artifact envelope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SaveArtifactBodyKind {
    RonChunkTextual = 1,
}

/// Binary bulk chunk bodies remain deferred to the serialization matrix.
pub const SAVE_BINARY_BULK_DEFERRED: &str = "serialization matrix binary bulk body";

#[must_use]
pub fn active_save_wire_format() -> SaveWireFormat {
    SaveWireFormat::RonTextual
}

#[must_use]
pub fn active_chunk_artifact_body_kind() -> SaveArtifactBodyKind {
    SaveArtifactBodyKind::RonChunkTextual
}

#[must_use]
pub fn active_save_payload_compression() -> SavePayloadCompression {
    SavePayloadCompression::Identity
}

/// Wrap a textual chunk body in the Wave S artifact envelope (identity compression).
#[must_use]
pub fn wrap_chunk_artifact_body(body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + 1 + 1 + 1 + 4 + body.len());
    out.extend_from_slice(SAVE_ARTIFACT_MAGIC);
    out.push(SAVE_ARTIFACT_ENVELOPE_VERSION);
    out.push(active_save_payload_compression() as u8);
    out.push(active_chunk_artifact_body_kind() as u8);
    let len = u32::try_from(body.len()).unwrap_or(u32::MAX);
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(body);
    out
}

/// Strip the artifact envelope when present; legacy raw RON bytes pass through unchanged.
pub fn unwrap_chunk_artifact_body(bytes: &[u8]) -> io::Result<&[u8]> {
    if bytes.len() < SAVE_ARTIFACT_MAGIC.len()
        || &bytes[..SAVE_ARTIFACT_MAGIC.len()] != SAVE_ARTIFACT_MAGIC
    {
        return Ok(bytes);
    }
    if bytes.len() < 11 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "truncated save artifact envelope",
        ));
    }
    if bytes[4] != SAVE_ARTIFACT_ENVELOPE_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported save artifact envelope version {}",
                bytes[4]
            ),
        ));
    }
    let codec = bytes[5];
    if codec != SavePayloadCompression::Identity as u8 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported save artifact compression codec {codec}"),
        ));
    }
    let body_kind = bytes[6];
    if body_kind != SaveArtifactBodyKind::RonChunkTextual as u8 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported save artifact body kind {body_kind}"),
        ));
    }
    let payload_len = u32::from_le_bytes(bytes[7..11].try_into().unwrap()) as usize;
    let payload_start = 11usize;
    let payload_end = payload_start
        .checked_add(payload_len)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "artifact payload overflow"))?;
    if payload_end > bytes.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "save artifact payload length exceeds file",
        ));
    }
    Ok(&bytes[payload_start..payload_end])
}

#[must_use]
pub fn compress_payload(bytes: &[u8]) -> Vec<u8> {
    wrap_chunk_artifact_body(bytes)
}

pub fn decompress_payload(bytes: &[u8]) -> io::Result<Vec<u8>> {
    Ok(unwrap_chunk_artifact_body(bytes)?.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_save_wire_format_is_ron_textual() {
        assert_eq!(active_save_wire_format(), SaveWireFormat::RonTextual);
    }

    #[test]
    fn identity_envelope_round_trips_and_tags_ron_chunk_body() {
        let body = b"(schema_version: 1, chunk: (0, 0), cells: [])";
        let wrapped = wrap_chunk_artifact_body(body);
        assert_eq!(&wrapped[..4], SAVE_ARTIFACT_MAGIC);
        assert_eq!(wrapped[4], SAVE_ARTIFACT_ENVELOPE_VERSION);
        assert_eq!(wrapped[5], SavePayloadCompression::Identity as u8);
        assert_eq!(wrapped[6], SaveArtifactBodyKind::RonChunkTextual as u8);
        let decoded = unwrap_chunk_artifact_body(&wrapped).unwrap();
        assert_eq!(decoded, body);
        assert_eq!(decompress_payload(&wrapped).unwrap(), body);
    }

    #[test]
    fn legacy_raw_ron_bytes_decode_without_envelope() {
        let body = b"(schema_version: 1, chunk: (0, 0), cells: [])";
        assert_eq!(unwrap_chunk_artifact_body(body).unwrap(), body);
    }

    #[test]
    fn binary_bulk_remains_matrix_deferred() {
        assert!(SAVE_BINARY_BULK_DEFERRED.contains("binary bulk"));
        assert_eq!(active_chunk_artifact_body_kind(), SaveArtifactBodyKind::RonChunkTextual);
    }

    /// ANIM-03 tripwire: Identity is the only active payload codec (curve-compress deferred).
    #[test]
    fn anim03_identity_is_sole_active_codec() {
        assert_eq!(
            active_save_payload_compression(),
            SavePayloadCompression::Identity
        );
        assert_eq!(SavePayloadCompression::Identity as u8, 0);
    }

    /// ANIM-03 tripwire: envelope rejects non-Identity codecs (no silent fallback).
    #[test]
    fn anim03_envelope_rejects_non_identity_codec() {
        let body = b"(schema_version: 1,)";
        let mut wrapped = wrap_chunk_artifact_body(body);
        wrapped[5] = 1; // forged non-Identity codec tag
        let err = unwrap_chunk_artifact_body(&wrapped).expect_err("non-identity must fail");
        assert!(
            err.to_string().contains("compression codec"),
            "unexpected err: {err}"
        );
    }

    /// ANIM-03 tripwire: unit-interval quantize is stable at the pinned scale.
    #[test]
    fn anim03_unit_quantize_matches_scale() {
        assert_eq!(SAVE_FLOAT_QUANT_SCALE, 10_000.0);
        assert_eq!(quantize_unit_interval_f32(0.0), 0);
        assert_eq!(quantize_unit_interval_f32(1.0), 10_000);
        assert_eq!(quantize_unit_interval_f32(0.5), 5_000);
        assert_eq!(quantize_unit_interval_f32(-0.1), 0);
        assert_eq!(quantize_unit_interval_f32(1.5), 10_000);
        // Sub-quantum noise collapses (key omission of redundant precision).
        assert_eq!(
            quantize_unit_interval_f32(0.50001),
            quantize_unit_interval_f32(0.50004)
        );
        assert_ne!(
            quantize_unit_interval_f32(0.50),
            quantize_unit_interval_f32(0.51)
        );
    }
}
