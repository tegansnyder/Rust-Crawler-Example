extern crate hyper;
extern crate select;
extern crate xhtmlchardet;
extern crate robotparser;
extern crate url;

use std::io::Read;
use hyper::client::Client;
use hyper::header::Connection;

use select::document::Document;
use select::predicate::*;

use std::string::String;

use url::Url;

use robotparser::RobotFileParser;

enum ParseOption {
    HostName,
    Port,
    Protocol,
    Username,
    Password,
    Path,
    Fragment,
    Query
}

fn get_parse_option(arg: &str) -> ParseOption {
    let option_prefix_chars: &[char] = &['-'];
    let trimmed_arg = arg.trim_matches(option_prefix_chars);

    match trimmed_arg {
        "host"     => ParseOption::HostName,
        "port"     => ParseOption::Port,
        "protocol" => ParseOption::Protocol,
        "password" => ParseOption::Password,
        "username" => ParseOption::Username,
        "path"     => ParseOption::Path,
        "fragment" => ParseOption::Fragment,
        "query"    => ParseOption::Query,
        "scheme"   => ParseOption::Protocol,
        _          => panic!("Invalid parsing option")
    }
}

fn parse_url(option: &str, url: &str) -> String {

    let result = url::Url::parse(url);

    match result {
        Err(e) => {
            return e.to_string();
        },
        _ => {}
    };

    let parsed = result.unwrap();

    let value = match get_parse_option(option) {
        ParseOption::HostName => parse_component(parsed.domain(), "hostname"),
        ParseOption::Port     => parse_component(parsed.port(), "port"),
        ParseOption::Protocol => parse_component(Some(parsed.scheme), "scheme"),
        ParseOption::Username => parse_component(parsed.username(), "username"),
        ParseOption::Password => parse_component(parsed.password(), "password"),
        ParseOption::Path     => parse_component(parsed.serialize_path(), "path"),
        ParseOption::Fragment => parse_component(parsed.fragment, "fragment"),
        ParseOption::Query    => parse_component(parsed.query, "query")
    };

    match value {
        Ok(v) => {
            let cval = v.to_string();
            return cval;
        }
        Err(e) => {
            return e.to_string();
        }
    };

}

fn parse_component<T: ToString>(option: Option<T>, description: &str) -> Result<String, String> {
    return match option {
        Some(x) => Ok(x.to_string()),
        None    => Err(format!("No {} found", description))
    }
}

fn crawl(url: &str) {

    // open up a new http client
    let client = Client::new();

    // creating an outgoing request
    let mut res = client.get(&*url)
        .header(Connection::close())
        .send().unwrap();

    // read the response.
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    // println!("Response: {}", res.status);
    // println!("Headers:\n{}", res.headers);
    // println!("Body:\n{}", body);

    let document = Document::from_str(&*body);

    for node in document.find(Attr("id", "navmenu")).find(Name("a")).iter() {
        println!("{} ({:?})", node.text(), node.attr("href").unwrap());
    }

}

fn main() {

    let url = "http://www.tegdesign.com/".to_string();
    let host = parse_url("host", &*url);
    let scheme = parse_url("scheme", &*url);
    let url_object = Url::parse(&url).unwrap();
    let relative_url = "/".to_string() + &url_object.path().unwrap().join("/");
    let base_url = scheme.to_string() + "://" + &*host;
    let robots_txt_url = base_url + "/robots.txt";

    // println!("url: {}", url);
    // println!("host: {}", host);
    // println!("scheme: {}", scheme);
    // println!("relative_url: {}", relative_url);
    // println!("base_url: {}", base_url);

    // check if we are allowed to crawl via robots.txt
    let parser = RobotFileParser::new(&*robots_txt_url);
    parser.read();
    if parser.can_fetch("*", &*relative_url) {
    	crawl(&*url);
    }

}