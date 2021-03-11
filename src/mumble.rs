use crate::common::MumbleResult;
use crate::mumbleproto::*;
use crate::packet::{MessageType, Packet};
use crate::socket::{SocketReader, SocketWriter};
use crate::channel::{Channel, ChannelList};

use tokio::{net::TcpStream, task::JoinHandle};
use tokio::sync::{mpsc, mpsc::{Sender, Receiver}, Mutex};
use tokio::io::{ReadHalf, WriteHalf};
use openssl::ssl::{SslMethod, SslVerifyMode, SslConnector};
use tokio_openssl::SslStream;
use std::{pin::Pin, sync::atomic::{AtomicBool, Ordering}, time::Duration};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const MUMBLE_VERSION: u32 = 0x1219;

enum MumbleAction {
    Ping,
    MoveChannel {
        channel: Channel
    },
    SetComment {
        comment: String
    }
}

enum MessageQueue {
    Action { 
        action: MumbleAction 
    },
    PacketRecieved {
        packet: Packet
    }
}

#[derive(Default)]
struct UserInfo {
    session_id: u32,
    name: String,
    channel_id: u32
}

pub struct MumbleClient {
    client_name: Option<String>,
    client_version: Option<String>,
    username: String,
    password: Option<String>,
    reader: Arc<Mutex<SocketReader<ReadHalf<SslStream<TcpStream>>>>>,
    writer: Arc<Mutex<SocketWriter<WriteHalf<SslStream<TcpStream>>>>>,
    threads: Vec<JoinHandle<()>>,
    running: Arc<AtomicBool>,
    tx_channel: Arc<Mutex<Sender<MessageQueue>>>,
    rx_channel: Arc<Mutex<Receiver<MessageQueue>>>,
    user_info: Arc<Mutex<UserInfo>>,
    connected: Arc<AtomicBool>,
    channels: Arc<Mutex<ChannelList>>
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

        let (reader, writer) = tokio::io::split(stream);

        let (tx, rx) = mpsc::channel::<MessageQueue>(3);
        let tx = Arc::new(Mutex::new(tx));
        let rx = Arc::new(Mutex::new(rx));

        Ok(Self {
            client_name: None,
            client_version: None,
            username: String::new(),
            password: None,
            reader: Arc::new(Mutex::new(SocketReader::new(reader))),
            writer: Arc::new(Mutex::new(SocketWriter::new(writer))),
            threads: Vec::new(),
            running: Arc::new(AtomicBool::new(false)),
            rx_channel: rx,
            tx_channel: tx,
            user_info: Arc::new(Mutex::new(UserInfo::default())),
            connected: Arc::new(AtomicBool::new(false)),
            channels: Arc::new(Mutex::new(ChannelList::default()))
        })
    }

    pub async fn set_username(&mut self, username: &str) -> &mut Self {
        self.username = username.to_owned();
        let user_info = Arc::clone(&self.user_info);
        let mut user_info = user_info.lock().await;
        user_info.name = username.to_owned();
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
        let writer = Arc::clone(&self.writer);
        let mut writer = writer.lock().await;
        writer.write_message(MessageType::Version, &version).await?;

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
        writer.write_message(MessageType::Authenticate, &authenticate).await?;

        Ok(self)
    }

    async fn ping(writer: &mut SocketWriter<WriteHalf<SslStream<TcpStream>>>) -> MumbleResult<Instant> {

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() as u64;

        let mut ping_message = Ping::default();
        ping_message.timestamp = Some(timestamp);

        let ping_message = Ping {
            good: None,
            lost: None,
            resync: None,
            late: None,
            tcp_packets: None,
            tcp_ping_avg: None,
            tcp_ping_var: None,
            udp_packets: None,
            udp_ping_avg: None,
            udp_ping_var: None,
            timestamp: Some(timestamp)
        };

        writer.write_message(MessageType::Ping, &ping_message).await?;

        println!("pinging!");

        Ok(Instant::now())
    }

    pub async fn listen(
        &mut self,
    ) -> MumbleResult<()> {

        // todo: move rx / tx to message handler

        let t1tx = self.tx_channel.clone();
        let t2tx = self.tx_channel.clone();
        let t3rx = self.rx_channel.clone();

        self.running.store(true, Ordering::Relaxed);

        let t1_running = Arc::clone(&self.running);

        let t1 = tokio::spawn(async move {

            let mut last_ping_time = Instant::now();
            let tx = t1tx.clone();

            while t1_running.load(Ordering::Relaxed) {
                if last_ping_time.elapsed().as_secs() >= 10 {

                    let tx = tx.lock().await;
                    tx.send(MessageQueue::Action { action: MumbleAction::Ping }).await.unwrap_or_default();
                    last_ping_time = Instant::now();
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        let reader_ref = Arc::clone(&self.reader);

        let t2_running = Arc::clone(&self.running);

        let t2 = tokio::spawn(async move {
            let tx = t2tx.clone();
            let reader_ref = reader_ref;

            while t2_running.load(Ordering::Relaxed) {
                let mut reader = reader_ref.lock().await;

                match reader.read_packet().await {
                    Ok(packet) => {
                        let tx = tx.lock().await;
                        tx.send(MessageQueue::PacketRecieved { packet }).await.unwrap_or_default();
                    },
                    _ => {}
                };

                drop(reader);

                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });

        let writer_ref = Arc::clone(&self.writer);
        let t3_running = Arc::clone(&self.running);
        let user_info = Arc::clone(&self.user_info);

        let connected = Arc::clone(&self.connected);
        let channels = Arc::clone(&self.channels);

        let t3 = tokio::spawn(async move {
            let rx = t3rx.clone();
            let writer_ref = writer_ref.clone();
            let user_info = user_info;
            let connected = connected;

            while t3_running.load(Ordering::Relaxed) {

                let message = {
                    let mut rx = rx.lock().await;

                    match rx.recv().await {
                        Some(message) => message,
                        None => continue
                    }
                };

                match message {
                    MessageQueue::Action { action} => { 

                        match action {
                            MumbleAction::Ping => {
                                let mut writer = writer_ref.lock().await;
                                Self::ping(&mut writer).await.unwrap(); 
                            },
                            MumbleAction::MoveChannel { channel} => {
                                let user_info = user_info.lock().await;

                                let mut user_state = UserState::default();
                                user_state.session = Some(user_info.session_id);
                                user_state.name = Some(user_info.name.clone());
                                user_state.channel_id = Some(channel.id);
                                let mut writer = writer_ref.lock().await;
                                writer.write_message(MessageType::UserState, &user_state).await.unwrap();
                            },
                            MumbleAction::SetComment { comment} => {
                                let user_info = user_info.lock().await;

                                let mut user_state = UserState::default();
                                user_state.session = Some(user_info.session_id);
                                user_state.comment = Some(comment);
                                user_state.name = Some(user_info.name.clone());
                                let mut writer = writer_ref.lock().await;
                                writer.write_message(MessageType::UserState, &user_state).await.unwrap();
                            }
                        }
                    },
                    MessageQueue::PacketRecieved { packet} => {
                        match packet.message_type() {
                            MessageType::ChannelState => {
                                let channel_state: ChannelState = packet.to_message().unwrap();
                                let channel = Channel::from_message(&channel_state).unwrap();
                                let mut channels = channels.lock().await;
                                channels.push(channel).unwrap();

                                if let Some(name) = channel_state.name {
                                    println!("Name: {}", name);
                                }
                            },
                            MessageType::UserState => {
                                let user_state: UserState = packet.to_message().unwrap();
                                // println!("{:?}", user_state);
                                if let Some(name) = user_state.name {
                                    println!("Name: {}", name);
                                }
                            },
                            MessageType::Ping => {
                                let ping: Ping = packet.to_message().unwrap();
                                println!("{:?}", ping);
                            },
                            MessageType::ServerSync => {
                                if !connected.load(Ordering::Relaxed) {
                                    connected.store(true, Ordering::Relaxed);
                                }
                                let mut user_info = user_info.lock().await;
                                let server_sync: ServerSync = packet.to_message().unwrap();
                                if let Some(session_id) = server_sync.session {
                                    user_info.session_id = session_id;
                                }
                            },
                            _ => {}
                        }
                    }
                }

                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        self.threads.push(t1);
        self.threads.push(t2);
        self.threads.push(t3);

        self.wait_for_connection().await?;

        Ok(())
    }

    async fn wait_for_connection(&mut self) -> MumbleResult<()> {
        let connected = Arc::clone(&self.connected);
        while !connected.load(Ordering::Relaxed) {}

        Ok(())
    }

    pub async fn set_comment(&mut self, comment: &str) -> MumbleResult<()> {
        let tx = self.tx_channel.clone();
        let message = MessageQueue::Action {
            action: MumbleAction::SetComment {
                comment: comment.to_owned()
            }
        };
        let tx = tx.lock().await;
        tx.send(message).await.unwrap_or_default();

        Ok(())
    }

    pub async fn get_channels(&self) -> ChannelList {
        let channels = self.channels.clone();
        let channels = channels.lock().await;
        channels.clone()
    }

    pub async fn join_channel(&mut self, channel: Channel) -> MumbleResult<()> {
        let tx = self.tx_channel.clone();
        let message = MessageQueue::Action {
            action: MumbleAction::MoveChannel {
                channel
            }
        };
        let tx = tx.lock().await;
        tx.send(message).await.unwrap_or_default();

        Ok(())
    }

    pub async fn shutdown(&mut self) -> MumbleResult<()> {
        self.running.store(false, Ordering::Relaxed);

        for thread in &self.threads {
            thread.abort();
        }

        self.threads.clear();

        Ok(())
    }
}


impl Drop for MumbleClient {
    fn drop(&mut self) {

        self.running.store(false, Ordering::Relaxed);

        for thread in &self.threads {
            thread.abort();
        }
    }
}