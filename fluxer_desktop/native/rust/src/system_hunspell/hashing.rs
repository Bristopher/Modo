// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use sha2::{Digest, Sha256};

pub const HEX_LEN: usize = 64;
const CHUNK_SIZE: usize = 64 * 1024;

pub fn bytes_to_hex_lower(bytes: &[u8], out: &mut [u8]) {
    assert!(out.len() >= bytes.len() * 2);
    const ALPHABET: &[u8; 16] = b"0123456789abcdef";
    for (index, byte) in bytes.iter().copied().enumerate() {
        out[index * 2] = ALPHABET[(byte >> 4) as usize];
        out[index * 2 + 1] = ALPHABET[(byte & 0x0f) as usize];
    }
}

pub fn hash_file_to_hex(path: impl AsRef<Path>) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; CHUNK_SIZE];
    loop {
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    let digest = hasher.finalize();
    let mut out = [0_u8; HEX_LEN];
    bytes_to_hex_lower(&digest, &mut out);
    let mut hex = String::with_capacity(HEX_LEN);
    for byte in out {
        hex.push(char::from(byte));
    }
    Ok(hex)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn bytes_to_hex_lower_formats_lowercase_hex() {
        let mut out = [0_u8; 8];
        bytes_to_hex_lower(&[0xde, 0xad, 0xbe, 0xef], &mut out);
        assert_eq!("deadbeef", std::str::from_utf8(&out).unwrap());
    }

    #[test]
    fn bytes_to_hex_lower_formats_zero_and_edge_bytes() {
        let mut out = [0_u8; 10];
        bytes_to_hex_lower(&[0x00, 0x0f, 0xf0, 0xff, 0x10], &mut out);
        assert_eq!("000ff0ff10", std::str::from_utf8(&out).unwrap());
    }

    #[test]
    fn hash_file_to_hex_matches_known_sha256_for_abc() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("abc");
        std::fs::write(&path, b"abc").unwrap();
        assert_eq!(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            hash_file_to_hex(&path).unwrap()
        );
    }

    #[test]
    fn hash_file_to_hex_over_chunk_boundary_matches_single_shot_sha256() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("chunked");
        let mut file = File::create(&path).unwrap();
        let mut payload = vec![0_u8; 200 * 1024];
        let mut value = 0xc0ffee_u64;
        for byte in &mut payload {
            value ^= value << 13;
            value ^= value >> 7;
            value ^= value << 17;
            *byte = value as u8;
        }
        file.write_all(&payload).unwrap();

        let mut oneshot = Sha256::new();
        oneshot.update(&payload);
        let digest = oneshot.finalize();
        let mut expected = [0_u8; HEX_LEN];
        bytes_to_hex_lower(&digest, &mut expected);
        assert_eq!(
            std::str::from_utf8(&expected).unwrap(),
            hash_file_to_hex(&path).unwrap()
        );
    }

    #[test]
    fn hash_file_to_hex_returns_open_error_for_missing_file() {
        assert!(hash_file_to_hex("/nonexistent/path/that/should/not/exist.bin").is_err());
    }
}
