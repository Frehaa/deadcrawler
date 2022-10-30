use http::Uri;
use hyper::{client::Client, body::HttpBody};
use hyper_tls::HttpsConnector;

use std::env;
use std::collections::HashSet;

#[tokio::main]
async fn get_data(uri: Uri) -> Result<String, Box<dyn std::error::Error + Send + Sync>>{
    let client = Client::new();

    let connector = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(connector);

    let mut response = client.get(uri).await?;

    let status = response.status();
    let version = response.version();
    let headers = response.headers();
    let extensions = response.extensions();
    println!("{:?}", response);

    println!("Status {status} - version {version:?} - extensions {extensions:?}");
    
    for key in headers.keys() {
        println!("headers[{key}] = {:?}", headers.get(key).unwrap());
    }

    let mut v = Vec::new();
    while let Some(chunk) = response.body_mut().data().await {
        v.push(chunk?);
    }

    Ok(String::from_utf8(v.concat())?)
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>{
    // let mut args = env::args();
    // let address = args.nth(1).unwrap();
    // let uri = address.parse()?;
    // let full_body = get_data(uri);

    let full_body = include_str!("itu.dk.txt");

    let mut state = 0;

    let mut links = HashSet::new();
    let mut href = Vec::new();

    let mut a_tags_idx = Vec::new();

    let mut curr_idx = 0;
    for (i, c) in full_body.chars().enumerate() {
        match (c, state) {
            ('<', 0) => { // Start of tag
                state = 1;
                curr_idx = i;
            },
            ('>', 8) => {
                panic!("Didn't expect end of tag in state 8");
            },
            ('>', _) => { // End of tag
                state = 0;
            },
            ('a', 1) => {
                state = 2;
                a_tags_idx.push(curr_idx);
            },
            (_, 1) => { // Not a start 'a' tag
                state = -1;
            }
            ('h', 2) => state = 3,
            (_, 2) => (),
            ('r', 3) => state = 4,
            (_, 3) => state = 2,
            ('e', 4) => state = 5,
            (_, 4) => state = 2,
            ('f', 5) => state = 6,
            (_, 5) => state = 2,
            ('=', 6) => state = 7,
            (_, 6) => state = 2,
            ('"', 7) => { // Enter href src
                state = 8;
            },
            (_, 7) => state = 2,
            ('"', 8) => { // Exit href src
                links.insert(String::from_iter(href.iter()));
                href = Vec::new();
                state = 2;
            },
            (_, 8) => {
                href.push(c);
            },
            _ => {
                // No end of tag before start of tag
                assert!(!(state == 0 && c == '>'), "Error at character {i}: Unexpected '>'")
            }
        }
    }
    

    println!("Number of a's {} - number of unique links {}", &a_tags_idx.len(), &links.len());
    for link in links.iter() {
        println!("{}", link);
    }
    // for idx in a_tags_idx.iter() {
    //     print!("{idx} ");
    //     // println!("{}", String::from_iter(tag.iter()));
    // }
    // println!("");


    // let re = Regex::new("href").unwrap();
    // println!("{}", re.is_match(&full_body));

    // for cap in re.captures_iter(&full_body) {
    //     println!("{:?}", cap);
    // }

    

    // let links = full_body?.matches(|x| { })

    // println!("{}", full_body);

    // response.
    
    // let r = response.poll();


    Ok(())
}
