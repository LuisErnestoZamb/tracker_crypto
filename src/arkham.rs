use crate::config::Config;
use crate::models::{ArkhamBatchRequest, ArkhamBatchResponse, ProcessedArkhamData};
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;

const ARKHAM_BASE_URL: &str = "https://api.arkm.com";

pub struct ArkhamClient {
    client: Client,
    api_key: Option<String>,
}

impl ArkhamClient {
    pub fn new(config: &Config) -> Self {
        let client = Client::new();
        Self {
            client,
            api_key: config.arkm_api_key.clone(),
        }
    }

    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    pub async fn get_batch_intelligence(
        &self,
        addresses: &[String],
    ) -> Result<ArkhamBatchResponse> {
        let api_key = self
            .api_key
            .as_deref()
            .context("ARKM_API_KEY not set")?;

        let url = format!("{}/intelligence/address_enriched/batch", ARKHAM_BASE_URL);

        let request = ArkhamBatchRequest {
            addresses: addresses.to_vec(),
        };

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("API-Key", api_key)
            .json(&request)
            .send()
            .await
            .context("Failed to send Arkham batch request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Arkham API error: {} {}", status, text);
        }

        let data: ArkhamBatchResponse = response
            .json()
            .await
            .context("Failed to parse Arkham batch response")?;

        Ok(data)
    }

    pub fn parse_arkham_data(response: &ArkhamBatchResponse) -> Vec<ProcessedArkhamData> {
        let addresses = match &response.addresses {
            Some(addr) => addr,
            None => return vec![],
        };

        let map = match addresses.as_object() {
            Some(m) => m,
            None => return vec![],
        };

        map.iter()
            .map(|(wallet, data)| {
                let arkham_label = data
                    .get("arkhamLabel")
                    .and_then(|l| l.get("name"))
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string());

                let populated_tags = data
                    .get("populatedTags")
                    .and_then(|t| t.as_array())
                    .map(|arr| {
                        let tags: Vec<Value> = arr.clone();
                        serde_json::to_string(&tags).unwrap_or_default()
                    });

                let is_contract_arkm = data
                    .get("contract")
                    .and_then(|c| c.as_bool())
                    .map(|b| if b { 1 } else { 0 })
                    .unwrap_or(0);

                let is_exchange = data
                    .get("arkhamEntity")
                    .and_then(|e| e.get("type"))
                    .and_then(|t| t.as_str())
                    .map(|t| t == "cex")
                    .unwrap_or(false)
                    || data
                        .get("populatedTags")
                        .and_then(|t| t.as_array())
                        .map(|tags| {
                            tags.iter().any(|tag| {
                                tag.get("label")
                                    .and_then(|l| l.as_str())
                                    .map(|label| {
                                        let lower = label.to_lowercase();
                                        lower.contains("exchange")
                                            || lower.contains("hot wallet")
                                            || lower.contains("high transacting")
                                    })
                                    .unwrap_or(false)
                            })
                        })
                        .unwrap_or(false);

                let raw_payload = serde_json::to_string(data).unwrap_or_default();

                ProcessedArkhamData {
                    wallet: wallet.clone(),
                    arkham_label,
                    populated_tags,
                    is_exchange_arkm: if is_exchange { 1 } else { 0 },
                    is_contract_arkm,
                    payload_arkm: raw_payload,
                }
            })
            .collect()
    }
}
