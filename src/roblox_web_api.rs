use std::{borrow::Cow, fmt};

use reqwest::{
    header::{HeaderValue, COOKIE},
    Client, Request, Response, StatusCode,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ImageUploadData<'a> {
    pub image_data: Cow<'a, [u8]>,
    pub name: &'a str,
    pub description: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UploadResponse {
    pub success: bool,
    pub asset_id: u64,
    pub backing_asset_id: u64,
}

pub struct RobloxApiClient {
    auth_token: String,
    csrf_token: Option<HeaderValue>,
    client: Client,
}

impl fmt::Debug for RobloxApiClient {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "RobloxApiClient>")
    }
}

impl RobloxApiClient {
    pub fn new(auth_token: String) -> Self {
        Self {
            auth_token,
            csrf_token: None,
            client: Client::new(),
        }
    }

    pub fn upload_image(
        &mut self,
        data: ImageUploadData,
    ) -> Result<UploadResponse, RobloxApiError> {
        let url = "https://data.roblox.com/data/upload/json?assetTypeId=13";

        let mut response = self.execute_with_csrf_retry(|client| {
            Ok(client
                .post(url)
                .query(&[("name", data.name), ("description", data.description)])
                .body(data.image_data.clone().into_owned())
                .build()?)
        })?;

        let body = response.text().unwrap();

        if response.status().is_success() {
            Ok(serde_json::from_str(&body)?)
        } else {
            Err(RobloxApiError::ResponseError {
                status: response.status(),
                body,
            })
        }
    }

    fn execute_with_csrf_retry<F>(&mut self, make_request: F) -> Result<Response, RobloxApiError>
    where
        F: Fn(&Client) -> Result<Request, RobloxApiError>,
    {
        let mut request = make_request(&self.client)?;
        self.attach_headers(&mut request);

        let response = self.client.execute(request)?;

        match response.status() {
            StatusCode::FORBIDDEN => {
                if let Some(csrf) = response.headers().get("X-CSRF-Token") {
                    log::debug!("Retrying request with X-CSRF-Token...");

                    self.csrf_token = Some(csrf.clone());

                    let mut new_request = make_request(&self.client)?;
                    self.attach_headers(&mut new_request);

                    Ok(self.client.execute(new_request)?)
                } else {
                    Ok(response)
                }
            }
            _ => Ok(response),
        }
    }

    fn attach_headers(&self, request: &mut Request) {
        let cookie_value = format!(".ROBLOSECURITY={}", self.auth_token);

        request.headers_mut().insert(
            COOKIE,
            HeaderValue::from_bytes(cookie_value.as_bytes()).unwrap(),
        );

        if let Some(csrf) = &self.csrf_token {
            request.headers_mut().insert("X-CSRF-Token", csrf.clone());
        }
    }
}

#[derive(Debug, Error)]
pub enum RobloxApiError {
    #[error("Roblox API HTTP error")]
    Http {
        #[from]
        source: reqwest::Error,
    },

    #[error("Roblox API returned success, but had malformed JSON response")]
    BadResponseJson {
        #[from]
        source: serde_json::Error,
    },

    #[error("Roblox API returned HTTP {status} with body: {body}")]
    ResponseError { status: StatusCode, body: String },
}
