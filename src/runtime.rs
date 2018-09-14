use http::Uri;
use hyper::client::HttpConnector;
use hyper::Client;
use hyper_rustls::HttpsConnector;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use tokio::prelude::Future;

pub fn run(uri: &Uri, number_of_requests: usize) -> Statistic {
    let https = HttpsConnector::new(4);
    let client = Client::builder().build::<_, hyper::Body>(https);
    // create a new reactor event loop
    let (sender, receiver) = mpsc::channel();
    for i in 1..=number_of_requests {
        create_request(sender.clone(), i, &client, uri);
    }

    let mut telemetry: Vec<Duration> = receiver.try_iter().collect();
    Statistic::new(&mut telemetry, number_of_requests)
}

fn create_request(
    sender: Sender<Duration>,
    i: usize,
    client: &Client<HttpsConnector<HttpConnector>>,
    uri: &Uri,
) {
    let start = Instant::now();
    let future = client
        .get(uri.clone())
        .map(move |res| {
            let elapsed = start.elapsed();
            sender.send(elapsed).unwrap();
            println!("{}. request took {:?}", i, elapsed);
            println!("Response: {}\n", res.status());
        }).map_err(|err| {
            println!("Error: {}\n", err);
        });

    tokio::run(future);
}

#[derive(Debug)]
pub struct Statistic {
    average: Duration,
    median: Duration,
}

impl Statistic {
    fn new(values: &mut Vec<Duration>, number_of_requests: usize) -> Self {
        values.sort();
        let sum: Duration = values.iter().sum();
        let average = sum / number_of_requests as u32;
        let middle = values.len() / 2;
        let median = values[middle];

        Self { average, median }
    }
}
