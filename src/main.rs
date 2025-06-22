use crate::net::packet::c2s::RegistrationSex;
use crate::{client::Client, net::packet::c2s::RegistrationFormMessage};
use ctrlc;
use std::sync::mpsc::channel;

pub mod arena_settings;
pub mod checksum;
pub mod client;
pub mod clock;
pub mod map;
pub mod math;
pub mod net;
pub mod player;
pub mod ship;
pub mod weapon;

fn main() -> anyhow::Result<()> {
    let (tx, rx) = channel();

    let _ = ctrlc::set_handler(move || {
        let _ = tx.send(());
    });

    // TODO: Load these from config
    let username = "test";
    let password = "none";
    let zone = "local";
    let remote_ip = "127.0.0.1";
    let remote_port = 5000;

    let registration = RegistrationFormMessage::new(
        "puppet",
        "puppet@puppet.com",
        "puppet city",
        "puppet state",
        RegistrationSex::Female,
        20,
    );

    let mut client = Client::new(
        username,
        password,
        zone,
        remote_ip,
        remote_port,
        registration,
    )?;

    client.run(rx)?;

    Ok(())
}
