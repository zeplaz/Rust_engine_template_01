//! Deserialize serde-friendly terrain registry DTOs from disk: **`.ron`** and **`.json`**, same policy as `tuning_io`.

use std::io::{self, ErrorKind};
use std::path::Path;

use serde::de::DeserializeOwned;

pub(crate) fn deserialize_from_str_by_path<T: DeserializeOwned>(text: &str, path: &Path) -> io::Result<T> {
    let ext = path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase());
    deserialize_from_str_with_extension_opt(text, ext.as_deref())
}

/// Extension only (e.g. from [`bevy::asset::AssetPath::get_full_extension`]); `None` ⇒ RON then JSON.
pub(crate) fn deserialize_from_str_with_extension_opt<T: DeserializeOwned>(
    text: &str,
    ext: Option<&str>,
) -> io::Result<T> {
    let ext_norm = ext.map(|e| e.to_ascii_lowercase());
    match ext_norm.as_deref() {
        Some("json") => serde_json::from_str(text).map_err(|e| {
            io::Error::new(ErrorKind::InvalidData, format!("JSON: {e}"))
        }),
        Some("ron") => ron::de::from_str(text).map_err(|e| {
            io::Error::new(ErrorKind::InvalidData, format!("RON: {e}"))
        }),
        None | Some(_) => match ron::de::from_str::<T>(text) {
            Ok(v) => Ok(v),
            Err(e_ron) => serde_json::from_str(text).map_err(|e_json| {
                io::Error::new(
                    ErrorKind::InvalidData,
                    format!("RON: {e_ron}; JSON: {e_json}"),
                )
            }),
        },
    }
}

pub(crate) fn read_to_deserializable<T: DeserializeOwned>(path: &Path) -> io::Result<T> {
    let s = std::fs::read_to_string(path)?;
    deserialize_from_str_by_path(&s, path)
}
