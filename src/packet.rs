use crate::common::MumbleResult;
use crate::errors::MumbleError;
use crate::utils::BufferParser;

use std::convert::{TryFrom, TryInto};
use prost::Message;

#[repr(u16)]
#[derive(Debug, Clone)]
pub enum MessageType {
    Version = 0,
    UDPTunnel = 1,
    Authenticate = 2,
    Ping = 3,
    Reject = 4,
    ServerSync = 5,
    ChannelRemove = 6,
    ChannelState = 7,
    UserRemove = 8,
    UserState = 9,
    BanList = 10,
    TextMessage = 11,
    PermissionDenied = 12,
    ACL = 13,
    QueryUsers = 14,
    CryptSetup = 15,
    ContextActionModify = 16,
    ContextAction = 17,
    UserList = 18,
    VoiceTarget = 19,
    PermissionQuery = 20,
    CodecVersion = 21,
    UserStats = 22,
    RequestBlob = 23,
    ServerConfig = 24,
    SuggestConfig = 25
}

impl TryFrom<u16> for MessageType {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: u16) -> Result<MessageType, Self::Error> {
        match value {
            x if x == MessageType::Version as u16 => Ok(MessageType::Version),
            x if x == MessageType::UDPTunnel as u16 => Ok(MessageType::UDPTunnel),
            x if x == MessageType::Authenticate as u16 => Ok(MessageType::Authenticate),
            x if x == MessageType::Ping as u16 => Ok(MessageType::Ping),
            x if x == MessageType::Reject as u16 => Ok(MessageType::Reject),
            x if x == MessageType::ServerSync as u16 => Ok(MessageType::ServerSync),
            x if x == MessageType::ChannelRemove as u16 => Ok(MessageType::ChannelRemove),
            x if x == MessageType::ChannelState as u16 => Ok(MessageType::ChannelState),
            x if x == MessageType::UserRemove as u16 => Ok(MessageType::UserRemove),
            x if x == MessageType::UserState as u16 => Ok(MessageType::UserState),
            x if x == MessageType::BanList as u16 => Ok(MessageType::BanList),
            x if x == MessageType::TextMessage as u16 => Ok(MessageType::TextMessage),
            x if x == MessageType::PermissionDenied as u16 => Ok(MessageType::PermissionDenied ),
            x if x == MessageType::ACL as u16 => Ok(MessageType::ACL),
            x if x == MessageType::QueryUsers as u16 => Ok(MessageType::QueryUsers),
            x if x == MessageType::CryptSetup as u16 => Ok(MessageType::CryptSetup),
            x if x == MessageType::ContextActionModify as u16 => Ok(MessageType::ContextActionModify),
            x if x == MessageType::ContextAction as u16 => Ok(MessageType::ContextAction),
            x if x == MessageType::UserList as u16 => Ok(MessageType::UserList),
            x if x == MessageType::VoiceTarget as u16 => Ok(MessageType::VoiceTarget),
            x if x == MessageType::PermissionQuery as u16 => Ok(MessageType::PermissionQuery),
            x if x == MessageType::CodecVersion as u16 => Ok(MessageType::CodecVersion),
            x if x == MessageType::UserStats as u16 => Ok(MessageType::UserStats),
            x if x == MessageType::RequestBlob as u16 => Ok(MessageType::RequestBlob),
            x if x == MessageType::ServerConfig as u16 => Ok(MessageType::ServerConfig),
            x if x == MessageType::SuggestConfig as u16 => Ok(MessageType::SuggestConfig),
            _ => Err(Box::new(MumbleError::new("Could not convert enum")))
        }
    }
}

pub struct PacketHeader {
    message_type: MessageType,
    packet_size: u32
}

impl PacketHeader {

    pub fn message_type(&self) -> MessageType {
        self.message_type.clone()
    }

    pub fn packet_size(&self) -> u32 {
        self.packet_size
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();
        packet.extend_from_slice(&(self.message_type.clone() as u16).to_be_bytes());
        packet.extend_from_slice(&self.packet_size.to_be_bytes());

        return packet;
    }
}

impl TryFrom<&[u8; 6]> for PacketHeader {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &[u8; 6]) -> Result<PacketHeader, Self::Error> {

        // packet header must be 8 bytes
        if value.len() != 6 {
            return Err(Box::new(MumbleError::new("Packet header length was incorrect")));
        }

        let mut parser = BufferParser::new(value);
        let message_type: MessageType = u16::from_be_bytes(parser.read(2).try_into()?).try_into()?;
        let packet_size = u32::from_be_bytes(parser.read(4).try_into()?);

        Ok(PacketHeader { message_type, packet_size })
    }
}

pub struct Packet {
    header: PacketHeader,
    packet: Vec<u8>
}

unsafe impl Send for Packet {}

impl Packet {

    pub fn message_type(&self) -> MessageType {
        return self.header.message_type();
    }

    pub fn from_message<T: Message>(message_type: MessageType, message: &T) -> MumbleResult<Self> {

        let mut buffer: Vec<u8> = Vec::new(); 
        message.encode(&mut buffer)?; // TODO: change to ?
        let header = PacketHeader { message_type, packet_size: buffer.len() as u32};

        Ok(Self {
            header,
            packet: buffer
        })
    }

    pub fn from_data(header: PacketHeader, data: &[u8]) -> MumbleResult<Self> {
        Ok(Self {
            header,
            packet: data.to_owned()
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();
        packet.extend_from_slice(&(self.header.message_type() as u16).to_be_bytes());
        packet.extend_from_slice(&(self.header.packet_size() as u32).to_be_bytes());
        packet.extend_from_slice(&self.packet);
        return packet;
    }

    pub fn to_message<T: Message + Default>(&self) -> MumbleResult<T> {
        Ok(T::decode(&*self.packet)?)
    }
}



pub fn build_packet<T: Message>(message_type: MessageType, data: &T) -> MumbleResult<Vec<u8>> {

    let mut packet: Vec<u8> = Vec::new();
    let mut buffer: Vec<u8> = Vec::new(); 
    data.encode(&mut buffer)?; // TODO: change to ?

    packet.extend_from_slice(&(message_type as u16).to_be_bytes());
    packet.extend_from_slice(&(buffer.len() as u32).to_be_bytes());
    packet.extend_from_slice(&buffer);

    println!("PACKET: {:?}", packet);

    Ok(packet)
}