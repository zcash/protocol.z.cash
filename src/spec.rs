/// Returns the URL to this identifier in its matching protocol specification, or the
/// empty string if no match is found.
pub(crate) fn get_location(identifier: &str) -> String {
    // For now, just hard-code the matching logic.
    let mut identifier_parts = identifier.split(':');
    if let Some(prefix) = identifier_parts.next() {
        match prefix {
            "TCR" => return format!("https://zips.z.cash/protocol/protocol.pdf#{}", identifier),
            _ if prefix.starts_with("zip-") => {
                if let Some(anchor) = identifier_parts.next() {
                    return format!("https://zips.z.cash/{}#{}", prefix, anchor);
                }
            }
            _ => (),
        }
    }

    // No match.
    String::new()
}
