use async_recursion::async_recursion;
use std::io::{Error, ErrorKind};

use crate::app_config::AppConfig;
use crate::consts::{APP_CONFIG_PATH, SHC_BACKEND_API_BASE_URL};
use crate::models::{RefreshTokenResponse, ShcFile, ShcFileResponse};
pub struct ApiClient {
    api_base_url: String,
    tried_refreshing_token: bool,
    app_config: AppConfig,
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new() -> ApiClient {
        let config_path = dirs::home_dir().unwrap().join(APP_CONFIG_PATH);
        ApiClient {
            api_base_url: SHC_BACKEND_API_BASE_URL.to_string(),
            tried_refreshing_token: false,
            app_config: AppConfig::new(&config_path),
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
        let config_path = dirs::home_dir().unwrap().join(APP_CONFIG_PATH);

        let res = self
            .client
            .get(format!("{}/auth/refresh-token", self.api_base_url))
            .header(
                "Authorization",
                self.app_config.refresh_token.as_ref().unwrap(),
            )
            .send()
            .await?;

        match res.status() {
            reqwest::StatusCode::OK => match res.json::<RefreshTokenResponse>().await {
                Ok(res) => {
                    self.app_config.access_token = Some(res.access_token);
                    self.app_config.refresh_token = Some(res.refresh_token);
                    self.app_config.save(&config_path);
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
        let access_token = self.app_config.access_token.as_ref().unwrap();

        let res = self
            .client
            .get(format!(
                "{}/api/files?search={}&page=1&limit=100",
                self.api_base_url, search
            ))
            .header("Authorization", access_token)
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
                Err(Error::new(ErrorKind::Other, "Something went wrong").into())
            }
        }
    }

    #[async_recursion]
    pub async fn remove_file(&mut self, file_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let access_token = self.app_config.access_token.as_ref().unwrap();

        let res = self
            .client
            .delete(format!(
                "{}/api/files/remove/{}",
                self.api_base_url, file_id
            ))
            .header("Authorization", access_token)
            .send()
            .await?;

        match res.status() {
            reqwest::StatusCode::OK => Ok(()),
            reqwest::StatusCode::UNAUTHORIZED => {
                self.refresh_token().await?;
                return self.remove_file(file_id).await;
            }
            _ => Err(Error::new(ErrorKind::Other, "Something went wrong").into()),
        }
    }

    #[async_recursion]
    pub async fn toggle_file_visibility(
        &mut self,
        file_id: &str,
    ) -> Result<ShcFile, Box<dyn std::error::Error>> {
        let access_token = self.app_config.access_token.as_ref().unwrap();

        let res = self
            .client
            .patch(format!(
                "{}/api/files/toggle-visibility/{}",
                self.api_base_url, file_id
            ))
            .header("Authorization", access_token)
            .send()
            .await?;

        match res.status() {
            reqwest::StatusCode::OK => {
                let res = res.json::<ShcFile>().await?;
                Ok(res)
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                self.refresh_token().await?;
                return self.toggle_file_visibility(file_id).await;
            }
            _ => Err(Error::new(ErrorKind::Other, "Something went wrong").into()),
        }
    }
}
