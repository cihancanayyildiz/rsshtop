use ssh2::Session;
use std::collections::HashMap;
use std::error::Error;
use std::io::Read;

#[derive(Debug)]
pub struct FileSystemInfo {
    mount_point: String,
    used: u64,
    free: u64,
}

pub struct NetIntfInfo {
    ipv4: String,
    ipv6: String,
    rx: u64,
    tx: u64,
}

pub struct CpuRaw {
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
pub struct CpuInfo {
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
    pub load1: String,
    pub load5: String,
    pub load10: String,
    pub running_procs: String,
    pub total_procs: String,
    pub mem_total: u64,
    pub mem_free: u64,
    pub mem_buffers: u64,
    pub mem_cached: u64,
    pub swap_total: u64,
    pub swap_free: u64,
    pub fs_infos: Vec<FileSystemInfo>,
    pub net_intf: HashMap<String, NetIntfInfo>,
    pub cpu: CpuInfo,
}

impl Stats {
    pub fn get_all_stats(&mut self, session: &Session) -> Result<(), Box<dyn Error>> {
        self.get_uptime(&session)?;
        self.get_hostname(&session)?;
        self.get_load(&session)?;
        self.get_mem_info(&session)?;
        self.get_fs_info(&session)?;
        self.get_interfaces(&session)?;
        Ok(())
    }

    pub fn get_uptime(&mut self, session: &Session) -> Result<(), Box<dyn Error>> {
        let parts = run_command(session, "/bin/cat /proc/uptime")?;
        let uptime_vec = parts.trim_end().split(' ').collect::<Vec<_>>();
        if uptime_vec.len() == 2 {
            self.uptime = uptime_vec[0].parse::<f64>()?;
            println!("{:?}", self.uptime);
        }
        Ok(())
    }

    fn get_hostname(&mut self, session: &Session) -> Result<(), Box<dyn Error>> {
        self.hostname = run_command(&session, "/bin/hostname -f")?;
        Ok(())
    }

    fn get_load(&mut self, session: &Session) -> Result<(), Box<dyn Error>> {
        let parts = run_command(session, "/bin/cat /proc/loadavg")?;
        let parts_vec = parts.split(' ').collect::<Vec<_>>();

        if parts_vec.len() == 5 {
            self.load1 = parts_vec[0].to_string();
            self.load5 = parts_vec[1].to_string();
            self.load10 = parts_vec[2].to_string();
            self.running_procs = parts_vec[3].split('/').collect::<Vec<_>>()[0].to_string();
            self.total_procs = parts_vec[3].split('/').collect::<Vec<_>>()[1].to_string()
        }
        Ok(())
    }

    fn get_mem_info(&mut self, session: &Session) -> Result<(), Box<dyn Error>> {
        let parts = run_command(session, "/bin/cat /proc/meminfo")?;
        let lines = parts.split('\n').collect::<Vec<_>>();
        for line in lines.iter() {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if parts.len() == 3 {
                match parts[1].parse::<u64>() {
                    Ok(mut value) => {
                        value *= 1024;
                        match parts[0] {
                            "MemTotal:" => self.mem_total = value,
                            "MemFree:" => self.mem_free = value,
                            "Buffers:" => self.mem_buffers = value,
                            "Cached:" => self.mem_cached = value,
                            "SwapTotal:" => self.swap_total = value,
                            "SwapFree:" => self.swap_free = value,
                            &_ => continue,
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
        Ok(())
    }

    fn get_fs_info(&mut self, session: &Session) -> Result<(), Box<dyn Error>> {
        let parts = run_command(session, "/bin/df -B1")?;
        let lines = parts.split('\n').collect::<Vec<_>>();

        let mut flag = 0;
        for line in lines {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            let dev = parts.len() > 0 && parts[0].starts_with("/dev/");
            if parts.len() == 1 && dev {
                flag = 1;
            } else if (parts.len() == 5 && flag == 1) || (parts.len() == 6 && dev) {
                let temp = flag;
                flag = 0;
                let used = parts[2 - temp].parse::<u64>();
                if used.is_err() {
                    continue;
                }
                let free = parts[3 - temp].parse::<u64>();
                if free.is_err() {
                    continue;
                }

                self.fs_infos = Vec::new();
                self.fs_infos.push(FileSystemInfo {
                    mount_point: parts[5 - temp].to_string(),
                    used: used.unwrap(),
                    free: free.unwrap(),
                });
            }
        }

        Ok(())
    }

    fn get_interfaces(&mut self, session: &Session) -> Result<(), Box<dyn Error>> {
        let lines = run_command(session, "/bin/ip -o addr")?;
        println!("{}", lines);
        Ok(())
    }
}

fn run_command(session: &Session, command: &str) -> Result<String, Box<dyn Error>> {
    let mut channel = session.channel_session()?;
    let mut result = String::new();
    channel.exec(command)?;
    channel.read_to_string(&mut result)?;
    Ok(result)
}
