use crate::config::Config;
use crate::models::{TronScanAccountResponse, TronScanTransferResponse, TronScanWalletAmount};
use anyhow::{Context, Result};
use reqwest::Client;

const TRONSCAN_BASE_URL: &str = "https://apilist.tronscanapi.com";
const USDT_CONTRACT: &str = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";

pub struct TronScanClient {
    client: Client,
    api_key: String,
}

impl TronScanClient {
    pub fn new(config: &Config) -> Self {
        let client = Client::new();
        Self {
            client,
            api_key: config.tron_token.clone(),
        }
    }

    pub async fn get_account_info(&self, address: &str) -> Result<TronScanAccountResponse> {
        let url = format!("{}/api/accountv2?address={}", TRONSCAN_BASE_URL, address);

        let response = self
            .client
            .get(&url)
            .header("TRON-PRO-API-KEY", &self.api_key)
            .send()
            .await
            .context("Failed to send account info request")?;

        let data: TronScanAccountResponse = response
            .json()
            .await
            .context("Failed to parse account info response")?;

        Ok(data)
    }

    pub async fn get_trc20_transfers(
        &self,
        address: &str,
        start: i64,
        block_timestamp: Option<i64>,
    ) -> Result<TronScanTransferResponse> {
        let timestamp_param = block_timestamp
            .map(|ts| format!("&start_timestamp={}", ts))
            .unwrap_or_default();

        let url = format!(
            "{}/api/transfer/trc20?address={}&trc20Id={}&start={}&limit=50&direction=1&reverse=false&db_version=1{}",
            TRONSCAN_BASE_URL, address, USDT_CONTRACT, start, timestamp_param
        );

        let response = self
            .client
            .get(&url)
            .header("TRON-PRO-API-KEY", &self.api_key)
            .send()
            .await
            .context("Failed to send TRC20 transfer request")?;

        let data: TronScanTransferResponse = response
            .json()
            .await
            .context("Failed to parse TRC20 transfer response")?;

        Ok(data)
    }

    pub async fn get_wallet_amount(&self, address: &str) -> Result<TronScanWalletAmount> {
        let url = format!(
            "{}/api/account/wallet?address={}",
            TRONSCAN_BASE_URL, address
        );

        let response = self
            .client
            .get(&url)
            .header("TRON-PRO-API-KEY", &self.api_key)
            .send()
            .await
            .context("Failed to send wallet amount request")?;

        let data: TronScanWalletAmount = response
            .json()
            .await
            .context("Failed to parse wallet amount response")?;

        Ok(data)
    }
}
