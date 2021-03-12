extern crate tokio;
extern crate openssl;
extern crate tokio_openssl;
extern crate prost;
extern crate bytes;

pub mod mumbleproto {
    include!(concat!(env!("OUT_DIR"), "/mumble.rs"));
}

mod common;
mod utils;
mod errors;
mod packet;
mod socket;
mod mumble;
mod ping;
mod channel;
mod voice;

use common::MumbleResult;
use mumble::MumbleClient;


#[tokio::main]
async fn main() -> MumbleResult<()> {

    let mut client = MumbleClient::new(MUMBLE_IP).await?;
    client.set_client_info(Some(CLIENT_NAME), Some(CLIENT_VERSION));
    client.set_username(MUMBLE_USERNAME).await;
    client.set_password(Some(MUMBLE_PASSWORD));
    client.authenticate(None, false).await?;
    client.listen().await?;

    client.set_comment("HELLO").await?;

    let channels = client.get_channels().await;
    if let Some(channel) = channels.find("Sp3cs personal AFK channel") {
        println!("Joining channel!");
        client.join_channel(channel).await?;
    }

    // client.send_message("testing").await?;

    client.send_image(FILE_TO_SEND).await?;

    loop {}


    Ok(())
}
