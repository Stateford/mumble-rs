use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use openssl::ssl::{SslMethod, SslVerifyMode, SslConnector};
use tokio_openssl::SslStream;
use prost::Message;

use std::convert::TryInto;
use std::pin::Pin;

use crate::common::MumbleResult;
use crate::packet::{Packet, PacketHeader, MessageType};

pub struct Socket {
    stream: SslStream<TcpStream>
}

impl Socket {
    pub async fn connect(ip_address: &str) -> MumbleResult<Socket> {
        let mut connector = SslConnector::builder(SslMethod::tls())?;
        connector.set_verify(SslVerifyMode::NONE);
        // connector.set_ca_file("tests/cert.pem")?;
        let ssl = connector.build()
            .configure()?
            .into_ssl("localhost")?;

        let stream = TcpStream::connect(ip_address).await?;
        let mut stream = SslStream::new(ssl, stream).unwrap();

        Pin::new(&mut stream).connect().await?;

        let socket = Socket {stream};

        Ok(socket)
    }

    pub async fn write_message<T: Message>(&mut self, message_type: MessageType, message: &T) -> MumbleResult<()> {

        let version_packet = Packet::from_message(message_type, message)?;
        self.write_packet(&version_packet).await?;

        Ok(())
    }

    pub async fn write_packet(&mut self, packet: &Packet) -> MumbleResult<()> {
        self.stream.write_all(&packet.to_bytes()).await?;
        Ok(())
    }

    pub async fn read_packet(&mut self) -> MumbleResult<Packet> {

        let packet_header: PacketHeader = self.read_packet_header().await?;
        let mut buffer = vec![0u8; packet_header.packet_size() as usize];
        self.stream.read_exact(&mut buffer).await?;

        Ok(Packet::from_data(packet_header, &buffer)?)
    }

    async fn read_packet_header(&mut self) -> MumbleResult<PacketHeader> {

        let mut buffer: [u8; 6] = [0; 6];
        self.stream.read_exact(&mut buffer).await?;

        let packet_header: PacketHeader = (&buffer).try_into()?;

        Ok(packet_header)
    }
}
