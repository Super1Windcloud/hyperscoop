use sha2::Digest;
use std::io::{self, Read};

/// Hash a reader with bounded memory, including readers that return short or interrupted reads.
pub fn hash_reader<D: Digest>(mut reader: impl Read) -> io::Result<String> {
    let mut hasher = D::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => return Ok(hex::encode(hasher.finalize())),
            Ok(len) => hasher.update(&buffer[..len]),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use sha1::Sha1;
    use sha2::{Sha256, Sha512};

    #[test]
    fn streaming_hash_handles_short_reads_with_known_digest() {
        struct ShortReads(std::io::Cursor<&'static [u8]>);
        impl Read for ShortReads {
            fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
                self.0.read(&mut buffer[..1])
            }
        }
        assert_eq!(
            hash_reader::<Sha256>(ShortReads(std::io::Cursor::new(b"abc"))).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn streaming_hashes_match_one_shot_across_buffer_boundaries() {
        for len in [0, 3, 65535, 65536, 65537, 1024 * 1024 + 3] {
            let data = vec![0x5a; len];
            assert_eq!(
                hash_reader::<Sha1>(&data[..]).unwrap(),
                hex::encode(Sha1::digest(&data))
            );
            assert_eq!(
                hash_reader::<Sha256>(&data[..]).unwrap(),
                hex::encode(Sha256::digest(&data))
            );
            assert_eq!(
                hash_reader::<Sha512>(&data[..]).unwrap(),
                hex::encode(Sha512::digest(&data))
            );
        }
    }

    #[test]
    fn streaming_hash_retries_interruption_and_propagates_io_errors() {
        struct Interrupted(bool);
        impl Read for Interrupted {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                if self.0 {
                    Ok(0)
                } else {
                    self.0 = true;
                    Err(io::ErrorKind::Interrupted.into())
                }
            }
        }
        assert_eq!(
            hash_reader::<Sha256>(Interrupted(false)).unwrap(),
            hex::encode(Sha256::digest([]))
        );
        struct Failed;
        impl Read for Failed {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::ErrorKind::PermissionDenied.into())
            }
        }
        assert_eq!(
            hash_reader::<Sha256>(Failed).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
    }
}
