const VARINT_7: u8 = 0b0111_1111;
const VARINT_32: u8 = 0b1111_0000;
const VARINT_64: u8 = 0b1111_0100;
const MAX_PACKET_SIZE: usize = 1020;

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum AudioPacketType {
    CELT_Alpha = 0b0000_0000,
    Ping = 0b0010_0000,
    Speex = 0b0100_0000,
    CELT_Beta = 0b0110_0000,
    OPUS = 0b1000_0000
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum AudioPacketTarget {
    NormalTalking = 0,
    RecvWhisper = 1,
    DirectWhisper = 2,
    ServerLoopback = 31
}

pub trait UdpPacket {

    fn encode_varint_u8(value: u8) -> Vec<u8> {
        return vec![VARINT_7 & value];
    }

    // fn encode_varint_u64(value: u64) -> &[u8] {
    //     return &[VARINT_64, value];
    // }

    // fn encode_varint_u32(value: u32) -> &[u8] {
    //     return &[VARINT_32, value];
    // }

    fn to_bytes(&self) -> Vec<u8>;
}

pub struct AudioPacket {
    packet_type: AudioPacketType,
    packet_target: AudioPacketTarget,
    session_id: u64,
    sequence_number: u8,
    payload: Vec<u8>,
    positional_info: [f64; 3]
}

impl AudioPacket {
    pub fn set_session_id(&mut self, session_id: u64) {
        self.session_id = session_id;
    }
}

impl UdpPacket for AudioPacket {

    fn to_bytes(&self) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();

        packet.push(self.packet_type as u8 | self.packet_target as u8);
        // packet.extend_from_slice(self.encode_varint_u64(self.session_id));
        // packet.extend_from_slice(self.encode_varint_u64(self.sequence_number));
        packet.extend_from_slice(&self.payload);
        // packet.extend_from_slice(&self.positional_info);

        packet
    }
}



struct AudioPingPacket {
    header: u8,
    timestamp: u64
}