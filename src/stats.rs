use ::time::Duration;
use colored::Colorize;
use ssh2::Session;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::io::Read;

const ESC: &str = "\x1B[2J\x1B[1;1H"; // Clears the terminal.

#[allow(dead_code)]
pub struct FileSystemInfo {
    mount_point: String,
    used: u64,
    free: u64,
}

#[derive(Debug)]
pub struct NetIntfInfo {
    ipv4: String,
    ipv6: String,
    rx: u64,
    tx: u64,
}

#[derive(Default, Debug)]
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

#[derive(Default, Debug)]
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
    pub prev_cpu: CpuRaw,
    pub cpu: CpuInfo,
}
impl Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut file_sys = String::new();
        for fs in &self.fs_infos {
            file_sys.push_str(
                format!(
                    "{}: {} free of {}\n",
                    fs.mount_point,
                    format_bytes(fs.free),
                    format_bytes(fs.used + fs.free)
                )
                .as_str(),
            );
        }

        let mut net_info = String::new();

        for (key, val) in &self.net_intf {
            net_info.push_str(format!("\t{} - {}", key, val.ipv4).as_str());
            if !val.ipv6.is_empty() {
                net_info.push_str(format!(", {}\n", val.ipv6).as_str());
            } else {
                net_info.push('\n');
            }
            net_info.push_str(
                format!(
                    "\trx = {}, tx = {}\n\n",
                    format_bytes(val.rx),
                    format_bytes(val.tx)
                )
                .as_str(),
            );
        }

        write!(
            f,
            "{}{}up {}\n\n{}\n\t{} {} {}\n\n{}\n\t{} user, {} sys, {} nice, {} idle, {} iowait, {} hardirq, {} softirq, {} guest\n\n{}\n\t{} running of {} total\n\n{}\n\tfree = {}\n\tused = {}\n\tbuffers = {}\n\tcached = {}\n\tswap = {} free of {}\n\n{}\n\t{}\n{}\n{}\n",
            ESC,
            self.hostname.bold().bright_green(),
            self.format_uptime().bold().bright_cyan(),
            "Load:".bright_yellow(),
            self.load1.bold().bright_white(),
            self.load5.bold().bright_white(),
            self.load10.bold().bright_white(),
            "CPU:".bright_yellow(),
            self.cpu.user.to_string().bold().bright_white(),
            self.cpu.system.to_string().bold().bright_white(),
            self.cpu.nice.to_string().bold().bright_white(),
            self.cpu.idle.to_string().bold().bright_white(),
            self.cpu.iowait.to_string().bold().bright_white(),
            self.cpu.irq.to_string().bold().bright_white(),
            self.cpu.soft_irq.to_string().bold().bright_white(),
            self.cpu.guest.to_string().bold().bright_white(),
            "Processes:".bright_yellow(),
            self.running_procs.bold().bright_white(),
            self.total_procs.bold().bright_white(),
            "Memory:".bright_yellow(),
            format_bytes(self.mem_free).bold().bright_white(),
            format_bytes(self.mem_total - self.mem_free - self.mem_buffers - self.mem_cached).bold().bright_white(),
            format_bytes(self.mem_buffers).bold().bright_white(),
            format_bytes(self.mem_cached).bold().bright_white(),
            format_bytes(self.swap_free).bold().bright_white(),
            format_bytes(self.swap_total).bold().bright_white(),
            "Filesystems:".bright_yellow(),
            file_sys.bold().bright_white(),
            "Network Interfaces:".bright_yellow(),
            net_info.bold().bright_white(),
        )
    }
}

impl Stats {
    pub fn get_all_stats(&mut self, session: &Session) -> Result<(), Box<dyn Error>> {
        self.get_uptime(session)?;
        self.get_hostname(session)?;
        self.get_load(session)?;
        self.get_mem_info(session)?;
        self.get_fs_info(session)?;
        self.get_interfaces(session)?;
        self.get_interface_info(session)?;
        self.get_cpu(session)?;
        Ok(())
    }

    fn get_uptime(&mut self, session: &Session) -> Result<(), Box<dyn Error>> {
        let parts = run_command(session, "/bin/cat /proc/uptime")?;
        // todo! split_whitespace
        let uptime_vec = parts.split_whitespace().collect::<Vec<_>>();
        if uptime_vec.len() == 2 {
            self.uptime = uptime_vec[0].parse::<f64>()?;
        }
        Ok(())
    }
    fn format_uptime(&self) -> String {
        let mut duration = self.uptime;
        duration = duration - (duration % Duration::SECOND.as_seconds_f64());

        let mut days = 0;
        loop {
            if duration / 3600.0 > 24.0 {
                days += 1;
                duration -= 24.0 * 60.0 * 60.0;
            } else {
                break;
            }
        }
        let mut hours = 0;
        loop {
            if duration / 3600.0 > 1.0 {
                hours += 1;
                duration -= 1.0 * 60.0 * 60.0;
            } else {
                break;
            }
        }
        let mut mins = 0;
        loop {
            if duration > 1.0 * 60.0 {
                mins += 1;
                duration -= 1.0 * 60.0;
            } else {
                break;
            }
        }
        let mut res = String::new();
        if days > 0 {
            res.push_str(format!("{days}d ").as_str());
        }
        if hours > 0 {
            res.push_str(format!("{hours}h ").as_str());
        }
        if mins > 0 {
            res.push_str(format!("{mins}m ").as_str());
        }
        res.push_str(format!("{duration}s ").as_str());
        res
    }

    fn get_hostname(&mut self, session: &Session) -> Result<(), Box<dyn Error>> {
        self.hostname = run_command(session, "/bin/hostname -f")?;
        Ok(())
    }

    fn get_load(&mut self, session: &Session) -> Result<(), Box<dyn Error>> {
        let parts = run_command(session, "/bin/cat /proc/loadavg")?;
        // todo! split_whitespace
        let parts_vec = parts.split(' ').collect::<Vec<_>>();

        if parts_vec.len() == 5 {
            self.load1 = parts_vec[0].to_string();
            self.load5 = parts_vec[1].to_string();
            self.load10 = parts_vec[2].to_string();
            self.running_procs = parts_vec[3].split('/').collect::<Vec<_>>()[0].to_string();
            self.total_procs = parts_vec[3].split('/').collect::<Vec<_>>()[1].to_string();
        }
        Ok(())
    }

    fn get_mem_info(&mut self, session: &Session) -> Result<(), Box<dyn Error>> {
        let parts = run_command(session, "/bin/cat /proc/meminfo")?;
        let lines = parts.lines().collect::<Vec<_>>();
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
        let lines = parts.lines().collect::<Vec<_>>();

        let mut flag = 0;
        for line in lines {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            let dev = !parts.is_empty() && parts[0].starts_with("/dev/");
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

    #[allow(clippy::collapsible_else_if)]
    fn get_interfaces(&mut self, session: &Session) -> Result<(), Box<dyn Error>> {
        let interfaces = run_command(session, "/bin/ip -o addr")
            .or_else(|_| run_command(session, "/sbin/ip -o addr"))?;

        let lines = interfaces.lines().collect::<Vec<_>>();
        for line in lines {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            if fields.len() >= 4 && (fields[2] == "inet" || fields[2] == "inet6") {
                let ipv4 = fields[2] == "inet";
                let int_name = fields[1];

                if let Some(value) = self.net_intf.get_mut(int_name) {
                    if ipv4 {
                        value.ipv4 = fields[3].to_string();
                    } else {
                        value.ipv6 = fields[3].to_string();
                    }
                } else {
                    if ipv4 {
                        self.net_intf.insert(
                            int_name.to_string(),
                            NetIntfInfo {
                                ipv4: fields[3].to_string(),
                                ipv6: String::new(),
                                rx: u64::default(),
                                tx: u64::default(),
                            },
                        );
                    } else {
                        self.net_intf.insert(
                            int_name.to_string(),
                            NetIntfInfo {
                                ipv4: String::new(),
                                ipv6: fields[3].to_string(),
                                rx: u64::default(),
                                tx: u64::default(),
                            },
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn get_interface_info(&mut self, session: &Session) -> Result<(), Box<dyn Error>> {
        if self.net_intf.is_empty() {
            return Ok(());
        }
        let infos = run_command(session, "/bin/cat /proc/net/dev")?;
        let lines = infos.lines().collect::<Vec<_>>();
        for line in lines {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if parts.len() == 17 {
                let intf = parts[0].trim().trim_matches(':');
                if let Some(value) = self.net_intf.get_mut(intf) {
                    let rx = parts[1].parse::<u64>()?;
                    let tx = parts[9].parse::<u64>()?;
                    value.rx = rx;
                    value.tx = tx;
                }
            }
        }
        Ok(())
    }

    fn get_cpu(&mut self, session: &Session) -> Result<(), Box<dyn Error>> {
        let cpu = run_command(session, "/bin/cat /proc/stat")?;
        let lines = cpu.lines().collect::<Vec<_>>();

        let mut current_cpu = CpuRaw::default();

        for line in lines {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            if !fields.is_empty() && fields[0] == "cpu" {
                parse_cpu(&fields, &mut current_cpu);
                break;
            }
        }

        if self.prev_cpu.total == 0 {
            self.prev_cpu = current_cpu;
            return Ok(());
        }

        let total = (current_cpu.total - self.prev_cpu.total) as f32;

        self.cpu.user = (current_cpu.user - self.prev_cpu.user) as f32 / total * 100.0;
        self.cpu.nice = (current_cpu.nice - self.prev_cpu.nice) as f32 / total * 100.0;
        self.cpu.system = (current_cpu.system - self.prev_cpu.system) as f32 / total * 100.0;
        self.cpu.idle = (current_cpu.idle - self.prev_cpu.idle) as f32 / total * 100.0;
        self.cpu.iowait = (current_cpu.iowait - self.prev_cpu.iowait) as f32 / total * 100.0;
        self.cpu.irq = (current_cpu.irq - self.prev_cpu.irq) as f32 / total * 100.0;
        self.cpu.soft_irq = (current_cpu.soft_irq - self.prev_cpu.soft_irq) as f32 / total * 100.0;
        self.cpu.steal = (current_cpu.steal - self.prev_cpu.steal) as f32 / total * 100.0;
        self.cpu.guest = (current_cpu.guest - self.prev_cpu.guest) as f32 / total * 100.0;

        self.prev_cpu = current_cpu;
        Ok(())
    }
}

fn format_bytes(val: u64) -> String {
    if val < 1024 {
        format!("{} bytes", val)
    } else if val < 1024 * 1024 {
        format!("{:6.2} KiB", val as f64 / 1024.0)
    } else if val < 1024 * 1024 * 1024 {
        format!("{:6.2} MiB", val as f64 / 1024.0 / 1024.0)
    } else {
        format!("{:6.2} GiB", val as f64 / 1024.0 / 1024.0 / 1024.0)
    }
}

fn parse_cpu(fields: &Vec<&str>, cpu: &mut CpuRaw) {
    for i in 1..fields.len() {
        if let Ok(val) = (*fields)[i].parse::<u64>() {
            cpu.total += val;
            match i {
                1 => cpu.user = val,
                2 => cpu.nice = val,
                3 => cpu.system = val,
                4 => cpu.idle = val,
                5 => cpu.iowait = val,
                6 => cpu.irq = val,
                7 => cpu.soft_irq = val,
                8 => cpu.steal = val,
                9 => cpu.guest = val,
                _ => continue,
            }
        }
    }
}

fn run_command(session: &Session, command: &str) -> Result<String, Box<dyn Error>> {
    let mut channel = session.channel_session()?;
    let mut result = String::new();
    channel.exec(command)?;
    channel.read_to_string(&mut result)?;
    Ok(result)
}
