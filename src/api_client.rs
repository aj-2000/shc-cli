use async_recursion::async_recursion;

use crate::consts;
use crate::models::{RefreshTokenResponse, ShcFileResponse};

pub struct ApiClient {
    api_base_url: String,
    access_token: String,
    refresh_token: String,
    tried_refreshing_token: bool,
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new(access_token: &str, refresh_token: &str) -> ApiClient {
        ApiClient {
            api_base_url: consts::SHC_BACKEND_API_BASE_URL.to_string(),
            access_token: access_token.to_string(),
            refresh_token: refresh_token.to_string(),
            tried_refreshing_token: false,
            client: reqwest::Client::new(),
        }
    }

    pub fn login_again(&mut self) {
        self.tried_refreshing_token = true;
        println!("Logged out, please login again");
        // TODO: logout - clear config
        // TODO: run login command (can we continue after login command?)
        std::process::exit(1);
    }

    async fn refresh_token(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.tried_refreshing_token {
            self.login_again();
        }

        let res = self
            .client
            .get(format!("{}/auth/refresh-token", self.api_base_url))
            .header("Authorization", &self.refresh_token)
            .send()
            .await?;

        match res.status() {
            reqwest::StatusCode::OK => match res.json::<RefreshTokenResponse>().await {
                Ok(res) => {
                    println!("Refreshed token");
                    self.access_token = res.access_token;
                    self.refresh_token = res.refresh_token;
                }
                Err(e) => {
                    return Err(e.into());
                }
            },
            _ => {
                print!("Error refreshing token: {:?}", res.status());
                self.login_again();
            }
        }
        Ok(())
    }

    #[async_recursion]
    pub async fn list_files(
        &mut self,
        search: &str,
    ) -> Result<ShcFileResponse, Box<dyn std::error::Error>> {
        let res = self
            .client
            .get(format!(
                "{}/api/files?search={}&page=1&limit=100",
                self.api_base_url, search
            ))
            .header("Authorization", &self.access_token)
            .send()
            .await?;

        match res.status() {
            reqwest::StatusCode::OK => {
                let res = res.json::<ShcFileResponse>().await?;
                Ok(res)
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                self.refresh_token().await?;
                return self.list_files(search).await;
            }
            _ => {
                // TODO: use server error message
                Err(std::io::Error::new(std::io::ErrorKind::Other, "Something went wrong").into())
            }
        }
    }
}
