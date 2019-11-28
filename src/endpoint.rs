use serde::Deserialize;
use std::collections::VecDeque;

#[derive(Debug, Deserialize)]
pub enum EndpointSelectors {
    #[serde(rename = "round robin")]
    RoundRobin,
}

pub trait EndpointSelector {
    fn next(&mut self) -> String;
}

pub struct RoundRobin {
    endpoints: VecDeque<String>,
}

impl RoundRobin {
    pub fn new(endpoints: VecDeque<String>) -> RoundRobin {
        assert!(!endpoints.is_empty());
        RoundRobin { endpoints }
    }
}

impl EndpointSelector for RoundRobin {
    fn next(&mut self) -> String {
        let result = self.endpoints.pop_front().unwrap();
        self.endpoints.push_back(result.clone());
        result
    }
}
