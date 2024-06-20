/// Definitions of `RBSP` pulled from Table 7-1.
#[derive(Debug)]
pub enum RBSP {
    Unspecified,
    CodedSliceNonIDRPicture,
    CodedSliceDataPartitionA,
    CodedSliceDataPartitionB,
    CodedSliceDataPartitionC,
    CodedSliceIDRPicture,
    SupplementalEnhancementInformation,
    SequenceParameterSet,
    PictureParameterSet,
    AccessUnitDelimiter,
    SequenceEnd,
    StreamEnd,
    FillerData,
    SequenceParameterSetExtension,
    PrefixNALUnit,
    SubsetSequenceParameterSet,
    DepthParameterSet,
    Reserved,
    CodedSliceAuxiliaryCodedPictureNonPartitioning,
    CodedSliceExtension,
    CodedSliceExtensionDepthViewComponent,
}

impl RBSP {
    pub fn from_nal_unit_type(nal_unit_type: &u8) -> Self {
        match nal_unit_type {
            0 | 24..=31 => RBSP::Unspecified,
            1 => RBSP::CodedSliceNonIDRPicture,
            2 => RBSP::CodedSliceDataPartitionA,
            3 => RBSP::CodedSliceDataPartitionB,
            4 => RBSP::CodedSliceDataPartitionC,
            5 => RBSP::CodedSliceIDRPicture,
            6 => RBSP::SupplementalEnhancementInformation,
            7 => RBSP::SequenceParameterSet,
            8 => RBSP::PictureParameterSet,
            9 => RBSP::AccessUnitDelimiter,
            10 => RBSP::SequenceEnd,
            11 => RBSP::StreamEnd,
            12 => RBSP::FillerData,
            13 => RBSP::SequenceParameterSetExtension,
            14 => RBSP::PrefixNALUnit,
            15 => RBSP::SubsetSequenceParameterSet,
            16 => RBSP::DepthParameterSet,
            17 | 18 | 22 | 23 => RBSP::Reserved,
            19 => RBSP::CodedSliceAuxiliaryCodedPictureNonPartitioning,
            20 => RBSP::CodedSliceExtension,
            21 => RBSP::CodedSliceExtensionDepthViewComponent,
            _ => panic!("unrecognized nal_unit_type: {nal_unit_type}"),
        }
    }
}
