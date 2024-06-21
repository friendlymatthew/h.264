use std::simd::Simd;

/// `NalUnit` is a syntax structure containing an indication of the type of data to follow and bytes
/// containing that data in the form of an `RBSP` interspersed as necessary with
/// `EMULATION_PREVENTION_BYTE`.
#[derive(Debug, Copy, Clone)]
pub struct NalUnit {
    offset: usize,
    num_bytes: usize,

    /// `forbidden_zero_bit` shall be equal to 0.
    forbidden_zero_bit: bool,

    /// `nal_ref_idc` not equal to 0 specifies that the content of the NAL unit contains:
    ///
    /// * a sequence parameter set
    /// * a sequence parameter set extension
    /// * a subset sequence parameter set
    /// * a picture parameter set
    /// * a slice of a reference picture
    /// * a slice data partition of a reference picture, or a prefix NAL unit preceding a slice of a reference picture
    ///
    /// For coded video sequences conforming to one or more of the profiles specified in Annex A that are decoded using the
    /// decoding process specified in clauses 2 to 9, `nal_ref_idc equal` to 0 for a NAL unit containing a slice or slice data partition
    /// indicates that the slice or slice data partition is part of a non-reference picture.
    ///
    /// `nal_ref_idc` shall not be equal to 0 for sequence parameter set or sequence parameter set extension or subset sequence
    /// parameter set or picture parameter set NAL units. When `nal_ref_idc` is equal to 0 for one NAL unit with `nal_unit_type` in
    /// the range of 1 to 4, inclusive, of a particular picture, it shall be equal to 0 for all NAL units with `nal_unit_type` in the range
    /// of 1 to 4, inclusive, of the picture.
    ///
    ///
    /// `nal_ref_idc` shall not be equal to 0 for NAL units with nal_unit_type equal to 5.
    ///
    /// `nal_ref_idc` shall be equal to 0 for all NAL units having nal_unit_type equal to 6, 9, 10, 11, or 12.
    nal_ref_idc: u8,

    /// `nal_unit_type` specifies the type of `RBSP` data structure contained in the NAL unit.
    nal_unit_type: u8,
}

impl NalUnit {
    /// A unique sequence of three bytes equal to `0x000001` embedded in the byte stream as a prefix
    /// to each `NALUnit`. The location of a `START_CODE_PREFIX` can be used by a decoder to identify
    /// the beginning of a new `NAL unit` and the end of a previous NAL unit. Emulation of start code
    /// prefixes is prevented within NAL units by the inclusion of `EMULATION_PREVENTION_BYTES`.
    pub(crate) const START_CODE_PREFIX: [u8; 3] = [0x00, 0x00, 0x01];

    /// A byte equal to 0x03 that may be present within a `NalUnit`.
    /// The presence of this byte ensures no sequence of consecutive byte-aligned bytes in the
    /// `NALUnit` contains a `START_CODE_PREFIX`.
    const EMULATION_PREVENTION_BYTE: u8 = 0x03;

    pub(crate) fn start_code_prefix_simd_array() -> Simd<u8, 4> {
        let mut prefix_array = [0x00; 4];
        prefix_array[..3].copy_from_slice(&Self::START_CODE_PREFIX);
        Simd::from_array(prefix_array)
    }
}
