use std::sync::Arc;
use std::convert::TryInto;

use crate::common::{MumbleResult, MumbleFuture};
use crate::packet::{Packet, PacketHeader, MessageType};

use tokio::{io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt}, net::tcp::{OwnedReadHalf,OwnedWriteHalf}};
use prost::Message;



pub async fn write_message<T: AsyncWriteExt + Unpin, S: Message>(stream: &mut T, message_type: MessageType, message: &S) -> MumbleResult<()> {

    let version_packet = Packet::from_message(message_type, message)?;
    write_packet(stream, &version_packet).await?;

    Ok(())
}

pub async fn write_packet<T: AsyncWriteExt + Unpin>(stream: &mut T, packet: &Packet) -> MumbleResult<()> {
    stream.write_all(&packet.to_bytes()).await?;
    Ok(())
}

pub async fn read_packet<T: AsyncReadExt + Unpin>(stream: &mut T) -> MumbleFuture<Packet> {

    let packet_header = read_packet_header(stream).await.unwrap();
    let mut buffer = vec![0u8; packet_header.packet_size() as usize];
    stream.read_exact(&mut buffer).await.unwrap();

    Ok(Packet::from_data(packet_header, &buffer).unwrap())
}

async fn read_packet_header<T: AsyncReadExt + Unpin>(stream: &mut T) -> MumbleResult<PacketHeader> {

    let mut buffer: [u8; 6] = [0; 6];
    stream.read_exact(&mut buffer).await?;

    let packet_header: PacketHeader = (&buffer).try_into()?;

    Ok(packet_header)
}