use std::convert::TryInto;

use crate::common::{MumbleResult, MumbleFuture};
use crate::packet::{Packet, PacketHeader, MessageType};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use prost::Message;



pub struct SocketWriter<T: AsyncWriteExt + Unpin> {
    stream: T
}

impl<T: AsyncWriteExt + Unpin> SocketWriter<T> {

    pub fn new(stream: T) -> Self {
        Self {
            stream
        }
    }

    pub async fn write_message<S: Message>(&mut self, message_type: MessageType, message: &S) -> MumbleResult<()> {

        let version_packet = Packet::from_message(message_type, message)?;
        self.write_packet(&version_packet).await?;

        Ok(())
    }

    pub async fn write_packet(&mut self, packet: &Packet) -> MumbleResult<()> {
        self.stream.write_all(&packet.to_bytes()).await?;
        Ok(())
    }
}

pub struct SocketReader<T: AsyncReadExt + Unpin> {
    stream: T
}

impl<T: AsyncReadExt + Unpin> SocketReader<T> {

    pub fn new(stream: T) -> Self {
        Self {
            stream
        }
    }

    pub async fn read_packet(&mut self) -> MumbleFuture<Packet> {

        let packet_header = self.read_packet_header().await.unwrap();
        let mut buffer = vec![0u8; packet_header.packet_size() as usize];
        self.stream.read_exact(&mut buffer).await.unwrap();

        Ok(Packet::from_data(packet_header, &buffer).unwrap())
    }

    async fn read_packet_header(&mut self) -> MumbleResult<PacketHeader> {

        let mut buffer: [u8; 6] = [0; 6];

        // TODO: fix this not reading enough bytes?
        self.stream.read_exact(&mut buffer).await?;

        let packet_header: PacketHeader = (&buffer).try_into()?;

        Ok(packet_header)
    }
}