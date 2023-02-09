// an implementation of IS-04 for NMOS in rust
//

use hyper::rt::{Future, Stream};
use hyper::Client;
use std::fs::File;
use std::io::prelude::*;
use std::{env, io};

fn main() {
    // register the node
    let mut file = File::open("node.json").expect("Unable to open file");
    let mut node = String::new();
    file.read_to_string(&mut node).expect("Unable to read file");
    let client = Client::new();
    let uri = "http://localhost:8080/x-nmos/node/v1.0/nodes"
        .parse()
        .unwrap();
    let work = client
        .post(uri)
        .body(node)
        .and_then(|res| {
            println!("Response: {}", res.status());
            println!(
                "Headers: {:#?}

",
                res.headers()
            );
            res.into_body()
                .for_each(|&chunk| io::stdout().write_all(&chunk).map_err(From::from))
        })
        .map_err(|err| {
            println!("Error: {}", err);
        });

    hyper::rt::run(work);
}
