use serde::Deserialize;
use std::collections::VecDeque;
use std::net::IpAddr;

#[derive(Debug, Deserialize)]
pub enum EndpointSelectors {
    #[serde(alias = "round robin")]
    RoundRobin,
}

pub trait EndpointSelector {
    fn next(&mut self) -> (IpAddr, u16);
}

pub struct RoundRobin {
    endpoints: VecDeque<(IpAddr, u16)>,
}

impl RoundRobin {
    pub fn new(endpoints: VecDeque<(IpAddr, u16)>) -> RoundRobin {
        assert!(!endpoints.is_empty());
        RoundRobin { endpoints }
    }
}

impl EndpointSelector for RoundRobin {
    fn next(&mut self) -> (IpAddr, u16) {
        let result = self.endpoints.pop_front().unwrap();
        self.endpoints.push_back(result);
        result
    }
}
