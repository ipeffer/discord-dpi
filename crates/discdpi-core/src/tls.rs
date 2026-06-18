//! Minimal TLS ClientHello parser for SNI extraction.

/// Returns true when the TCP payload begins with a TLS Handshake ClientHello record.
pub fn is_tls_client_hello(payload: &[u8]) -> bool {
    payload.len() >= 6 && payload[0] == 0x16 && payload[5] == 0x01
}

/// Extract the first SNI hostname from a TLS ClientHello carried in `payload`.
pub fn parse_client_hello_sni(payload: &[u8]) -> Option<String> {
    if !is_tls_client_hello(payload) {
        return None;
    }

    let handshake = &payload[5..];
    if handshake.len() < 4 || handshake[0] != 0x01 {
        return None;
    }

    let mut pos = 4usize;
    pos += 2; // client version
    pos += 32; // random
    if pos >= handshake.len() {
        return None;
    }

    let session_id_len = handshake[pos] as usize;
    pos += 1;
    pos = pos.checked_add(session_id_len)?;
    if pos + 2 > handshake.len() {
        return None;
    }

    let cipher_len = u16::from_be_bytes([handshake[pos], handshake[pos + 1]]) as usize;
    pos += 2;
    pos = pos.checked_add(cipher_len)?;
    if pos >= handshake.len() {
        return None;
    }

    let compression_len = handshake[pos] as usize;
    pos += 1;
    pos = pos.checked_add(compression_len)?;
    if pos + 2 > handshake.len() {
        return None;
    }

    let extensions_len = u16::from_be_bytes([handshake[pos], handshake[pos + 1]]) as usize;
    pos += 2;
    let extensions_end = pos.checked_add(extensions_len)?;
    if extensions_end > handshake.len() {
        return None;
    }

    while pos + 4 <= extensions_end {
        let ext_type = u16::from_be_bytes([handshake[pos], handshake[pos + 1]]);
        let ext_size = u16::from_be_bytes([handshake[pos + 2], handshake[pos + 3]]) as usize;
        pos += 4;
        if pos + ext_size > extensions_end {
            break;
        }

        if ext_type == 0 {
            if let Some(host) = parse_sni_extension(&handshake[pos..pos + ext_size]) {
                return Some(host);
            }
        }

        pos += ext_size;
    }

    None
}

fn parse_sni_extension(data: &[u8]) -> Option<String> {
    if data.len() < 3 {
        return None;
    }

    let list_len = u16::from_be_bytes([data[0], data[1]]) as usize;
    let mut pos = 2usize;
    let list_end = pos.checked_add(list_len)?;
    if list_end > data.len() {
        return None;
    }

    while pos + 3 <= list_end {
        let name_len = u16::from_be_bytes([data[pos + 1], data[pos + 2]]) as usize;
        pos += 3;
        if pos + name_len > list_end {
            break;
        }
        let host = std::str::from_utf8(&data[pos..pos + name_len]).ok()?;
        return Some(host.to_ascii_lowercase());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_client_hello() {
        let payload = [0x16, 0x03, 0x01, 0x00, 0x05, 0x01, 0x00, 0x00, 0x01, 0x03];
        assert!(is_tls_client_hello(&payload));
    }
}
