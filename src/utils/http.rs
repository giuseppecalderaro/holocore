use reqwest;
use std::collections::HashMap;

// TODO-GSC: finish implementation
pub fn get(url: &str) {
    let client = reqwest::blocking::Client::new();
    match client.get(url).send() {
        Ok(res) => match res.status() {
            reqwest::StatusCode::OK => log::info!("GET succeeded"),
            _ => log::error!("GET failed")
        },
        Err(e) => log::error!("GET failed - {}", e)
    };
}

pub fn post(url: &str, body: HashMap<String, String>) {
    let client = reqwest::blocking::Client::new();

    match client.post(url).json(&body).send() {
        Ok(res) => match res.status() {
            reqwest::StatusCode::OK => log::info!("POST succeeded"),
            _ => log::error!("POST failed")
        },
        Err(e) => log::error!("POST failed - {}", e)
    };
}

pub fn delete(url: &str) {
    let client = reqwest::blocking::Client::new();
    match client.delete(url).send() {
        Ok(res) => match res.status() {
            reqwest::StatusCode::OK => log::info!("DELETE succeeded"),
            _ => log::error!("DELETE failed")
        },
        Err(e) => log::error!("DELETE failed - {}", e)
    }
}

pub fn get_name_from_uri(uri: &rocket::http::uri::Origin<'_>) -> String {
    let uri_string = uri.to_string();
    let tokens = uri_string.split('/').collect::<Vec<&str>>();
    String::from(tokens[1])
}
