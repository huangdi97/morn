//! Shared safety limits for out-of-process provider wires.
//!
//! Provider subprocesses are untrusted runtime peers. A malformed runtime must
//! not be able to allocate an unbounded line or deadlock the owning Morn thread
//! by filling a bounded reader channel during teardown.

use std::io::BufRead;

pub(crate) const MAX_PROVIDER_WIRE_FRAME_BYTES: usize = 4 * 1024 * 1024;
pub(crate) const MAX_PROVIDER_WIRE_BUFFERED_FRAMES: usize = 1024;

pub(crate) fn read_bounded_utf8_line<R: BufRead>(
    reader: &mut R,
    label: &str,
) -> std::result::Result<Option<String>, String> {
    let mut bytes = Vec::new();
    loop {
        let available = reader
            .fill_buf()
            .map_err(|error| format!("read {label}: {error}"))?;
        if available.is_empty() {
            if bytes.is_empty() {
                return Ok(None);
            }
            break;
        }
        let take = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |index| index + 1);
        if bytes.len().saturating_add(take) > MAX_PROVIDER_WIRE_FRAME_BYTES {
            return Err(format!(
                "{label} frame exceeds {} bytes",
                MAX_PROVIDER_WIRE_FRAME_BYTES
            ));
        }
        bytes.extend_from_slice(&available[..take]);
        let ended = bytes.last() == Some(&b'\n');
        reader.consume(take);
        if ended {
            break;
        }
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| format!("{label} emitted non-UTF-8 data"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn bounded_reader_accepts_line_and_eof_record() {
        let mut reader = Cursor::new(b"{\"ok\":true}\nlast".to_vec());
        assert_eq!(
            read_bounded_utf8_line(&mut reader, "fixture").unwrap().as_deref(),
            Some("{\"ok\":true}\n")
        );
        assert_eq!(
            read_bounded_utf8_line(&mut reader, "fixture").unwrap().as_deref(),
            Some("last")
        );
        assert!(read_bounded_utf8_line(&mut reader, "fixture").unwrap().is_none());
    }

    #[test]
    fn bounded_reader_rejects_oversized_and_non_utf8_frames() {
        let mut oversized = Cursor::new(vec![b'x'; MAX_PROVIDER_WIRE_FRAME_BYTES + 1]);
        assert!(read_bounded_utf8_line(&mut oversized, "fixture").is_err());
        let mut invalid = Cursor::new(vec![0xff, b'\n']);
        assert!(read_bounded_utf8_line(&mut invalid, "fixture").is_err());
    }
}
