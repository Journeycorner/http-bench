use http::Uri;
use std::env;
use std::result::Result::{self, Err, Ok};
mod runtime;

fn main() {
    match parse_input_arguments() {
        Ok(result) => {
            if let Some(arguments) = result {
                let statistic = runtime::run(&arguments.uri, arguments.number_of_requests);
                println!("{:?}", statistic);
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

struct Arguments {
    number_of_requests: usize,
    uri: Uri,
}
