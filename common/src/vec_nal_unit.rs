use std::simd::{LaneCount, SupportedLaneCount};
use std::simd::prelude::*;

/// `VecNalUnit` are columnar representations of `NalUnit`. Helpful for SIMD.
#[derive(Debug)]
pub struct VecNalUnit {
    forbidden_zero_bits: Vec<u8>,
    nal_ref_idcs: Vec<u8>,
    nal_unit_types: Vec<u8>,
}

impl VecNalUnit {
    fn parse_nal_units<const N: usize>(
        chunk: Simd<u8, N>,
    ) -> (Simd<u8, N>, Simd<u8, N>, Simd<u8, N>)
    where
        LaneCount<N>: SupportedLaneCount,
    {
        let forbidden_zero_bits = chunk >> 7;
        let nal_ref_idcs = (chunk >> 5) & Simd::splat(0b11);
        let nal_unit_types = chunk & Simd::splat(0b00011111);

        (forbidden_zero_bits, nal_ref_idcs, nal_unit_types)
    }
}
