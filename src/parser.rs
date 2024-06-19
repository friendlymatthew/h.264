
pub(crate) struct Parser<'a> (&'a [u8]);
impl Parser {
    pub(crate) fn new(data: &[u8]) -> Self {
        Self(data)
    }

    pub(crate) fn parse_nal_unit_headers(&self, )
}