// VpnCloud - Peer-to-Peer VPN
// Copyright (C) 2015-2020  Dennis Schwerdel
// This software is licensed under GPL-3 or newer (see LICENSE.md)

#![cfg_attr(feature = "bench", feature(test))]

#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;

#[cfg(test)] extern crate tempfile;
#[cfg(feature = "bench")] extern crate test;

#[macro_use]
pub mod util;
#[cfg(test)]
#[macro_use]
mod tests;
pub mod beacon;
pub mod cloud;
pub mod config;
pub mod crypto;
pub mod device;
pub mod error;
pub mod messages;
pub mod net;
pub mod oldconfig;
pub mod payload;
pub mod poll;
pub mod port_forwarding;
pub mod table;
pub mod traffic;
pub mod types;

use structopt::StructOpt;

use std::{
    fs::{self, File, Permissions},
    io::{self, Write},
    net::{Ipv4Addr, UdpSocket},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
    str::FromStr,
    sync::Mutex,
    thread
};

use crate::{
    cloud::GenericCloud,
    config::{Args, Config},
    crypto::Crypto,
    device::{Device, TunTapDevice, Type},
    oldconfig::OldConfigFile,
    payload::Protocol,
    port_forwarding::PortForwarding,
    util::SystemTimeSource
};


struct DualLogger {
    file: Option<Mutex<File>>
}

impl DualLogger {
    pub fn new<P: AsRef<Path>>(path: Option<P>) -> Result<Self, io::Error> {
        if let Some(path) = path {
            let path = path.as_ref();
            if path.exists() {
                fs::remove_file(path)?
            }
            let file = File::create(path)?;
            Ok(DualLogger { file: Some(Mutex::new(file)) })
        } else {
            Ok(DualLogger { file: None })
        }
    }
}

impl log::Log for DualLogger {
    #[inline]
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    #[inline]
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
            if let Some(ref file) = self.file {
                let mut file = file.lock().expect("Lock poisoned");
                let time = time::OffsetDateTime::now_local().format("%F %H:%M:%S");
                writeln!(file, "{} - {} - {}", time, record.level(), record.args())
                    .expect("Failed to write to logfile");
            }
        }
    }

    #[inline]
    fn flush(&self) {
        if let Some(ref file) = self.file {
            let mut file = file.lock().expect("Lock poisoned");
            try_fail!(file.flush(), "Logging error: {}");
        }
    }
}

fn run_script(script: &str, ifname: &str) {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(&script).env("IFNAME", ifname);
    debug!("Running script: {:?}", cmd);
    match cmd.status() {
        Ok(status) => {
            if !status.success() {
                error!("Script returned with error: {:?}", status.code())
            }
        }
        Err(e) => error!("Failed to execute script {:?}: {}", script, e)
    }
}

fn parse_ip_netmask(addr: &str) -> Result<(Ipv4Addr, Ipv4Addr), String> {
    let (ip_str, len_str) = match addr.find('/') {
        Some(pos) => (&addr[..pos], &addr[pos + 1..]),
        None => (addr, "24")
    };
    let prefix_len = u8::from_str(len_str).map_err(|_| format!("Invalid prefix length: {}", len_str))?;
    if prefix_len > 32 {
        return Err(format!("Invalid prefix length: {}", prefix_len))
    }
    let ip = Ipv4Addr::from_str(ip_str).map_err(|_| format!("Invalid ip address: {}", ip_str))?;
    let netmask = Ipv4Addr::from(u32::max_value().checked_shl(32 - prefix_len as u32).unwrap());
    Ok((ip, netmask))
}

fn setup_device(config: &Config) -> TunTapDevice {
    let device = try_fail!(
        TunTapDevice::new(&config.device_name, config.device_type, config.device_path.as_ref().map(|s| s as &str)),
        "Failed to open virtual {} interface {}: {}",
        config.device_type,
        config.device_name
    );
    info!("Opened device {}", device.ifname());
    if let Err(err) = device.set_mtu(None) {
        error!("Error setting optimal MTU on {}: {}", device.ifname(), err);
    }
    if let Some(ip) = &config.ip {
        let (ip, netmask) = try_fail!(parse_ip_netmask(ip), "Invalid ip address given: {}");
        info!("Configuring device with ip {}, netmask {}", ip, netmask);
        try_fail!(device.configure(ip, netmask), "Failed to configure device: {}");
    }
    if let Some(script) = &config.ifup {
        run_script(script, device.ifname());
    }
    if config.fix_rp_filter {
        try_fail!(device.fix_rp_filter(), "Failed to change rp_filter settings: {}");
    }
    if let Ok(val) = device.get_rp_filter() {
        if val != 1 {
            warn!("Your networking configuration might be affected by a vulnerability (https://vpncloud.ddswd.de/docs/security/cve-2019-14899/), please change your rp_filter setting to 1 (currently {}).", val);
        }
    }
    device
}


#[allow(clippy::cognitive_complexity)]
fn run<P: Protocol>(config: Config) {
    let device = setup_device(&config);
    let port_forwarding = if config.port_forwarding { PortForwarding::new(config.listen.port()) } else { None };
    let stats_file = match config.stats_file {
        None => None,
        Some(ref name) => {
            let path = Path::new(name);
            if path.exists() {
                try_fail!(fs::remove_file(path), "Failed to remove file {}: {}", name);
            }
            let file = try_fail!(File::create(name), "Failed to create stats file: {}");
            try_fail!(
                fs::set_permissions(name, Permissions::from_mode(0o644)),
                "Failed to set permissions on stats file: {}"
            );
            Some(file)
        }
    };
    let mut cloud =
        GenericCloud::<TunTapDevice, P, UdpSocket, SystemTimeSource>::new(&config, device, port_forwarding, stats_file);
    for addr in config.peers {
        try_fail!(cloud.connect(&addr as &str), "Failed to send message to {}: {}", &addr);
        cloud.add_reconnect_peer(addr);
    }
    if config.daemonize {
        info!("Running process as daemon");
        let mut daemonize = daemonize::Daemonize::new();
        if let Some(user) = config.user {
            daemonize = daemonize.user(&user as &str);
        }
        if let Some(group) = config.group {
            daemonize = daemonize.group(&group as &str);
        }
        if let Some(pid_file) = config.pid_file {
            daemonize = daemonize.pid_file(pid_file).chown_pid_file(true);
            // Give child process some time to write PID file
            daemonize = daemonize.exit_action(|| thread::sleep(std::time::Duration::from_millis(10)));
        }
        try_fail!(daemonize.start(), "Failed to daemonize: {}");
    } else if config.user.is_some() || config.group.is_some() {
        info!("Dropping privileges");
        let mut pd = privdrop::PrivDrop::default();
        if let Some(user) = config.user {
            pd = pd.user(user);
        }
        if let Some(group) = config.group {
            pd = pd.group(group);
        }
        try_fail!(pd.apply(), "Failed to drop privileges: {}");
    }
    cloud.run();
    if let Some(script) = config.ifdown {
        run_script(&script, cloud.ifname());
    }
}

fn main() {
    let args: Args = Args::from_args();
    if args.version {
        println!("VpnCloud v{}", env!("CARGO_PKG_VERSION"));
        return
    }
    if args.genkey {
        let (privkey, pubkey) = Crypto::generate_keypair(args.password.as_deref());
        println!("Private key: {}\nPublic key: {}\n", privkey, pubkey);
        println!(
            "Attention: Keep the private key secret and use only the public key on other nodes to establish trust."
        );
        return
    }
    let logger = try_fail!(DualLogger::new(args.log_file.as_ref()), "Failed to open logfile: {}");
    log::set_boxed_logger(Box::new(logger)).unwrap();
    assert!(!args.verbose || !args.quiet);
    log::set_max_level(if args.verbose {
        log::LevelFilter::Debug
    } else if args.quiet {
        log::LevelFilter::Error
    } else {
        log::LevelFilter::Info
    });
    if args.migrate_config {
        let file = args.config.unwrap();
        info!("Trying to convert from old config format");
        let f = try_fail!(File::open(&file), "Failed to open config file: {:?}");
        let config_file_old: OldConfigFile =
            try_fail!(serde_yaml::from_reader(f), "Config file not valid for version 1: {:?}");
        let new_config = config_file_old.convert();
        info!("Successfully converted from old format");
        info!("Renaming original file to {}.orig", file);
        try_fail!(fs::rename(&file, format!("{}.orig", file)), "Failed to rename original file: {:?}");
        info!("Writing new config back into {}", file);
        let f = try_fail!(File::create(&file), "Failed to open config file: {:?}");
        try_fail!(
            fs::set_permissions(&file, fs::Permissions::from_mode(0o600)),
            "Failed to set permissions on file: {:?}"
        );
        try_fail!(serde_yaml::to_writer(f, &new_config), "Failed to write converted config: {:?}");
        return
    }
    let mut config = Config::default();
    if let Some(ref file) = args.config {
        info!("Reading config file '{}'", file);
        let f = try_fail!(File::open(file), "Failed to open config file: {:?}");
        let config_file = match serde_yaml::from_reader(f) {
            Ok(config) => config,
            Err(err) => {
                error!("Failed to read config file: {}", err);
                info!("Trying to convert from old config format");
                let f = try_fail!(File::open(file), "Failed to open config file: {:?}");
                let config_file_old: OldConfigFile =
                    try_fail!(serde_yaml::from_reader(f), "Config file is neither version 2 nor version 1: {:?}");
                let new_config = config_file_old.convert();
                info!("Successfully converted from old format, please migrate your config using migrate-config");
                new_config
            }
        };
        config.merge_file(config_file)
    }
    config.merge_args(args);
    debug!("Config: {:?}", config);
    match config.device_type {
        Type::Tap => run::<payload::Frame>(config),
        Type::Tun => run::<payload::Packet>(config)
    }
}
