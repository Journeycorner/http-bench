use http::Uri;
use hyper::client::HttpConnector;
use hyper::Client;
use hyper_rustls::HttpsConnector;
use std::env;
use std::result::Result::{self, Err, Ok};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use tokio::prelude::*;

fn main() {
    match parse_input_arguments() {
        Ok(result) => {
            if let Some(arguments) = result {
                run(&arguments.uri, arguments.number_of_requests);
            } else {
                println!("Usage: http-bench 3 https://some-2-example-12-url.com");
            }
        }
        Err(message) => eprintln!("{}", message),
    }
}

fn parse_input_arguments() -> Result<Option<Arguments>, String> {
    let raw_arguments: Vec<String> = env::args().skip(1).collect();

    if raw_arguments.len() == 1 && raw_arguments[0] == "usage" {
        // skip parsing if usage mode
        return Ok(None);
    } else if raw_arguments.len() != 2 {
        // check input argument count
        return Err(String::from(
            "Needs exactly two input arguments. Type usage for more information.",
        ));
    }

    // parse request repeat count
    let number_of_requests = if let Ok(value) = raw_arguments[0].parse::<usize>() {
        if value < 1 || value > 10 {
            Err(String::from(
                "Request repeat count must be between 1 and 10",
            ))
        } else {
            Ok(value)
        }
    } else {
        Err(format!("{} is not a number", raw_arguments[0]))
    }?;

    // parse uri
    let uri = match raw_arguments[1].parse::<Uri>() {
        Ok(value) => Ok(value),
        Err(_err) => Err(format!("{} is not a valid uri", raw_arguments[1])),
    }?;
    if uri.scheme_part().is_none() {
        return Err(format!("{} needs to be an absolute uri", raw_arguments[1]));
    }

    let arguments = Arguments {
        number_of_requests,
        uri,
    };
    Ok(Some(arguments))
}

fn run(uri: &Uri, number_of_requests: usize) {
    let https = HttpsConnector::new(4);
    let client = Client::builder().build::<_, hyper::Body>(https);
    // create a new reactor event loop
    let (sender, receiver) = mpsc::channel();
    for i in 1..=number_of_requests {
        create_request(sender.clone(), i, &client, uri);
    }

    let mut telemetry: Vec<Duration> = receiver.try_iter().collect();
    let statistic = Statistic::new(&mut telemetry, number_of_requests);
    println!("{:?}", statistic);
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
