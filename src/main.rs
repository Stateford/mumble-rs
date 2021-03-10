extern crate tokio;
extern crate openssl;
extern crate tokio_openssl;
extern crate prost;

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

use common::MumbleResult;
use mumble::MumbleClient;

static MUMBLE_IP: &'static str = "66.232.124.123:64738";
static MUMBLE_USERNAME: &'static str = "imnotabot";
static MUMBLE_PASSWORD: &'static str = "ye";

static CLIENT_NAME: &'static str = "mumble-rs";
static CLIENT_VERSION: &'static str = "0.0.1"; // TODO: set to cargo version

#[tokio::main]
async fn main() -> MumbleResult<()> {

    let mut client = MumbleClient::new(MUMBLE_IP).await?;
    client.set_client_info(Some(CLIENT_NAME), Some(CLIENT_VERSION));
    client.set_username(MUMBLE_USERNAME);
    client.set_password(Some(MUMBLE_PASSWORD));
    client.authenticate(None, false).await?;
    client.listen().await?;

    Ok(())
}
