#![feature(portable_simd)]

use std::simd::SupportedLaneCount;

mod byte_stream;
mod nal_unit;
mod rbsp;
mod vec_nal_unit;
