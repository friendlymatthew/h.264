use std::simd::{LaneCount, SupportedLaneCount};
use std::simd::prelude::*;

use crate::errors::ByteStreamError;

/// `ByteStream` is an encapsulation of a NAL unit stream containing `START_CODE_PREFIX` and `NalUnit`.
pub struct ByteStream<'a> {
    cursor: usize,
    data: &'a [u8],
}

impl<'a> ByteStream<'a> {
    /// `data` must be an ordered stream of bytes consisting of a sequence of byte stream `NalUnit` syntax structures.
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    pub(crate) fn process(&mut self) -> Result<(), ByteStreamError> {
        self.pre_process::<64>()?;

        Ok(())
    }

    fn pre_process<const N: usize>(&mut self) -> Result<(), ByteStreamError>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        let to_find = [0x00, 0x00, 0x00, 0x01];

        if to_find.len() > N {
            return Err(ByteStreamError::InvalidLaneCount(
                "matching. provide a number that is greater than 4".to_string(),
            ));
        }

        // 1. extracts and discards each 0x00 syntax element if present,
        // moving the current position in the byte stream forward by one byte at a time, until the
        // current position in the byte stream is such that the next four bytes
        // form the four-byte sequence 0x00000001
        let mut temp_chunk = [0u8; N];

        while self.cursor < self.data.len() {
            let end = (self.cursor + N).min(self.data.len());

            let simd_chunk = match end - self.cursor == N {
                true => Simd::from_slice(&self.data[self.cursor..end]),
                false => {
                    temp_chunk[..end - self.cursor].copy_from_slice(&self.data[self.cursor..end]);
                    Simd::from_array(temp_chunk)
                }
            };

            let m1 = simd_chunk.simd_eq(Simd::splat(to_find[0]));
            if !m1.any() {
                self.cursor += N - (to_find.len() - 1);
                continue;
            }
            let m2 = simd_chunk
                .rotate_elements_left::<1>()
                .simd_eq(Simd::splat(to_find[1]));
            let m3 = simd_chunk
                .rotate_elements_left::<2>()
                .simd_eq(Simd::splat(to_find[2]));
            let m4 = simd_chunk
                .rotate_elements_left::<3>()
                .simd_eq(Simd::splat(to_find[3]));

            let mut matches = m1 & m2 & m3 & m4;
            if let Some(rel_index) = matches.first_set() {
                matches.set(rel_index, false);
                self.cursor += rel_index;
                return Ok(());
            }

            /*
            Suppose our chunk looked like this:
                                                  |  next chunk...
                                                  |
            [...0x00, 0x00, 0x00, 0x00, 0x00, 0x00][ 0x01...]
                                                  |
            Since our marker is  [0x00, 0x00, 0x00, 0x01], worst case is to move our cursor
                                | <-- here!
             */
            self.cursor += N - (to_find.len() - 1);
        }

        Err(ByteStreamError::PreProcessTerminationMarkerNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_basic() -> Result<(), ByteStreamError> {
        let data = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 8,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 8,
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let mut bs = ByteStream::new(&data);
        bs.pre_process::<8>()?;

        assert_eq!(bs.cursor, 13);
        Ok(())
    }

    #[test]
    fn test_preprocess_err() -> Result<(), ByteStreamError> {
        let data = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let mut bs = ByteStream::new(&data);
        assert!(bs.pre_process::<4>().is_err());

        Ok(())
    }
}
