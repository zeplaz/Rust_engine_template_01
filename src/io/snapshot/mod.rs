//! **M5 / wave S** stub: hybrid-shaped **dev** snapshot (text header + body bytes).
//! Body is **transport R8** UTF-8: **RON** canonical (editor); **JSON** still accepted on load (try RON then JSON).

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

/// First line of file (JSON). `transport_byte_len` must match the body size in bytes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldSnapshotHeaderDevV0 {
    pub format: String,
    pub format_version: u32,
    pub transport_byte_len: usize,
}

const FORMAT_ID: &str = "hybrid_world_snapshot_dev_v0";

fn parse_dev_snapshot_header_line(line: &[u8]) -> io::Result<WorldSnapshotHeaderDevV0> {
    let s = std::str::from_utf8(line).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let t = s.trim_start();
    if t.starts_with('{') {
        serde_json::from_slice(line).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    } else {
        ron::de::from_str(s).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

/// Write **JSON** header line + newline + body (transport RON/JSON bytes).
pub fn write_hybrid_world_snapshot_dev_v0(path: &Path, transport_body: &[u8]) -> io::Result<()> {
    let header = WorldSnapshotHeaderDevV0 {
        format: FORMAT_ID.into(),
        format_version: 1,
        transport_byte_len: transport_body.len(),
    };
    let h = serde_json::to_string(&header).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut out = Vec::with_capacity(h.len() + 1 + transport_body.len());
    out.extend_from_slice(h.as_bytes());
    out.push(b'\n');
    out.extend_from_slice(transport_body);
    fs::write(path, out)
}

/// Same as [`write_hybrid_world_snapshot_dev_v0`] but header is **RON** (single line).
pub fn write_hybrid_world_snapshot_dev_v0_ron_header(path: &Path, transport_body: &[u8]) -> io::Result<()> {
    let header = WorldSnapshotHeaderDevV0 {
        format: FORMAT_ID.into(),
        format_version: 1,
        transport_byte_len: transport_body.len(),
    };
    let h = ron::ser::to_string(&header).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut out = Vec::with_capacity(h.len() + 1 + transport_body.len());
    out.extend_from_slice(h.as_bytes());
    out.push(b'\n');
    out.extend_from_slice(transport_body);
    fs::write(path, out)
}

/// Read header + body. Does not decode transport (domain owns RON/JSON schema).
pub fn read_hybrid_world_snapshot_dev_v0(path: &Path) -> io::Result<(WorldSnapshotHeaderDevV0, Vec<u8>)> {
    let data = fs::read(path)?;
    let nl = data
        .iter()
        .position(|&b| b == b'\n')
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing newline after header"))?;
    let header: WorldSnapshotHeaderDevV0 = parse_dev_snapshot_header_line(&data[..nl])?;
    let body = data[nl + 1..].to_vec();
    if body.len() != header.transport_byte_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "body len {} != header.transport_byte_len {}",
                body.len(),
                header.transport_byte_len
            ),
        ));
    }
    if header.format != FORMAT_ID {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown format {}", header.format),
        ));
    }
    Ok((header, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hybrid_dev_round_trip_header_and_len() {
        let body = b"(schema_version:1,nodes:[],edges:[])".to_vec();
        let dir = std::env::temp_dir().join("hybrid_dev_test.sav");
        write_hybrid_world_snapshot_dev_v0(&dir, &body).unwrap();
        let (h, b) = read_hybrid_world_snapshot_dev_v0(&dir).unwrap();
        assert_eq!(h.transport_byte_len, body.len());
        assert_eq!(b, body);
        let _ = fs::remove_file(&dir);
    }

    #[test]
    fn hybrid_dev_round_trip_ron_header() {
        let body = b"(schema_version:1,nodes:[],edges:[])".to_vec();
        let dir = std::env::temp_dir().join("hybrid_dev_ron_hdr.sav");
        write_hybrid_world_snapshot_dev_v0_ron_header(&dir, &body).unwrap();
        let (h, b) = read_hybrid_world_snapshot_dev_v0(&dir).unwrap();
        assert_eq!(h.transport_byte_len, body.len());
        assert_eq!(b, body);
        let _ = fs::remove_file(&dir);
    }
}
