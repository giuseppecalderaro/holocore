use crate::components::sinks::binance::api::API;
use crate::utils::time;

use hex::encode;
use hmac::{Hmac, Mac, NewMac};
use reqwest::{ Client, Response, StatusCode };
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT, CONTENT_TYPE};
use serde::de::DeserializeOwned;
use sha2::Sha256;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct BinanceSpotClient {
    base_url: String,
    api_key: HeaderValue,
    secret_key: String,
    user_agent: HeaderValue,
    content_type: HeaderValue,
    inner: Client,
}

impl BinanceSpotClient {
    pub fn new(base_url: &str, api_key: &str, secret_key: &str, user_agent: &str, content_type: &str) -> Self {
        let api_key = match HeaderValue::from_str(api_key) {
            Ok(api_key) => api_key,
            Err(e) => panic!("BinanceSpotClient: cannot create header value api_key - {}", e)
        };

        let content_type = match HeaderValue::from_str(content_type) {
            Ok(content_type) => content_type,
            Err(e) => panic!("BinanceSpotClient: cannot create header value content_type - {}", e)
        };

        let user_agent = match HeaderValue::from_str(user_agent) {
            Ok(user_agent) => user_agent,
            Err(e) => panic!("BinanceSpotClient: cannot create header value user_agent - {}", e)
        };

        Self {
            base_url: String::from(base_url),
            api_key,
            secret_key: String::from(secret_key),
            user_agent,
            content_type,
            inner: reqwest::Client::builder()
                .pool_idle_timeout(None)
                .build()
                .expect("Cannot create client"),
        }
    }

    pub async fn get_signed<T: DeserializeOwned>(&self, endpoint: API, request: Option<String>) -> Result<T, String> {
        let url = self.sign_request(endpoint, request);
        match self.inner
            .get(&url)
            .headers(self.build_headers(true))
            .send().await {
                Ok(response) => {
                    match self.handler(response).await {
                        Ok(obj) => Ok(obj),
                        Err(e) => Err(e)
                    }
                },
                Err(e) => Err(e.to_string())
            }
    }

    pub async fn post_signed<T: DeserializeOwned>(&self, endpoint: API, request: Option<String>) -> Result<T, String> {
        let url = self.sign_request(endpoint, request);
        log::debug!("post_signed: {}", url);

        match self.inner
            .post(&url)
            .headers(self.build_headers(true))
            .send().await {
                Ok(response) => {
                    match self.handler(response).await {
                        Ok(obj) => Ok(obj),
                        Err(e) => Err(e)
                    }
                },
                Err(e) => Err(e.to_string())
            }
    }

    pub async fn delete_signed<T: DeserializeOwned>(&self, endpoint: API, request: Option<String>) -> Result<T, String> {
        let url = self.sign_request(endpoint, request);
        log::debug!("delete_signed: {}", url);

        match self.inner
            .delete(&url)
            .headers(self.build_headers(true))
            .send().await {
                Ok(response) => {
                    match self.handler(response).await {
                        Ok(obj) => Ok(obj),
                        Err(e) => Err(e)
                    }
                },
                Err(e) => Err(e.to_string())
            }
    }

    // pub fn get<T: DeserializeOwned>(&self, endpoint: API, request: Option<String>) -> Result<T> {
    //     let mut url: String = format!("{}{}", self.host, String::from(endpoint));
    //     if let Some(request) = request {
    //         if !request.is_empty() {
    //             url.push_str(format!("?{}", request).as_str());
    //         }
    //     }

    //     let client = &self.inner_client;
    //     let response = client.get(url.as_str()).send()?;

    //     self.handler(response)
    // }

    // pub fn post<T: DeserializeOwned>(&self, endpoint: API) -> Result<T> {
    //     let url: String = format!("{}{}", self.host, String::from(endpoint));

    //     let client = &self.inner_client;
    //     let response = client
    //         .post(url.as_str())
    //         .headers(self.build_headers(false)?)
    //         .send()?;

    //     self.handler(response)
    // }

    // pub fn put<T: DeserializeOwned>(&self, endpoint: API, listen_key: &str) -> Result<T> {
    //     let url: String = format!("{}{}", self.host, String::from(endpoint));
    //     let data: String = format!("listenKey={}", listen_key);

    //     let client = &self.inner_client;
    //     let response = client
    //         .put(url.as_str())
    //         .headers(self.build_headers(false)?)
    //         .body(data)
    //         .send()?;

    //     self.handler(response)
    // }

    // pub fn delete<T: DeserializeOwned>(&self, endpoint: API, listen_key: &str) -> Result<T> {
    //     let url: String = format!("{}{}", self.host, String::from(endpoint));
    //     let data: String = format!("listenKey={}", listen_key);

    //     let client = &self.inner_client;
    //     let response = client
    //         .delete(url.as_str())
    //         .headers(self.build_headers(false)?)
    //         .body(data)
    //         .send()?;

    //     self.handler(response)
    // }

    async fn handler<T: DeserializeOwned>(&self, response: Response) -> Result<T, String> {
        let status = response.status();

        match status {
            StatusCode::OK => match response.json::<T>().await {
                Ok(json) => Ok(json),
                Err(e) => {
                    log::error!("{}", e);
                    Err(e.to_string())
                }
            },
            _ => {
                match response.text().await {
                    Ok(text) => Err(format!("Response {}: {}", status, text)),
                    Err(e) => Err(format!("Response {} - {}: Cannot extract text response ", status, e))
                }
            }
        }
    }

    fn build_headers(&self, has_content_type: bool) -> HeaderMap {
        let mut custom_headers = HeaderMap::new();

        custom_headers.insert(
            USER_AGENT,
            self.user_agent.clone()
        );

        if has_content_type {
            custom_headers.insert(
                CONTENT_TYPE,
                self.content_type.clone()
            );
        }

        custom_headers.insert(
            HeaderName::from_static("x-mbx-apikey"),
            self.api_key.clone()
        );

        custom_headers
    }

    pub fn build_request(mut parameters: BTreeMap<String, String>, recv_window: Option<u64>) -> Option<String> {
        match recv_window {
            Some(recv_window) => parameters.insert(String::from("recv_window"), recv_window.to_string()),
            None => None
        };

        parameters.insert(String::from("timestamp"), time::now_millis().to_string());

        let mut request = String::new();

        for (key, value) in parameters {
            let param = format!("{}={}&", key, value);
            request.push_str(param.as_ref());
        }

        request.pop();
        log::debug!("Built request {}", request);
        Some(request)
    }

    fn sign_request(&self, endpoint: API, request: Option<String>) -> String {
        let signature = match Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes()) {
            Ok(mut signed_key) => {
                match &request {
                    Some(request) => signed_key.update(request.as_bytes()),
                    None => ()
                };

                encode(signed_key.finalize().into_bytes())
            },
            Err(e) => {
                log::error!("BinanceSpotClient: failed to sign request - {}", e);
                String::new()
            }
        };

        let request_body: String = match &request {
            Some(request) => format!("{}&signature={}", &request, signature),
            None => format!("&signature={}", signature)
        };
        format!("{}{}?{}", self.base_url, String::from(endpoint), request_body)
    }
}
