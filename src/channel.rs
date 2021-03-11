use crate::{common::MumbleResult, mumbleproto::ChannelState};

#[derive(Default, Clone)]
pub struct ChannelList {
    channels: Vec<Channel>
}

impl ChannelList {

    pub fn push(&mut self, channel: Channel) -> MumbleResult<()> {
        self.channels.push(channel);

        Ok(())
    }

    pub fn find(&self, name: &str) -> Option<Channel> {

        let channel = self.channels.iter()
            .find(|&x | { x.name == name });

        if let Some(channel) = channel {
            let channel = channel.clone();
            return Some(channel);
        }

        None
    }
}

#[derive(Default, Clone)]
pub struct Channel {
    pub id: u32,
    pub parent: u32,
    pub name: String,
}

impl Channel {
    pub fn from_message(message: &ChannelState) -> MumbleResult<Self> {

        let id = match message.channel_id {
            Some(id) => id,
            None => 0
        };

        let parent = match message.parent {
            Some(parent) => parent,
            None => 0
        };

        let name = match &message.name {
            Some(name) => name.clone(),
            None => String::new()
        };

        Ok(Self {
            id,
            parent,
            name
        })
    }
}