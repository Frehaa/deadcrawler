use http::uri::Scheme;
use http::{Uri, Error, StatusCode};
use hyper::{client::Client, body::HttpBody};
use hyper_tls::HttpsConnector;
use url::Url;

use std::env;
use std::collections::{HashSet, HashMap};

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

struct DeadResults {
    address : String,
    status_code: ResponseStatusCode
}

fn report_results(results: DeadResults) {
    match results.status_code {
        ResponseStatusCode::NOT_FOUND => {
            println!("{} was not found.", results.address);
        },
        ResponseStatusCode::OK => {
            println!("Everything all right");
        },
        ResponseStatusCode::OTHER => {
            println!("I don't know what happend for {}", results.address);
        },
    }

}

// TODO: Check for subdomains myself or not? Do I want to crawl over github if I find a link to it? Maybe I should instead make a separate crawl for subdomains. Or maybe just make it toggleable
fn main() -> () { //Result<(), Box<dyn std::error::Error + Send + Sync>>{
    let result = parse_to_uri("https://github.com");

    /* 
    1. Take as input a string (needs at least one argument on command line or fails) 

    2. Try to parse string as a URL and save it as the base URL (if fail the program halts with error)

    3. Make HTTPS GET request for URL (try once and report error on fail. A fail
    in this case is something which does not even give an error code)

    4. Given the result of a page, if it is an error code we finish and report
    it, and on success we save a key-value pair of the url and content, parse
    the content for all links (a link is the href attribute in an <a> tag) and
    save them.
    
    5. For each link, parse the link as a URL, either a relative path or
    absolute. If the link is relative then combine it with the base from where
    it was extracted from (For each URL we need to know all the links it found
    and make the links proper URLs if they are relative by combining them). At
    the end we should have a lot of new candidate URLs.
    
    6. For each proper URL, make an HTTPS GET request for the content, if the 
    URL is in the same domain as the base URL then we will parse the content 
    of the page for links like in step 4., if the URL is external we simply 
    save the result code. 

    7. Keep going until all links have been traversed. 

    ### Another explanation:
    
    We have a domain and a queue of unvisited URLs. For each of these unvisited
    URLs, we make an HTTPS GET request. The request will fail or return a
    response, in the case of a fail we (TBD? retry? silently ignore?
    diagnostics?), and in the case of a response we get a status code. For 404
    (or similar?) we mark the URL as dead, for relocated we mark it as such, and
    for success it depends on whether the current URL is part of the domain. If
    it is not part of the domain we are examining then we just mark the status
    code, if it part of the domain then we continue our crawl by traversing the
    content of the response to find new URLs. If the URL is relative then we
    need to combine it with the base URL of the request URL. All of the newfound
    URLs are added to the queue if they have not already been visited, so we
    also keep track of all URLs we have already visited. 
    
    */
    let uri = result.unwrap();

    println!("{:?}", uri.scheme_str());
    println!("{:?}", uri.path());
    println!("{:?}", uri.query());
    println!("{:?}", uri.host());



    return;

    // 1. Take as input a string (needs at least one argument on command line or
    // fails) 
    let mut args = env::args();

    let address = args.nth(1);
    if address.is_none() {
        println!("Error: No address given.");
        return;
    }
    let address = unsafe { address.unwrap_unchecked() };

    // 2. Try to parse string as a URL and save it as the base URL (if fail the
    // program halts with error)
    let base_uri = parse_to_uri(&address);
    if let Err(e) = base_uri {
        println!("Error: {}", e);
        return;
    }

    // 3. Make HTTPS GET request for URL (try once and report error on fail. A fail
    // in this case is something which does not even give an error code)
    let data = get_data(unsafe { base_uri.unwrap_unchecked() });
    if let Err(e) = data {
        println!("Something went wrong {}", e);
        return;
    }
    let data = unsafe { data.unwrap_unchecked() };

    // 4. Given the result of a page, if it is an error code we finish and report
    // it, and ...
    match data.statusCode {
        ResponseStatusCode::NOT_FOUND => {
            let results = DeadResults{ 
                address: uri.to_string(), 
                status_code: data.statusCode
            };
            report_results(results);
            return;
        },
        ResponseStatusCode::OK => {
            // Continue
        },
        ResponseStatusCode::OTHER => {
            let results = DeadResults{ 
                address: uri.to_string(), 
                status_code: data.statusCode
            };
            report_results(results);
            return;
        }
    }


    // ... on success we save a key-value pair of the url and content, ... 
    let mut address_to_content = HashMap::new();
    address_to_content.insert(uri.to_string(), data);

    // ...parse the content for all links (a link is the href attribute in an
    // <a> tag) and save them.
    let full_body = include_str!("itu.dk.txt"); // Placeholder 
    // let full_body = data.body.unwrap();

    let mut links = get_links(full_body);

    // 5. For each link, parse the link as a URL, either a relative path or
    // absolute. If the link is relative then combine it with the base from where
    // it was extracted from (For each URL we need to know all the links it found
    // and make the links proper URLs if they are relative by combining them). At
    // the end we should have a lot of new candidate URLs.
    let mut candidate_urls = Vec::new();
    for link in links {
        let mut result_link = link;
        if is_relative_path_link(link) {
            result_link = combine_links(base_uri, link);
        } else {
            result_link = result_link;
        }
        candidate_urls.push(result_link);
    }

    // 6. For each proper URL, make an HTTPS GET request for the content, if the 
    // URL is in the same domain as the base URL then we will parse the content 
    // of the page for links like in step 4., if the URL is external we simply 
    // save the result code. 

    // 7. Keep going until all links have been traversed. 
}

fn combine_links(base_uri: Result<Uri, &str>, link: Link) -> Link {
    todo!()
}

fn is_relative_path_link(link: Link) -> bool {
    todo!()
}

struct Link {
    address: String
}

fn get_links(full_body: &str) -> Vec<Link> { // To be updated. Right now ignores the data
    let mut result = Vec::new();

    // Might be an easy regex for this since double-quote (") characters are not allowed in a URI
    // https://www.ietf.org/rfc/rfc2396.txt (Just skimmed, might be wrong)
    // Candidate regex (the group would be the link):    <a .*href="(.*)">
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

    return result;
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