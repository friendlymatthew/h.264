#[derive(Debug, Clone, Copy)]
pub(crate) struct NalUnitHeader {
    forbidden_zero_bit: u8,
    nal_ref_idc: u8,
    nal_unit_type: u8,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NalUnit {
    nal_unit_header: NalUnitHeader,
    raw_byte_seq_payload: RawByteSequencePayload,
}

impl NalUnit {
    pub(crate) const PREFIX_CODE: [u8; 3] = [0x00, 0x00, 0x03];
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RawByteSequencePayload {}
