pub enum ConnectionType {
    Agent,
    PrivateKey,
    Password,
}

pub struct SshConnection<'a> {
    pub user: &'a str,
    pub hostname: &'a str, // host:port
    pub password: Option<&'a str>,
    pub private_key_path: Option<&'a str>,
    pub interval: usize,
    pub connection_type: ConnectionType,
}

impl<'a> SshConnection<'a> {
    pub fn new(
        user: &'a str,
        hostname: &'a str,
        password: Option<&'a str>,
        private_key_path: Option<&'a str>,
        interval: usize,
        connection_type: ConnectionType,
    ) -> Self {
        SshConnection {
            user,
            hostname,
            password,
            private_key_path,
            interval,
            connection_type,
        }
    }
}
