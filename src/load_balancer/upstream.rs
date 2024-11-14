use std::fmt::Display;

use crate::config::Config;

pub struct HostPort {
    pub host: String,
    pub port: u16,
}

pub struct Upstream {
    pub servers: Vec<HostPort>,
}

impl HostPort {
    fn new(host_port: &str) -> Self {
        let host_port: Vec<&str> = host_port.split(':').collect();
        HostPort {
            host: host_port[0].to_string(),
            port: host_port[1].parse().unwrap(),
        }
    }
}

impl Display for HostPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

impl Upstream {
    pub fn new(upstream_servers: &Vec<String>) -> Self {
        Upstream {
            servers: upstream_servers.iter().map(|s| HostPort::new(s)).collect(),
        }
    }

    pub fn from_config(config: &Config) -> Self {
        Self::new(&config.upstream_servers)
    }

    pub fn start_from_random(&self) -> UpstreamIterator<'_> {
        let start_ind = rand::random::<usize>();
        UpstreamIterator::new(&self.servers, start_ind)
    }
}

pub struct UpstreamIterator<'a> {
    servers: &'a Vec<HostPort>,
    start_ind: usize,
    current_ind: usize,
}

impl<'a> UpstreamIterator<'a> {
    pub fn new(servers: &'a Vec<HostPort>, start_ind: usize) -> Self {
        UpstreamIterator {
            servers,
            start_ind,
            current_ind: start_ind,
        }
    }
}

impl<'a> Iterator for UpstreamIterator<'a> {
    type Item = &'a HostPort;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_ind - self.start_ind + 1 > self.servers.len() {
            return None;
        }
        let result = Some(&self.servers[self.current_ind % self.servers.len()]);
        self.current_ind += 1;
        result
    }
}
