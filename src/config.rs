// VpnCloud - Peer-to-Peer VPN
// Copyright (C) 2015-2019  Dennis Schwerdel
// This software is licensed under GPL-3 or newer (see LICENSE.md)

use super::{Args, MAGIC};

use super::{
    crypto::CryptoMethod,
    device::Type,
    types::{HeaderMagic, Mode},
    util::{Duration, Encoder}
};

use siphasher::sip::SipHasher24;
use std::hash::{Hash, Hasher};


const HASH_PREFIX: &str = "hash:";


#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Config {
    pub device_type: Type,
    pub device_name: String,
    pub device_path: Option<String>,
    pub ifup: Option<String>,
    pub ifdown: Option<String>,
    pub crypto: CryptoMethod,
    pub shared_key: Option<String>,
    pub magic: Option<String>,
    pub port: u16,
    pub peers: Vec<String>,
    pub peer_timeout: Duration,
    pub keepalive: Option<Duration>,
    pub beacon_store: Option<String>,
    pub beacon_load: Option<String>,
    pub beacon_interval: Duration,
    pub mode: Mode,
    pub dst_timeout: Duration,
    pub subnets: Vec<String>,
    pub port_forwarding: bool,
    pub daemonize: bool,
    pub pid_file: Option<String>,
    pub stats_file: Option<String>,
    pub user: Option<String>,
    pub group: Option<String>
}

impl Default for Config {
    fn default() -> Self {
        Config {
            device_type: Type::Tap,
            device_name: "vpncloud%d".to_string(),
            device_path: None,
            ifup: None,
            ifdown: None,
            crypto: CryptoMethod::ChaCha20,
            shared_key: None,
            magic: None,
            port: 3210,
            peers: vec![],
            peer_timeout: 1800,
            keepalive: None,
            beacon_store: None,
            beacon_load: None,
            beacon_interval: 3600,
            mode: Mode::Normal,
            dst_timeout: 300,
            subnets: vec![],
            port_forwarding: true,
            daemonize: false,
            pid_file: None,
            stats_file: None,
            user: None,
            group: None
        }
    }
}

impl Config {
    pub fn merge_file(&mut self, file: ConfigFile) {
        if let Some(val) = file.device_type {
            self.device_type = val;
        }
        if let Some(val) = file.device_name {
            self.device_name = val;
        }
        if let Some(val) = file.device_path {
            self.device_path = Some(val);
        }
        if let Some(val) = file.ifup {
            self.ifup = Some(val);
        }
        if let Some(val) = file.ifdown {
            self.ifdown = Some(val);
        }
        if let Some(val) = file.crypto {
            self.crypto = val;
        }
        if let Some(val) = file.shared_key {
            self.shared_key = Some(val);
        }
        if let Some(val) = file.magic {
            self.magic = Some(val);
        }
        if let Some(val) = file.port {
            self.port = val;
        }
        if let Some(mut val) = file.peers {
            self.peers.append(&mut val);
        }
        if let Some(val) = file.peer_timeout {
            self.peer_timeout = val;
        }
        if let Some(val) = file.keepalive {
            self.keepalive = Some(val);
        }
        if let Some(val) = file.beacon_store {
            self.beacon_store = Some(val);
        }
        if let Some(val) = file.beacon_load {
            self.beacon_load = Some(val);
        }
        if let Some(val) = file.beacon_interval {
            self.beacon_interval = val;
        }
        if let Some(val) = file.mode {
            self.mode = val;
        }
        if let Some(val) = file.dst_timeout {
            self.dst_timeout = val;
        }
        if let Some(mut val) = file.subnets {
            self.subnets.append(&mut val);
        }
        if let Some(val) = file.port_forwarding {
            self.port_forwarding = val;
        }
        if let Some(val) = file.pid_file {
            self.pid_file = Some(val);
        }
        if let Some(val) = file.stats_file {
            self.stats_file = Some(val);
        }
        if let Some(val) = file.user {
            self.user = Some(val);
        }
        if let Some(val) = file.group {
            self.group = Some(val);
        }
    }

    pub fn merge_args(&mut self, mut args: Args) {
        if let Some(val) = args.flag_type {
            self.device_type = val;
        }
        if let Some(val) = args.flag_device {
            self.device_name = val;
        }
        if let Some(val) = args.flag_device_path {
            self.device_path = Some(val);
        }
        if let Some(val) = args.flag_ifup {
            self.ifup = Some(val);
        }
        if let Some(val) = args.flag_ifdown {
            self.ifdown = Some(val);
        }
        if let Some(val) = args.flag_crypto {
            self.crypto = val;
        }
        if let Some(val) = args.flag_shared_key {
            self.shared_key = Some(val);
        }
        if let Some(val) = args.flag_network_id {
            warn!("The --network-id argument is deprecated, please use --magic instead.");
            self.magic = Some(val);
        }
        if let Some(val) = args.flag_magic {
            self.magic = Some(val);
        }
        if let Some(val) = args.flag_listen {
            self.port = val;
        }
        self.peers.append(&mut args.flag_connect);
        if let Some(val) = args.flag_peer_timeout {
            self.peer_timeout = val;
        }
        if let Some(val) = args.flag_keepalive {
            self.keepalive = Some(val);
        }
        if let Some(val) = args.flag_beacon_store {
            self.beacon_store = Some(val);
        }
        if let Some(val) = args.flag_beacon_load {
            self.beacon_load = Some(val);
        }
        if let Some(val) = args.flag_beacon_interval {
            self.beacon_interval = val;
        }
        if let Some(val) = args.flag_mode {
            self.mode = val;
        }
        if let Some(val) = args.flag_dst_timeout {
            self.dst_timeout = val;
        }
        self.subnets.append(&mut args.flag_subnet);
        if args.flag_no_port_forwarding {
            self.port_forwarding = false;
        }
        if args.flag_daemon {
            self.daemonize = true;
        }
        if let Some(val) = args.flag_pid_file {
            self.pid_file = Some(val);
        }
        if let Some(val) = args.flag_stats_file {
            self.stats_file = Some(val);
        }
        if let Some(val) = args.flag_user {
            self.user = Some(val);
        }
        if let Some(val) = args.flag_group {
            self.group = Some(val);
        }
    }

    pub fn get_magic(&self) -> HeaderMagic {
        if let Some(ref name) = self.magic {
            if name.starts_with(HASH_PREFIX) {
                let mut s = SipHasher24::new();
                name[HASH_PREFIX.len()..].hash(&mut s);
                let mut data = [0; 4];
                Encoder::write_u32((s.finish() & 0xffff_ffff) as u32, &mut data);
                data
            } else {
                let num = try_fail!(u32::from_str_radix(name, 16), "Failed to parse header magic: {}");
                let mut data = [0; 4];
                Encoder::write_u32(num, &mut data);
                data
            }
        } else {
            MAGIC
        }
    }

    pub fn get_keepalive(&self) -> Duration {
        match self.keepalive {
            Some(dur) => dur,
            None => self.peer_timeout / 2 - 60
        }
    }
}


#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct ConfigFile {
    pub device_type: Option<Type>,
    pub device_name: Option<String>,
    pub device_path: Option<String>,
    pub ifup: Option<String>,
    pub ifdown: Option<String>,
    pub crypto: Option<CryptoMethod>,
    pub shared_key: Option<String>,
    pub magic: Option<String>,
    pub port: Option<u16>,
    pub peers: Option<Vec<String>>,
    pub peer_timeout: Option<Duration>,
    pub keepalive: Option<Duration>,
    pub beacon_store: Option<String>,
    pub beacon_load: Option<String>,
    pub beacon_interval: Option<Duration>,
    pub mode: Option<Mode>,
    pub dst_timeout: Option<Duration>,
    pub subnets: Option<Vec<String>>,
    pub port_forwarding: Option<bool>,
    pub pid_file: Option<String>,
    pub stats_file: Option<String>,
    pub user: Option<String>,
    pub group: Option<String>
}


#[test]
fn config_file() {
    let config_file = "
device_type: tun
device_name: vpncloud%d
device_path: /dev/net/tun
magic: 0123ABCD
ifup: ifconfig $IFNAME 10.0.1.1/16 mtu 1400 up
ifdown: 'true'
crypto: aes256
shared_key: mysecret
port: 3210
peers:
  - remote.machine.foo:3210
  - remote.machine.bar:3210
peer_timeout: 1800
keepalive: 840
dst_timeout: 300
beacon_store: /run/vpncloud.beacon.out
beacon_load: /run/vpncloud.beacon.in
beacon_interval: 3600
mode: normal
subnets:
  - 10.0.1.0/24
port_forwarding: true
user: nobody
group: nogroup
pid_file: /run/vpncloud.run
stats_file: /var/log/vpncloud.stats
    ";
    assert_eq!(serde_yaml::from_str::<ConfigFile>(config_file).unwrap(), ConfigFile {
        device_type: Some(Type::Tun),
        device_name: Some("vpncloud%d".to_string()),
        device_path: Some("/dev/net/tun".to_string()),
        ifup: Some("ifconfig $IFNAME 10.0.1.1/16 mtu 1400 up".to_string()),
        ifdown: Some("true".to_string()),
        crypto: Some(CryptoMethod::AES256),
        shared_key: Some("mysecret".to_string()),
        magic: Some("0123ABCD".to_string()),
        port: Some(3210),
        peers: Some(vec!["remote.machine.foo:3210".to_string(), "remote.machine.bar:3210".to_string()]),
        peer_timeout: Some(1800),
        keepalive: Some(840),
        beacon_store: Some("/run/vpncloud.beacon.out".to_string()),
        beacon_load: Some("/run/vpncloud.beacon.in".to_string()),
        beacon_interval: Some(3600),
        mode: Some(Mode::Normal),
        dst_timeout: Some(300),
        subnets: Some(vec!["10.0.1.0/24".to_string()]),
        port_forwarding: Some(true),
        user: Some("nobody".to_string()),
        group: Some("nogroup".to_string()),
        pid_file: Some("/run/vpncloud.run".to_string()),
        stats_file: Some("/var/log/vpncloud.stats".to_string())
    })
}

#[test]
fn config_merge() {
    let mut config = Config::default();
    config.merge_file(ConfigFile {
        device_type: Some(Type::Tun),
        device_name: Some("vpncloud%d".to_string()),
        device_path: None,
        ifup: Some("ifconfig $IFNAME 10.0.1.1/16 mtu 1400 up".to_string()),
        ifdown: Some("true".to_string()),
        crypto: Some(CryptoMethod::AES256),
        shared_key: Some("mysecret".to_string()),
        magic: Some("0123ABCD".to_string()),
        port: Some(3210),
        peers: Some(vec!["remote.machine.foo:3210".to_string(), "remote.machine.bar:3210".to_string()]),
        peer_timeout: Some(1800),
        keepalive: Some(840),
        beacon_store: Some("/run/vpncloud.beacon.out".to_string()),
        beacon_load: Some("/run/vpncloud.beacon.in".to_string()),
        beacon_interval: Some(7200),
        mode: Some(Mode::Normal),
        dst_timeout: Some(300),
        subnets: Some(vec!["10.0.1.0/24".to_string()]),
        port_forwarding: Some(true),
        user: Some("nobody".to_string()),
        group: Some("nogroup".to_string()),
        pid_file: Some("/run/vpncloud.run".to_string()),
        stats_file: Some("/var/log/vpncloud.stats".to_string())
    });
    assert_eq!(config, Config {
        device_type: Type::Tun,
        device_name: "vpncloud%d".to_string(),
        device_path: None,
        ifup: Some("ifconfig $IFNAME 10.0.1.1/16 mtu 1400 up".to_string()),
        ifdown: Some("true".to_string()),
        magic: Some("0123ABCD".to_string()),
        crypto: CryptoMethod::AES256,
        shared_key: Some("mysecret".to_string()),
        port: 3210,
        peers: vec!["remote.machine.foo:3210".to_string(), "remote.machine.bar:3210".to_string()],
        peer_timeout: 1800,
        keepalive: Some(840),
        dst_timeout: 300,
        beacon_store: Some("/run/vpncloud.beacon.out".to_string()),
        beacon_load: Some("/run/vpncloud.beacon.in".to_string()),
        beacon_interval: 7200,
        mode: Mode::Normal,
        port_forwarding: true,
        subnets: vec!["10.0.1.0/24".to_string()],
        user: Some("nobody".to_string()),
        group: Some("nogroup".to_string()),
        pid_file: Some("/run/vpncloud.run".to_string()),
        stats_file: Some("/var/log/vpncloud.stats".to_string()),
        ..Default::default()
    });
    config.merge_args(Args {
        flag_type: Some(Type::Tap),
        flag_device: Some("vpncloud0".to_string()),
        flag_device_path: Some("/dev/null".to_string()),
        flag_ifup: Some("ifconfig $IFNAME 10.0.1.2/16 mtu 1400 up".to_string()),
        flag_ifdown: Some("ifconfig $IFNAME down".to_string()),
        flag_crypto: Some(CryptoMethod::ChaCha20),
        flag_shared_key: Some("anothersecret".to_string()),
        flag_magic: Some("hash:mynet".to_string()),
        flag_listen: Some(3211),
        flag_peer_timeout: Some(1801),
        flag_keepalive: Some(850),
        flag_dst_timeout: Some(301),
        flag_beacon_store: Some("/run/vpncloud.beacon.out2".to_string()),
        flag_beacon_load: Some("/run/vpncloud.beacon.in2".to_string()),
        flag_beacon_interval: Some(3600),
        flag_mode: Some(Mode::Switch),
        flag_subnet: vec![],
        flag_connect: vec!["another:3210".to_string()],
        flag_no_port_forwarding: true,
        flag_daemon: true,
        flag_pid_file: Some("/run/vpncloud-mynet.run".to_string()),
        flag_stats_file: Some("/var/log/vpncloud-mynet.stats".to_string()),
        flag_user: Some("root".to_string()),
        flag_group: Some("root".to_string()),
        ..Default::default()
    });
    assert_eq!(config, Config {
        device_type: Type::Tap,
        device_name: "vpncloud0".to_string(),
        device_path: Some("/dev/null".to_string()),
        ifup: Some("ifconfig $IFNAME 10.0.1.2/16 mtu 1400 up".to_string()),
        ifdown: Some("ifconfig $IFNAME down".to_string()),
        magic: Some("hash:mynet".to_string()),
        crypto: CryptoMethod::ChaCha20,
        shared_key: Some("anothersecret".to_string()),
        port: 3211,
        peers: vec![
            "remote.machine.foo:3210".to_string(),
            "remote.machine.bar:3210".to_string(),
            "another:3210".to_string()
        ],
        peer_timeout: 1801,
        keepalive: Some(850),
        dst_timeout: 301,
        beacon_store: Some("/run/vpncloud.beacon.out2".to_string()),
        beacon_load: Some("/run/vpncloud.beacon.in2".to_string()),
        beacon_interval: 3600,
        mode: Mode::Switch,
        port_forwarding: false,
        subnets: vec!["10.0.1.0/24".to_string()],
        user: Some("root".to_string()),
        group: Some("root".to_string()),
        pid_file: Some("/run/vpncloud-mynet.run".to_string()),
        stats_file: Some("/var/log/vpncloud-mynet.stats".to_string()),
        daemonize: true
    });
}
