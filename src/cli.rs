use crate::sshconnect::*;
use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    /// Optional argument
    /// PEM-encoded private key file to use (default: ~/.ssh/id_rsa if present)
    #[arg(short, long, value_name = "private_key_path")]
    pub private_key_file: Option<String>,

    /// The SSH server to connect to
    /// user@host:port
    #[arg(long, value_name = "hostname")]
    pub hostname: String,

    /// Optional argument
    /// Password for ssh connection
    #[arg(long, value_name = "password")]
    pub password: Option<String>,

    /// interval
    #[arg(short, long, value_name = "interval")]
    pub interval: usize,
}

pub fn validate_parameters(cli: &Cli) -> SshConnection {
    let parsed_host = cli.hostname.split('@').collect::<Vec<_>>();
    if parsed_host.len() != 2 {
        panic!("Please provide proper host! user@host:port");
    }
    let parsed_ip_port = parsed_host[1].split(':').collect::<Vec<_>>();
    if parsed_ip_port.len() != 2 {
        panic!("Please provide proper ip and port! user@host:port");
    }
    if parsed_ip_port[1].parse::<usize>().is_err() {
        panic!("Please provide proper port!");
    }
    let parsed_ip = parsed_ip_port[0].split('.').collect::<Vec<_>>();
    if parsed_ip.len() != 4 {
        panic!("Please provide proper ip! x.x.x.x");
    }
    let proper_digit_count = parsed_ip
        .iter()
        .filter(|digit| digit.parse::<usize>().is_ok())
        .count();

    if proper_digit_count != 4 {
        panic!("Please provide proper ip with integers! ex: 127.0.0.1");
    }

    let user = parsed_host[0];
    let hostname = parsed_host[1];
    let interval = cli.interval;

    if let Some(private_key_path) = cli.private_key_file.as_deref() {
        return SshConnection::new(
            user,
            hostname,
            None,
            Some(private_key_path),
            interval,
            ConnectionType::SshPrivateKey,
        );
    } else if let Some(password) = cli.password.as_deref() {
        return SshConnection::new(
            user,
            hostname,
            Some(password),
            None,
            interval,
            ConnectionType::SshPassword,
        );
    } else {
        return SshConnection::new(
            user,
            hostname,
            None,
            None,
            interval,
            ConnectionType::SshAgent,
        );
    }
}
