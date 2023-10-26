use crate::sshconnect::ConnectionType;
use crate::stats::*;
use clap::Parser;
use ssh2::Session;
use std::net::TcpStream;

mod cli;
mod sshconnect;
mod stats;

fn main() {
    let cli = cli::Cli::parse();
    let ssh_connection = cli::validate_parameters(&cli);
    let tcp = TcpStream::connect(ssh_connection.hostname);
    match tcp {
        Ok(tcp) => {
            println!("Connected to the server");
            let session = Session::new();
            match session {
                Ok(mut session) => {
                    session.set_tcp_stream(tcp);
                    session.handshake().expect("Handshake failed!");

                    let auth: Result<(), ssh2::Error>;
                    match ssh_connection.connection_type {
                        ConnectionType::SshAgent => {
                            auth = session.userauth_agent(ssh_connection.hostname);
                        }
                        ConnectionType::SshPrivateKey => {
                            auth = session.userauth_pubkey_file(
                                ssh_connection.user,
                                None,
                                std::path::Path::new(ssh_connection.private_key_path.unwrap()),
                                None,
                            )
                        }
                        ConnectionType::SshPassword => {
                            auth = session.userauth_password(
                                ssh_connection.user,
                                ssh_connection.password.unwrap(),
                            );
                        }
                    }
                    match auth {
                        Ok(_) => {
                            println!("{}", session.authenticated());
                            let mut stats = Stats::default();
                            match stats.get_all_stats(&mut session) {
                                Ok(()) => {
                                    println!("hostname: {}", stats.hostname)
                                }
                                Err(e) => eprint!("Error: {}", e),
                            };
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
