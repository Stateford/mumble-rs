use crate::common::MumbleResult;
use crate::mumbleproto::*;
use crate::packet::{MessageType, Packet};
use crate::socket::{read_packet, write_message, write_packet};

use tokio::{io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt}, net::TcpStream, sync::{mpsc, Mutex}};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use openssl::ssl::{SslMethod, SslVerifyMode, SslConnector};
use tokio_openssl::SslStream;
use tokio::sync::oneshot;
use tokio::sync::oneshot::{Receiver, Sender};
use std::{future, pin::Pin, sync::Arc, thread::sleep, time::{Instant, Duration}};

const MUMBLE_VERSION: u32 = 0x1219;

enum MessageQueue {
    Ping,
    PacketRecieved {
        packet: Packet
    }
}

pub struct MumbleClient {
    stream: SslStream<TcpStream>,
    client_name: Option<String>,
    client_version: Option<String>,
    username: String,
    password: Option<String>
}

impl MumbleClient {

    pub async fn new(ip_address: &str) -> MumbleResult<Self> {

        let mut connector = SslConnector::builder(SslMethod::tls())?;
        connector.set_verify(SslVerifyMode::NONE);
        // connector.set_ca_file("tests/cert.pem")?;
        let ssl = connector.build()
            .configure()?
            .into_ssl("localhost")?;

        let tcp_stream = TcpStream::connect(ip_address).await?;
        let mut stream = SslStream::new(ssl, tcp_stream)?;

        Pin::new(&mut stream).connect().await?;

        Ok(Self {
            stream,
            client_name: None,
            client_version: None,
            username: String::new(),
            password: None,
        })
    }

    pub fn set_username(&mut self, username: &str) -> &mut Self {
        self.username = username.to_owned();
        self
    }

    pub fn set_password(&mut self, password: Option<&str>) -> &mut Self {
        self.password = match password {
            Some(password) => Some(password.to_owned()),
            None => None
        };

        self
    }

    pub fn set_client_info(&mut self, client_name: Option<&str>, client_version: Option<&str>) -> &mut Self {

        self.client_name = match client_name {
            Some(client_name) => Some(client_name.to_owned()),
            None => None
        };

        self.client_version = match client_version {
            Some(client_version) => Some(client_version.to_owned()),
            None => None
        };

        self
    }

    pub async fn authenticate(
        &mut self,
        tokens: Option<Vec<String>>,
        opus: bool
    ) -> MumbleResult<&mut Self> {

        let version = Version {
            version: Some(MUMBLE_VERSION),
            os: self.client_name.clone(),
            os_version: self.client_version.clone(),
            release: None
        };
        write_message(&mut self.stream, MessageType::Version, &version).await?;

        let token = match tokens {
            Some(result) => result,
            None => Vec::new()
        };

        let authenticate = Authenticate {
            username: Some(self.username.clone()),
            password: self.password.clone(),
            tokens: token,
            opus: Some(opus),
            celt_versions: Vec::new()
        };
        write_message(&mut self.stream, MessageType::Authenticate, &authenticate).await?;

        Ok(self)
    }

    async fn ping<S: AsyncWriteExt + Unpin>(stream: &mut S) -> MumbleResult<Instant> {
        let ping_message = Ping::default();

        write_message(stream, MessageType::Ping, &ping_message).await?;

        println!("pinging!");

        Ok(Instant::now())
    }

    pub async fn listen(
        self,
    ) -> MumbleResult<()> {

        let (mut reader, mut writer) = tokio::io::split(self.stream);

        let (tx, rx) = mpsc::channel::<MessageQueue>(1);
        let tx = Arc::new(Mutex::new(tx));
        let rx = Arc::new(Mutex::new(rx));

        let t1tx = tx.clone();
        let t2tx = tx.clone();
        let t3rx = rx.clone();

        let t1 = tokio::spawn(async move {

            let mut last_ping_time = Instant::now();
            let tx = t1tx.clone();

            loop {
                if last_ping_time.elapsed().as_secs() >= 20 {

                    let tx = tx.lock().await;
                    tx.send(MessageQueue::Ping).await.unwrap_or_default();

                    last_ping_time = Instant::now();
                }
            }
        });

        let t2 = tokio::spawn(async move {
            let tx = t2tx.clone();
            loop {

                match read_packet(&mut reader).await {
                    Ok(packet) => {
                        let tx = tx.lock().await;
                        tx.send(MessageQueue::PacketRecieved { packet}).await.unwrap_or_default();
                    },
                    _ => {}
                };
            }
        });

        let t3 = tokio::spawn(async move {
            let rx = t3rx.clone();
            loop {

                let message = {
                    let mut rx = rx.lock().await;

                    match rx.recv().await {
                        Some(message) => message,
                        None => continue
                    }
                };

                match message {
                    MessageQueue::Ping => { Self::ping(&mut writer).await.unwrap(); },
                    MessageQueue::PacketRecieved { packet} => {
                        match packet.message_type() {
                            MessageType::ChannelState => {
                                let channel_state: ChannelState = packet.to_message().unwrap();
                                if let Some(name) = channel_state.name {
                                    println!("Name: {}", name);
                                }
                            },
                            MessageType::UserState => {
                                let user_state: UserState = packet.to_message().unwrap();
                                if let Some(name) = user_state.name {
                                    println!("Name: {}", name);
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }
        });

        let(a, b, c) = tokio::join!(t1, t2, t3);
        a.unwrap();
        b.unwrap();
        c.unwrap();

        Ok(())
    }
}
