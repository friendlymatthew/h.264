use std::fs::File;

use memmap::Mmap;

use crate::byte_stream::ByteStream;
use crate::errors::DecodingError;

pub struct H264Decoder {
    data: Mmap,
}

impl H264Decoder {
    pub fn from_file(file: File) -> Result<Self, DecodingError> {
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self { data: mmap })
    }

    pub fn from_file_path(file_path: &str) -> Result<Self, DecodingError> {
        let file = File::open(file_path)?;
        H264Decoder::from_file(file)
    }

    pub fn decode(&mut self) -> Result<(), DecodingError> {
        let mut byte_stream = ByteStream::new(&self.data);

        /// Input to this process consists of an ordered stream of bytes consisting of a sequence of
        /// byte stream NAL unit syntax structures.
        ///
        /// Output of this process consists of a sequence of NAL unit syntax structures.
        let nal_units = byte_stream.process()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading_from_gravity() -> Result<(), DecodingError> {
        let mut decoder = H264Decoder::from_file_path("../gravity.h264")?;
        decoder.decode()?;

        Ok(())
    }
}
