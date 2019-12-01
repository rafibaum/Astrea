use serde::Deserialize;
use std::collections::VecDeque;

#[derive(Debug, Deserialize)]
pub enum Strategy {
    #[serde(rename = "round robin")]
    RoundRobin,
}

pub trait EndpointStrategy<T: Clone> {
    fn next(&self, queue: &mut VecDeque<T>) -> T;
}

pub struct RoundRobin;

impl<T: Clone> EndpointStrategy<T> for RoundRobin {
    fn next(&self, queue: &mut VecDeque<T>) -> T {
        let endpoint = queue.pop_front().unwrap();
        queue.push_back(endpoint.clone());
        endpoint
    }
}

pub struct EndpointSelector<T: Clone> {
    endpoints: VecDeque<T>,
    strategy: Box<dyn EndpointStrategy<T> + Send + Sync>,
}

pub trait EndpointParser<T: Clone> {
    fn parse_endpoint(&self, input: String) -> T;
}

impl<T: Clone> EndpointSelector<T> {
    pub fn new<P: EndpointParser<T>>(endpoints: Vec<String>, parser: P, strategy: Box<dyn EndpointStrategy<T> + Send + Sync>) -> Self {
        EndpointSelector {
            endpoints: endpoints
                .into_iter()
                .map(|x| parser.parse_endpoint(x))
                .collect(),
                strategy,
        }
    }

    pub fn next(&mut self) -> T {
        self.strategy.next(&mut self.endpoints)
    }
}
