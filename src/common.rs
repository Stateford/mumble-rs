use std::future::Future;

pub type MumbleResult<T> = Result<T, Box<dyn std::error::Error>>;
pub type MumbleFuture<T> = Result<T, Box<dyn std::error::Error + Send>>;