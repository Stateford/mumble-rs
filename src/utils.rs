pub struct BufferParser<'a> {
    data: &'a [u8],
    position: usize
}

impl<'a> BufferParser<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            data: buffer,
            position: 0
        }
    }

    pub fn reset(&mut self) {
        self.position = 0;
    }

    pub fn set_position(&mut self, position: usize) {
        self.position = position;
    }

    pub fn read(&mut self, count: usize) -> &[u8] {

        let mut end_position = self.position + count;
        if end_position > self.data.len() {
            end_position = self.data.len();
        }

        let buffer = &self.data[self.position..end_position];
        self.position = end_position;

        return buffer;
    }

    pub fn read_until_end(&mut self) -> &[u8] {
        let buffer = &self.data[self.position..];
        self.position = self.data.len();
        return buffer;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn test_util_buffer() {
        let data: Vec<u8> = vec![0x0, 0x0, 0x0, 0x0, 0x0, 0x2, 0x32, 0x33];
        let mut buffer_parser = BufferParser::new(&data);

        let message_type = u16::from_be_bytes(buffer_parser.read(2).try_into().unwrap());
        let message_size = u32::from_be_bytes(buffer_parser.read(4).try_into().unwrap());
        let data: Vec<u8> = buffer_parser.read(message_size as usize).to_vec();

        assert_eq!(message_type, 0);
        assert_eq!(message_size, 2);
        assert_eq!(data, vec![0x32, 0x33]);
    }
}