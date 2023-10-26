use ssh2::{Channel, Session};
use std::collections::HashMap;
use std::error::Error;
use std::io::Read;

struct FileSystemInfo {
    mount_point: String,
    used: u64,
    free: u64,
}

struct NetIntfInfo {
    ipv4: String,
    ipv6: String,
    rx: u64,
    tx: u64,
}

struct CpuRaw {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    soft_irq: u64,
    steal: u64,
    guest: u64,
    total: u64,
}

#[derive(Default)]
struct CpuInfo {
    user: f32,
    nice: f32,
    system: f32,
    idle: f32,
    iowait: f32,
    irq: f32,
    soft_irq: f32,
    steal: f32,
    guest: f32,
}

#[derive(Default)]
pub struct Stats {
    pub uptime: f64,
    pub hostname: String,
    load1: String,
    load5: String,
    load10: String,
    running_procs: String,
    total_procs: String,
    mem_total: u64,
    mem_free: u64,
    mem_buffers: u64,
    mem_cached: u64,
    swap_total: u64,
    swap_free: u64,
    fs_infos: Vec<FileSystemInfo>,
    net_intf: HashMap<String, NetIntfInfo>,
    cpu: CpuInfo,
}

impl Stats {
    pub fn get_all_stats(&mut self, session: &mut Session) -> Result<(), Box<dyn Error>> {
        let mut channel = session.channel_session()?;
        self.get_uptime(&mut channel)?;
        let mut channel = session.channel_session()?;
        self.get_hostname(&mut channel)?;
        Ok(())
    }

    fn get_uptime(&mut self, channel: &mut Channel) -> Result<(), Box<dyn Error>> {
        channel.exec("/bin/cat /proc/uptime")?;
        let mut parts = String::new();
        channel.read_to_string(&mut parts)?;
        let uptime_vec = parts.trim_end().split(' ').collect::<Vec<_>>();
        if uptime_vec.len() == 2 {
            self.uptime = uptime_vec[0].parse::<f64>()?;
            println!("{:?}", self.uptime);
        }
        let _ = channel.wait_close();
        Ok(())
    }

    fn get_hostname(&mut self, channel: &mut Channel) -> Result<(), Box<dyn Error>> {
        channel.exec("/bin/hostname -f")?;
        channel.read_to_string(&mut self.hostname)?;
        let _ = channel.wait_close();
        Ok(())
    }
}
