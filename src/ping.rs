use std::convert::TryInto;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;

use crate::common::MumbleResult;

pub struct PingRequest {
    pub request_type: u32,
    pub identity: u64
}

impl PingRequest {
    pub fn new() -> MumbleResult<Self> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
        Ok(Self {
            request_type: 0,
            identity: current_time
        })
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.extend_from_slice(&self.request_type.to_be_bytes());
        data.extend_from_slice(&self.identity.to_be_bytes());
        return data;
    }
}

pub struct PingResponse {
    pub version: u32,
    pub identity: u64,
    pub connected_users: u32,
    pub maximum_users : u32,
    pub allowed_bandwidth: u32
}

impl PingResponse {
    pub fn from_u8_buffer(buffer: &[u8]) -> MumbleResult<Self> {
        // TODO: use buffer parser here
        Ok(Self {
            version: u32::from_be_bytes(buffer[0..4].try_into()?),
            identity: u64::from_be_bytes(buffer[4..12].try_into()?),
            connected_users: u32::from_be_bytes(buffer[12..16].try_into()?),
            maximum_users: u32::from_be_bytes(buffer[16..20].try_into()?),
            allowed_bandwidth: u32::from_be_bytes(buffer[20..24].try_into()?)
        })
    }
}

pub async fn ping_mumble_server(ip_addr: &str) -> MumbleResult<PingResponse> {
    let socket = UdpSocket::bind("0.0.0.0:8000").await?;
    socket.set_broadcast(true)?;

    let data = PingRequest::new()?;
    socket.send_to(&data.to_vec(), ip_addr).await?;

    let mut buf: [u8; 256] = [0; 256];
    let (_, _) = socket.recv_from(&mut buf).await?;

    let ping_response = PingResponse::from_u8_buffer(&buf)?;
    Ok(ping_response)
}