use base64::{engine::general_purpose::STANDARD, Engine as _};

pub fn decode_after<'a>(after: &'a str) -> Option<String> {
    let u8_vec = STANDARD.decode(after).ok()?;
    std::str::from_utf8(&u8_vec).ok().map(|s| s.to_string())
}

pub fn encode_cursor(cursor: &str) -> String {
    STANDARD.encode(cursor.as_bytes())
}
