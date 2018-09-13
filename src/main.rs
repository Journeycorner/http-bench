use hyper::Client;
use hyper::rt::Future;
use std::time::{Instant};
use std::env;
use tokio::runtime::Runtime;
use hyper_tls::HttpsConnector;
use http::Uri;

fn main() {
    match parse_input_arguments() {
        None => println!("Usage: http-bench 3 https://some-2-example-12-url.com"),
        Some(arguments) => run(arguments.uri, arguments.number_of_requests),
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
    let number_of_requests : i32 = env::args().nth(1)
        .expect("Can't access input param, that's a bug!")
        .parse()
        .expect("Input value is not a valid number!");
    if number_of_requests < 1 || number_of_requests > 10 {
        panic!("Request repeat count must be between 1 and 10");
    }
    // parse uri
    let uri : Uri = env::args().nth(2)
        .expect("Can't access input param, that's a bug!")
        .parse()
        .expect("Not a valid Uri");

    let arguments = Arguments {
        number_of_requests,
        uri,
    };
    Some(arguments)
}

fn run(uri: Uri, number_of_requests: i32) {
    let https = HttpsConnector::new(4).expect("TLS initialization failed");
    let client = Client::builder()
        .build::<_, hyper::Body>(https);

    // create a new reactor event loop
    let mut rt = Runtime::new().unwrap();

    for i in 1..=number_of_requests {
        let start = Instant::now();
        let future = client
            .get(uri.clone())
            .map(move |res| {
                println!("Response: {}", res.status());
                println!("{}. request took {}ms to complete\n", i, start.elapsed().subsec_millis());
            })
            .map_err(|err| {
                println!("Error: {}", err);
            });
        let _result = rt.block_on(future);
    }
}

struct Arguments {
    number_of_requests: i32,
    uri: Uri,
}