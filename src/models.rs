use serde::{Deserialize, Serialize};

// ============================================================
// Helper structs
// ============================================================

#[derive(Debug, Clone, Default)]
pub struct AccountType {
    pub exists: bool,
    pub is_exchange: bool,
    pub is_contract: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ProcessedArkhamData {
    pub wallet: String,
    pub arkham_label: Option<String>,
    pub populated_tags: Option<String>,
    pub is_exchange_arkm: i32,
    pub is_contract_arkm: i32,
    pub payload_arkm: String,
}

// ============================================================
// API response structs (unchanged)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronScanAccountResponse {
    pub address: Option<String>,
    #[serde(rename = "addressTag")]
    pub address_tag: Option<String>,
    #[serde(rename = "publicTag")]
    pub public_tag: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "accountType")]
    pub account_type: Option<i32>,
    #[serde(rename = "contractMap")]
    pub contract_map: Option<serde_json::Value>,
    #[serde(rename = "transactions_in")]
    pub transactions_in: Option<i64>,
    #[serde(rename = "transactions_out")]
    pub transactions_out: Option<i64>,
    #[serde(rename = "totalTransactionCount")]
    pub total_transaction_count: Option<i64>,
    #[serde(rename = "transactions")]
    pub transactions: Option<i64>,
    #[serde(rename = "balanceStr")]
    pub balance_str: Option<String>,
    pub balance: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronScanTransferResponse {
    pub code: Option<i32>,
    #[serde(rename = "page_size")]
    pub page_size: Option<i32>,
    pub data: Option<Vec<TronScanTransaction>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronScanTransaction {
    pub amount: Option<String>,
    pub status: Option<i32>,
    #[serde(rename = "approval_amount")]
    pub approval_amount: Option<String>,
    #[serde(rename = "block_timestamp")]
    pub block_timestamp: Option<i64>,
    pub block: Option<i64>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub hash: Option<String>,
    pub confirmed: Option<i32>,
    #[serde(rename = "contract_type")]
    pub contract_type: Option<String>,
    #[serde(rename = "contractType")]
    pub contract_type_alt: Option<String>,
    pub revert: Option<i32>,
    #[serde(rename = "contract_ret")]
    pub contract_ret: Option<String>,
    #[serde(rename = "event_type")]
    pub event_type: Option<String>,
    #[serde(rename = "issue_address")]
    pub issue_address: Option<String>,
    pub decimals: Option<i32>,
    pub direction: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronScanWalletAmount {
    #[serde(rename = "totalValueInUsd")]
    pub total_value_in_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArkhamBatchRequest {
    pub addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArkhamBatchResponse {
    pub addresses: Option<serde_json::Value>,
}
