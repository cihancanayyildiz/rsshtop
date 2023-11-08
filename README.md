# rsshtop

`rsshtop` is a remote system monitor. It connects over SSH to a remote system.
Displays system metrics (CPU, disk, memory, network).
Only SSH server and working credentials needed on remote system.

Only Linux systems can be monitored.

## build & run

`rsshtop` written in Rust.

[Install rust](https://www.rust-lang.org/tools/install)

cargo run -- --hostname user@127.0.0.1:22 --password {password_here} -i {interval_here}
