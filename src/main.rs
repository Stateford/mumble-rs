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

const MUMBLE_IP: &str = env!("MUMBLE_IP");
const MUMBLE_USERNAME: &str = env!("MUMBLE_USERNAME");
const MUMBLE_PASSWORD: &str = env!("MUMBLE_PASSWORD");

#[tokio::main]
async fn main() -> MumbleResult<()> {

    MumbleClient::new(MUMBLE_IP).await?
        .listen(
            MUMBLE_USERNAME, 
            Some(MUMBLE_PASSWORD.to_owned()), 
            None, 
            false
        ).await?;

    Ok(())
}
