#![feature(portable_simd)]
pub(crate) const LANE_COUNT: usize = 64;

mod decoder;
mod nal_unit;
