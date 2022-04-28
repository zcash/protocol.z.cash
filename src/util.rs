use serde::{de, Deserialize, Deserializer};

/// Serde deserialization decorator to map empty Strings to `true`.
pub(crate) fn empty_string_as_true<'de, D>(de: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(true),
        Some(s) => s.parse().map_err(de::Error::custom),
    }
}
