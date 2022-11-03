use http::uri::Scheme;
use http::{Uri, Error, StatusCode};
use hyper::{client::Client, body::HttpBody};
use hyper_tls::HttpsConnector;
use url::Url;

use std::env;
use std::collections::HashSet;

#[derive(Debug)]
enum ResponseStatusCode {
    NOT_FOUND,
    OK,
    OTHER
}

#[derive(Debug)]
struct ResponseData {
    statusCode : ResponseStatusCode,
    body : Option<String>
}

#[derive(Debug)]
struct ATag {
    start_idx: usize,
    id: Option<String>,
    href: String
}

#[tokio::main]
async fn get_data(uri: Uri) -> Result<ResponseData, Box<dyn std::error::Error + Send + Sync>>{
    let connector = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(connector);

    let mut response = client.get(uri).await?;

    let status = response.status();

    match status {
        StatusCode::NOT_FOUND => {
            Ok(ResponseData { statusCode: ResponseStatusCode::NOT_FOUND, body: None })
        },
        StatusCode::OK => {
            let mut v = Vec::new();
            while let Some(chunk) = response.body_mut().data().await {
                v.push(chunk?);
            }
            Ok(ResponseData { 
                statusCode: ResponseStatusCode::OK, 
                body: Some(String::from_utf8(v.concat())?) 
            })
        },
        _ => {
            Ok(ResponseData { statusCode: ResponseStatusCode::OTHER, body: None })
        }


    }

    // let version = response.version();
    // let headers = response.headers();
    // let extensions = response.extensions();
    // println!("{:?}", response);
    // println!("Status {status} - version {version:?} - extensions {extensions:?}");

    // for key in headers.keys() {
    //     println!("headers[{key}] = {:?}", headers.get(key).unwrap());
    // }

}

fn parse_to_uri(address : &str) -> Result<Uri, &str> {
    let uri = address.parse::<Uri>();
    if uri.is_err() {
        return Err("Could not parse as Uri");
    }
    let uri = unsafe { uri.unwrap_unchecked() };

    let scheme = uri.scheme().filter(|&s| Scheme::HTTP.eq(s) || Scheme::HTTPS.eq(s));
    if scheme.is_none() {
        return Err("Invalid Uri scheme: Not http or https");
    }

    Ok(uri)
}

fn url_to_uri(url: Url) -> Uri {
    todo!()
}

// Figure out if the new address is relative and if so, join it to the base, otherwise use it
fn consolidate_uri(address : &str, base : &Url) -> Uri {
    // How to join url with relative paths? 
    // let base = "https://www.itu.dk/".parse::<Url>().unwrap();

    // let part = "#a";
    let url = address.parse::<Url>();
    if let Ok(r) = url { // This is not 
        return url_to_uri(r);
    }

    let join = base.join(&address);
    println!("{:?}", join);

    // let r = join.unwrap().host().unwrap().eq(&base.host().unwrap());
    // println!("{r}");
    // join.unwrap().

    Uri::default()
}

// TODO: Check for subdomains myself or not? Do I want to crawl over github if I find a link to it? Maybe I should instead make a separate crawl for subdomains. Or maybe just make it toggleable
fn main() -> () { //Result<(), Box<dyn std::error::Error + Send + Sync>>{
    let result = parse_to_uri("https://github.com");

    let uri = result.unwrap();

    println!("{:?}", uri.scheme_str());
    println!("{:?}", uri.path());
    println!("{:?}", uri.query());
    println!("{:?}", uri.host());



    return;

    let mut args = env::args();

    let address = args.nth(1);
    if address.is_none() {
        println!("Error: No address given.");
        return;
    }
    let address = unsafe { address.unwrap_unchecked() };

    let uri = parse_to_uri(&address);
    if let Err(e) = uri {
        println!("Error: {}", e);
        return;
    }

    let data = get_data(unsafe { uri.unwrap_unchecked() });
    if let Err(e) = data {
        println!("Something went wrong {}", e);
        return;
    }
    let data = unsafe { data.unwrap_unchecked() };

    match data.statusCode {
        ResponseStatusCode::NOT_FOUND => {

        },
        ResponseStatusCode::OK => {

        },
        ResponseStatusCode::OTHER => {

        }
    }

    // println!("{address}");
    // println!("{url:?}");
    // println!("{:?}", &uri.path_and_query());
    // println!("{:?}", uri.host());
    // let parts = &uri.into_parts();
    // println!("{:?}", parts);
    // let full_body = get_data(uri);


    // return;

    let full_body = include_str!("itu.dk.txt");

    let mut state = 0;

    let mut atags = Vec::new();
    let mut hrefbuilder = Vec::new();
    let mut a_tags_idx = Vec::new();

    let mut current_href = "".to_string();
    let mut tag_start_idx = 0;
    for (i, c) in full_body.chars().enumerate() {
        match (c, state) {
            ('<', 0) => { // Start of tag
                state = 1;
                tag_start_idx = i;
            },
            ('>', 0) => panic!("Error at character {i}: Unexpected '>'"),// No end of tag before start of tag
            ('>', 8) => {
                panic!("Didn't expect end of tag in state 8");
            },
            ('>', _) => { // End of tag
                if state != -1 {
                    let atag = ATag { 
                        start_idx: tag_start_idx, 
                        id: None, // TODO: Include parsing of id
                        href: current_href
                    };
                    current_href = "".to_string();
                    atags.push(atag);
                }
                state = 0;
            },
            ('a', 1) => {
                state = 2;
                a_tags_idx.push(tag_start_idx);
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
                current_href = String::from_iter(hrefbuilder.iter());
                hrefbuilder = Vec::new();
                state = 2;
            },
            (_, 8) => {
                hrefbuilder.push(c);
            },
            _ => () 
        }
    }

    println!("Number of a's {}", &atags.len());
    for atag in atags.iter() {
        println!("{:?}", atag);
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


}

#[cfg(test)]
mod parse_to_uri_tests {
    use http::{Uri, uri::Scheme};

    use crate::parse_to_uri;

    #[test]
    fn https_and_authority() {
        let result = parse_to_uri("https://github.com/");

        assert!(result.is_ok());

        let uri = result.unwrap();

        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(uri.path(), "/");
        assert_eq!(uri.query(), None);
        assert_eq!(uri.host(), Some("github.com"));
    }

    #[test]
    fn http_and_authority_with_subdomain() {
        let result = parse_to_uri("http://www.github.com/");

        assert!(result.is_ok());

        let uri = result.unwrap();

        assert_eq!(uri.scheme_str(), Some("http"));
        assert_eq!(uri.path(), "/");
        assert_eq!(uri.query(), None);
        assert_eq!(uri.host(), Some("www.github.com"));
    }

    #[test]
    fn no_final_slash_parses_to_slash_in_path() {
        let result = parse_to_uri("https://github.com");

        assert!(result.is_ok());

        let uri = result.unwrap();

        assert_eq!(uri.scheme_str(), Some("https"));
        assert_eq!(uri.path(), "/");
        assert_eq!(uri.query(), None);
        assert_eq!(uri.host(), Some("github.com"));
    }

    #[test]
    fn non_http_scheme_is_error() {
        let result = parse_to_uri("mailto:frehaa@github.com");
        assert!(result.is_err());
    }

}

#[cfg(test)]
mod moretests {
    #[test]
    fn it_also_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}