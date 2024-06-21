use std::simd::{LaneCount, SupportedLaneCount};
use std::simd::prelude::*;

use anyhow::{anyhow, Result};

use crate::nal_unit::NalUnit;

/// `ByteStream` is an encapsulation of a NAL unit stream containing `START_CODE_PREFIX` and `NalUnit`.
pub struct ByteStream<'a> {
    cursor: usize,
    data: &'a [u8],
}

impl<'a> ByteStream<'a> {
    /// `data` must be an ordered stream of bytes consisting of a sequence of byte stream `NalUnit` syntax structures.
    fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    // `read` is a scalar implementation of the byte stream NAL unit decoding process
    fn read(&mut self) -> Result<()> {
        self.pre_process::<64>()?;

        while self.cursor < self.data.len() {
            debug_assert!(
                Simd::from_array([
                    self.data[self.cursor],
                    self.data[self.cursor + 1],
                    self.data[self.cursor + 2],
                    self.data[self.cursor + 3]
                ])
                .simd_eq(Simd::from_array([0x00, 0x00, 0x00, 0x01]))
                .all(),
                "Expected the next four bytes in the bytestream form the four-byte sequence 0x00, 0x00, 0x00, 0x01. Got: {:?}", &self.data[self.cursor..self.cursor + 4]
            );

            self.cursor += 4;

            debug_assert_eq!(
                self.data[self.cursor],
                0x00,
               "Expected the next byte in the byte stream to be a zero_byte syntax element. Got: {:?}", &self.data[self.cursor]
            );

            self.cursor += 1;

            debug_assert!(
                NalUnit::start_code_prefix_simd_array()
                .simd_eq(Simd::from_array([
                        self.data[self.cursor],
                        self.data[self.cursor + 1],
                        self.data[self.cursor + 2],
                        0x00,
                    ]))
                    .all(),
                "Expected the next three-byte sequence to be the NalUnit start code prefix. Got: {:?}", &self.data[self.cursor..self.cursor + 3]
            );

            self.cursor += 3;
        }

        Ok(())
    }

    fn pre_process<const N: usize>(&mut self) -> Result<()>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        let to_find = [0x00, 0x00, 0x00, 0x01];

        if to_find.len() > N {
            return Err(anyhow!("provide a LaneCount number that is greater than 4"));
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
            while let Some(rel_index) = matches.first_set() {
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

        Err(anyhow!(
            "reached end of bytestream and unable to find `0x00000001`."
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_basic() -> Result<()> {
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
    fn test_preprocess_err() -> Result<()> {
        let data = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let mut bs = ByteStream::new(&data);
        assert!(bs.pre_process::<4>().is_err());

        Ok(())
    }
}
