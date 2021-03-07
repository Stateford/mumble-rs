use crate::common::MumbleResult;
use crate::socket::Socket;
use crate::mumbleproto::*;
use crate::packet::MessageType;

use std::time::Instant;

const MUMBLE_VERSION: u32 = 0x1219;
const CLIENT_NAME: &str = "mumble-rs";
const CLIENT_VERSION: &str = "0.0.1"; // TODO: set to cargo version

pub struct MumbleClient {
    socket: Socket,
    last_ping: Instant
}

impl MumbleClient {

    pub async fn new(ip_address: &str) -> MumbleResult<Self> {
        Ok(Self {
            socket: Socket::connect(ip_address).await?,
            last_ping: Instant::now()
        })
    }

    async fn authenticate(
        &mut self,
        username: &str,
        password: Option<String>,
        tokens: Option<Vec<String>>,
        opus: bool
    ) -> MumbleResult<()> {

        let version = Version {
            version: Some(MUMBLE_VERSION),
            os: Some(CLIENT_NAME.to_string()),
            os_version: Some(CLIENT_VERSION.to_string()),
            release: None
        };

        self.socket.write_message(MessageType::Version, &version).await?;

        let token = match tokens {
            Some(result) => result,
            None => Vec::new()
        };

        let authenticate = Authenticate {
            username: Some(username.to_string()),
            password,
            tokens: token,
            opus: Some(opus),
            celt_versions: Vec::new()
        };

        self.socket.write_message(MessageType::Authenticate, &authenticate).await?;

        Ok(())
    }

    async fn ping(&mut self) -> MumbleResult<()> {
        let ping_message = Ping::default();
        self.socket.write_message(MessageType::Ping, &ping_message).await?;
        self.last_ping = Instant::now();

        Ok(())
    }

    pub async fn listen(
        &mut self,
        username: &str,
        password: Option<String>,
        tokens: Option<Vec<String>>,
        opus: bool
    ) -> MumbleResult<()> {

        self.authenticate(username, password, tokens, opus).await?;

        while let Ok(packet) = self.socket.read_packet().await {

            let wait_ping = if self.last_ping.elapsed().as_secs() >= 20 {
                Some(self.ping())
            } else {
                None
            };

            // TODO: message handler

            match packet.message_type() {
                MessageType::ChannelState => {
                    let channel_state: ChannelState = packet.to_message()?;
                    if let Some(name) = channel_state.name {
                        println!("Name: {}", name);
                    }
                },
                MessageType::UserState => {
                    let user_state: UserState = packet.to_message()?;
                    if let Some(name) = user_state.name {
                        println!("Name: {}", name);
                    }
                },
                _ => {}
            }

            if let Some(wait_ping) = wait_ping {
                wait_ping.await?;
            }
        }

        Ok(())
    }
}
