use crate::config::Config;
use reqwest::header;
use reqwest::Response;
use serde::de::DeserializeOwned;
use std::error::Error;

// Wrapper around reqwest to make requests to the controller.
pub struct Client {
    client: reqwest::Client,
    base_url: reqwest::Url,
}

impl Client {
    // Create a new client with the given config.
    pub fn new(config: &Config) -> Result<Client, Box<dyn std::error::Error>> {
        let base_url = reqwest::Url::parse(&config.controller_url)?;

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Accept",
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "Content-Type",
            header::HeaderValue::from_static("application/json"),
        );

        Ok(Client {
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()?,
            base_url,
        })
    }

    // Send a request to the controller.
    async fn send_request(
        &self,
        endpoint: &str,
        method: reqwest::Method,
        body: Option<&str>,
    ) -> Result<Response, Box<dyn Error>> {
        let url = self.base_url.join(endpoint)?;
        let mut request = (*self).client.request(method, url);

        if let Some(body) = body {
            request = request.body(body.to_owned());
        }

        request
            .send()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }

    // Send a request to the controller and deserialize the response.
    pub async fn send_json_request<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        method: reqwest::Method,
        body: Option<&str>,
    ) -> Result<T, Box<dyn Error>> {
        let response = self.send_request(endpoint, method, body).await?;
        response
            .json::<T>()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }
}
