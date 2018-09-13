use http::Uri;
use hyper::client::HttpConnector;
use hyper::rt::Future;
use hyper::Client;
use hyper_tls::HttpsConnector;
use std::env;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

fn main() {
    match parse_input_arguments() {
        None => println!("Usage: http-bench 3 https://some-2-example-12-url.com"),
        Some(arguments) => run(&arguments.uri, arguments.number_of_requests),
    };
}

fn parse_input_arguments() -> Option<Arguments> {
    // skip parsing if usage mode
    if env::args().nth(1).unwrap() == "usage" {
        return None;
    }
    // check input argument count
    if env::args().len() != 3 {
        panic!("Needs exactly two input arguments. Type usage for more information.");
    }
    // parse request repeat count
    let number_of_requests: usize = env::args()
        .nth(1)
        .expect("Can't access input param, that's a bug!")
        .parse()
        .expect("Input value is not a valid number!");
    if number_of_requests < 1 || number_of_requests > 10 {
        panic!("Request repeat count must be between 1 and 10");
    }
    // parse uri
    let uri: Uri = env::args()
        .nth(2)
        .expect("Can't access input param, that's a bug!")
        .parse()
        .expect("Not a valid Uri");

    let arguments = Arguments {
        number_of_requests,
        uri,
    };
    Some(arguments)
}

fn run(uri: &Uri, number_of_requests: usize) {
    let https = HttpsConnector::new(4).expect("TLS initialization failed");
    let client = Client::builder().build::<_, hyper::Body>(https);
    // create a new reactor event loop
    let mut rt = Runtime::new().unwrap();
    let (sender, receiver) = mpsc::channel();
    for i in 1..=number_of_requests {
        create_request(sender.clone(), &mut rt, i, &client, uri);
    }

    let mut telemetry: Vec<Duration> = receiver.try_iter().collect();

    let statistic = Statistic::new(&mut telemetry, number_of_requests);
    println!("{:?}", statistic);
}

fn create_request(
    sender: Sender<Duration>,
    runtime: &mut Runtime,
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
    let _result = runtime.block_on(future);
}

#[derive(Debug)]
struct Statistic {
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

struct Arguments {
    number_of_requests: usize,
    uri: Uri,
}
