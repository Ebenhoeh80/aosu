//! Generic, publicly documented Tuya LAN wire framing (magic bytes,
//! sequence, command, length, CRC32, footer). This is the open protocol
//! format published in independent implementations such as tinytuya and
//! LocalTuya -- not a secret of any particular vendor or app, and no
//! device key is needed to build or parse the envelope itself.
//!
//! This crate is deliberately **brand-agnostic**: it knows nothing about
//! Aosu, Glazero, or any other specific Tuya-derived product. Any client
//! for any Tuya-ecosystem camera or device can depend on it. Device-key
//! handling, DP (data point) semantics and vendor-specific behavior stay
//! out of this crate on purpose -- that's each client's own problem, and
//! each may differ (see `docs/architecture.md` in the aosu-desktop
//! project for how one such client, built on this crate, is laid out).

pub const PREFIX: u32 = 0x0000_55AA;
pub const SUFFIX: u32 = 0x0000_AA55;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    DpQuery,
    SessKeyNegStart,
    SessKeyNegFinish,
    Other(u32),
}

impl Command {
    fn code(self) -> u32 {
        match self {
            Command::DpQuery => 0x0a,
            Command::SessKeyNegStart => 0x03,
            Command::SessKeyNegFinish => 0x04,
            Command::Other(c) => c,
        }
    }
}

/// Build a frame with the given sequence number, command and payload.
/// Mirrors the generic envelope; carries no vendor-specific or secret
/// bytes -- callers own any encryption/signing of the payload itself.
pub fn build_frame(seq: u32, command: Command, payload: &[u8]) -> Vec<u8> {
    let mut body = Vec::with_capacity(16 + payload.len());
    body.extend_from_slice(&PREFIX.to_be_bytes());
    body.extend_from_slice(&seq.to_be_bytes());
    body.extend_from_slice(&command.code().to_be_bytes());
    body.extend_from_slice(&((payload.len() + 8) as u32).to_be_bytes());
    body.extend_from_slice(payload);
    let crc = crc32fast::hash(&body);
    let mut frame = body;
    frame.extend_from_slice(&crc.to_be_bytes());
    frame.extend_from_slice(&SUFFIX.to_be_bytes());
    frame
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameHeader {
    pub prefix_ok: bool,
    pub seq: u32,
    pub command: u32,
    pub declared_length: u32,
}

/// Parse just the 16-byte header of a response, for fingerprinting. Does
/// not attempt to decrypt or interpret any payload.
pub fn parse_header(data: &[u8]) -> Option<FrameHeader> {
    if data.len() < 16 {
        return None;
    }
    let prefix = u32::from_be_bytes(data[0..4].try_into().ok()?);
    let seq = u32::from_be_bytes(data[4..8].try_into().ok()?);
    let command = u32::from_be_bytes(data[8..12].try_into().ok()?);
    let declared_length = u32::from_be_bytes(data[12..16].try_into().ok()?);
    Some(FrameHeader {
        prefix_ok: prefix == PREFIX,
        seq,
        command,
        declared_length,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_header_fields() {
        let frame = build_frame(7, Command::DpQuery, b"");
        let header = parse_header(&frame).expect("header parses");
        assert!(header.prefix_ok);
        assert_eq!(header.seq, 7);
        assert_eq!(header.command, 0x0a);
        assert_eq!(header.declared_length as usize, frame.len() - 16);
    }

    #[test]
    fn frame_ends_with_suffix() {
        let frame = build_frame(1, Command::SessKeyNegStart, &[0u8; 16]);
        let tail = &frame[frame.len() - 4..];
        assert_eq!(u32::from_be_bytes(tail.try_into().unwrap()), SUFFIX);
    }

    #[test]
    fn crc_changes_when_payload_changes() {
        let a = build_frame(1, Command::DpQuery, b"a");
        let b = build_frame(1, Command::DpQuery, b"b");
        assert_ne!(a, b);
    }

    #[test]
    fn too_short_header_is_rejected() {
        assert!(parse_header(&[0u8; 10]).is_none());
    }

    proptest::proptest! {
        #[test]
        fn any_payload_roundtrips_header(seq in 0u32..=u32::MAX, len in 0usize..64) {
            let payload = vec![0xABu8; len];
            let frame = build_frame(seq, Command::Other(0x42), &payload);
            let header = parse_header(&frame).unwrap();
            assert!(header.prefix_ok);
            assert_eq!(header.seq, seq);
            assert_eq!(header.command, 0x42);
        }
    }
}
