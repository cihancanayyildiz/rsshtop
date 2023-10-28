use crate::sshconnect::ConnectionType;
use crate::stats::*;
use clap::Parser;
use crossbeam_channel::{bounded, select, tick, Receiver};
use ssh2::Session;
use std::env;
use std::net::TcpStream;
use std::time::Duration;

mod cli;
mod sshconnect;
mod stats;
fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}
fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    let cli = cli::Cli::parse();
    let ssh_connection = cli::validate_parameters(&cli);
    let tcp = TcpStream::connect(ssh_connection.hostname);
    match tcp {
        Ok(tcp) => {
            let session = Session::new();
            match session {
                Ok(mut session) => {
                    session.set_tcp_stream(tcp);
                    session.handshake().expect("Handshake failed!");

                    let auth: Result<(), ssh2::Error> = match ssh_connection.connection_type {
                        ConnectionType::Agent => session.userauth_agent(ssh_connection.hostname),
                        ConnectionType::PrivateKey => session.userauth_pubkey_file(
                            ssh_connection.user,
                            None,
                            std::path::Path::new(ssh_connection.private_key_path.unwrap()),
                            None,
                        ),
                        ConnectionType::Password => session.userauth_password(
                            ssh_connection.user,
                            ssh_connection.password.unwrap(),
                        ),
                    };
                    match auth {
                        Ok(_) => {
                            let ctrl_c_events = ctrl_channel().unwrap();
                            let ticks = tick(Duration::from_secs(cli.interval as u64));
                            let mut stats = Stats::default();
                            loop {
                                select! {
                                    recv(ticks) -> _ => {
                                        if let Ok(()) = stats.get_all_stats(&session){
                                            println!("{}",stats);
                                        };
                                    }
                                    recv(ctrl_c_events) -> _ => {
                                        println!("Goodbye!");
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprint!("Authentication failed: {}", e);
                        }
                    }
                }
                Err(e) => eprint!("Failed to create session: {}", e),
            };
        }
        Err(e) => {
            eprint!("Failed to connect: {}", e);
        }
    };
}
