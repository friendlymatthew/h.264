use std::fs::File;
use std::simd::prelude::*;

use anyhow::Result;
use memmap::Mmap;
use simd_iter::SimdIterExt;

use crate::LANE_COUNT;
use crate::nal_unit::NalUnit;

#[derive(Debug)]
pub struct Decoder {
    data: Mmap,
    cursor: usize,
}

impl Decoder {
    pub(crate) fn from_file_path(file_path: &str) -> Result<Decoder> {
        Ok(Decoder {
            data: unsafe { Mmap::map(&File::open(file_path)?)? },
            cursor: 0,
        })
    }

    pub(crate) fn decode(&mut self) -> Result<()> {
        let (_nal_unit_headers, _nal_unit_header_offsets) = self.find_nal_units()?;

        Ok(())
    }

    fn find_nal_units(&mut self) -> Result<(Vec<u8>, Vec<usize>)> {
        let mut headers = vec![];
        let mut header_offsets = vec![];

        while let Some(curr_chunk) = self.data.simd_iter::<LANE_COUNT>().next() {
            let [p1, p2, p3] = NalUnit::PREFIX_CODE;

            let match_p1 = curr_chunk.simd_eq(Simd::splat(p1));
            let match_p2 = curr_chunk
                .rotate_elements_left::<1>()
                .simd_eq(Simd::splat(p2));
            let match_p3 = curr_chunk
                .rotate_elements_left::<2>()
                .simd_eq(Simd::splat(p3));

            let mut nal_unit_p_matches = match_p1 & match_p2 & match_p3;
            while let Some(p_index) = nal_unit_p_matches.first_set() {
                nal_unit_p_matches.set(p_index, false);
                let h_offset = self.cursor + p_index + NalUnit::PREFIX_CODE.len();
                headers.push(self.data[h_offset]);
                header_offsets.push(h_offset);
            }
        }

        Ok((headers, header_offsets))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silver_surfer() -> Result<()> {
        let _decoder = Decoder::from_file_path("./gravity.mp4")?;

        Ok(())
    }
}
