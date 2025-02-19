/// Single-byte chars from Big Endian `u32` value.
/// Maps 0-255 to `char`, exceeding ascii.
pub(crate) fn chars_from_be_u32(value: u32) -> [char; 4] {
    let a = value.to_be_bytes();
    chars_from_bytes(a)
}

/// Single-byte chars from `[u8; 4]`.
/// Each byte maps 0-255 to `char`, exceeding ascii.
pub(crate) fn chars_from_bytes(bytes: [u8; 4]) -> [char; 4] {
    [
        bytes[0] as char,
        bytes[1] as char,
        bytes[2] as char,
        bytes[3] as char,
    ]
}

/// String from `[u8; 4]`.
/// Each byte maps 0-255 to `char`, exceeding ascii.
pub(crate) fn string_from_bytes(bytes: [u8; 4]) -> String {
    chars_from_bytes(bytes).iter().collect()
}

/// String from Big Endian `u32` value.
pub(crate) fn string_from_be_u32(value: u32, ignore_null: bool) -> String {
    match ignore_null {
        true => value.to_be_bytes().iter()
            .filter_map(|b| if b == &0 {None} else {Some(*b as char)})
            .collect(),
        false => value.to_be_bytes().iter()
            .map(|b| *b as char)
            .collect(),
    }
}

/// Counted string.
pub(crate) fn counted_string(bytes: &[u8], ignore_null: bool) -> String {
    assert!(!bytes.is_empty(), "No data to construct counted string from.");
    let count = bytes[0];
    match ignore_null {
        true => bytes[1 .. count as usize + 1].iter()
            .filter_map(|b| if b != &0 {Some(*b as char)} else {None})
            .collect(),
        false => bytes[1 .. count as usize + 1].iter()
            .map(|b| *b as char)
            .collect(),
    }
}

/// Converts a `Vec<T>`` a sized array.
pub(crate) fn vec2arr<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but received length {}", N, v.len()))
}

/// Returns a array of `char`s.
pub(crate) fn str2arr<const N: usize>(value: &str) -> [char; N] {
    let val = value.chars().collect::<Vec<_>>();
    vec2arr::<char, N>(val)
}