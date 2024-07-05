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
        self.preprocess::<64>()?;

        let mut nal_units = vec![];

        while self.cursor < self.data.len() {
            // check the next four bytes in the bitstream
            if self.cursor + 4 >= self.data.len() {
                return Err(ByteStreamError::UnexpectedTermination(
                    "reached end of bitstream".to_string(),
                ));
            }

            // 1. the next four bytes in the bitstream form the four-byte sequence 0x00 00 00 01
            let mut first_four_bytes = !Simd::from_slice(&self.data[self.cursor..self.cursor + 4])
                .simd_eq(Simd::from_array([0x00, 0x00, 0x00, 0x01]));

            if let Some(_) = first_four_bytes.first_set() {
                return Err(ByteStreamError::IncorrectByteSequence {
                    expected: "0x00 0x00 0x00 0x01".to_string(),
                    got: format!("{:?}", self.data[self.cursor..self.cursor + 4].to_vec()),
                });
            }

            self.cursor += 4;
            let mut num_bytes_in_nal_unit = (self.cursor, 0);

            while self.cursor < self.data.len() {
                if self.cursor == self.data.len() - 1 {
                    num_bytes_in_nal_unit.1 = self.cursor - num_bytes_in_nal_unit.0;
                    nal_units.push(num_bytes_in_nal_unit);
                    break;
                }

                if self.cursor + 3 < self.data.len() {
                    let next_three_bytes = Simd::from_array([
                        self.data[self.cursor],
                        self.data[self.cursor + 1],
                        self.data[self.cursor + 2],
                        0x00,
                    ]);
                    let (m1, m2) = (
                        Simd::from_array([0x00, 0x00, 0x01, 0x00]),
                        Simd::splat(0x00),
                    );

                    if next_three_bytes.simd_eq(m1).all() || next_three_bytes.simd_eq(m2).all() {
                        num_bytes_in_nal_unit.1 = self.cursor - num_bytes_in_nal_unit.0;
                        self.cursor += 3;

                        while self.cursor + 4 < self.data.len() {
                            let next_four_chunk =
                                Simd::from_slice(&self.data[self.cursor..self.cursor + 4]);

                            let next_batch_mask =
                                next_four_chunk.simd_eq(Simd::from_array([0x00, 0x00, 0x00, 0x01]));

                            if next_batch_mask.all() {
                                break;
                            }

                            if let Some(rel_index) = next_batch_mask.first_set() {
                                if rel_index != 3 {
                                    if self.data[self.cursor + 3] != 0x00 {
                                        return Err(ByteStreamError::UnexpectedByte(
                                            format!(
                                                "{:?}",
                                                self.data[self.cursor..self.cursor + 4].to_vec()
                                            ),
                                            "0x00".to_string(),
                                        ));
                                    }
                                }
                            }

                            self.cursor += 4;
                        }
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn preprocess<const N: usize>(&mut self) -> Result<(), ByteStreamError>
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
                panic!("no trailing zeros exist at all. Are you sure this is a h264 file?");
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

        Err(ByteStreamError::UnexpectedTermination(
            "reached end of bitstream during preprocessing.".to_string(),
        ))
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
        bs.preprocess::<8>()?;

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
        assert!(bs.preprocess::<4>().is_err());

        Ok(())
    }
}
