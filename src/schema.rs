use toasty::Model;

#[derive(Debug, Default, Model)]
#[table = "accounts"]
pub struct Account {
    #[key]
    #[auto]
    pub id: i64,

    #[unique]
    pub wallet: String,

    pub exchange_name: Option<String>,

    pub is_exchange: Option<i32>,

    pub is_contract: Option<i32>,

    pub is_tracked: Option<i32>,

    #[column("transferIn")]
    pub transfer_in: Option<i64>,

    #[column("transferOut")]
    pub transfer_out: Option<i64>,

    #[column("transactionsTron")]
    pub transactions_tron: Option<i64>,

    #[column("balanceTron")]
    pub balance_tron: Option<i64>,

    pub deep: Option<i32>,

    pub payload_tronscan: Option<String>,

    pub is_exchange_arkm: Option<i32>,

    pub is_contract_arkm: Option<i32>,

    pub is_tracked_arkm: Option<i32>,

    pub payload_arkm: Option<String>,

    pub arkham_label: Option<String>,

    pub populated_tags: Option<String>,

    pub mandatory_scan: Option<i32>,

    pub observations: Option<String>,

    pub is_receiver: Option<i32>,

    pub created_at: Option<i64>,

    pub updated_at: Option<i64>,

    pub is_amount_collected: Option<i32>,

    pub total_usd_amount: Option<f64>,
}

#[derive(Debug, Default, Model)]
#[table = "transactions"]
pub struct Transaction {
    #[key]
    pub hash_tx: String,

    pub amount: Option<String>,

    pub status: Option<i32>,

    pub approval_amount: Option<String>,

    pub block_timestamp: Option<i64>,

    pub block: Option<i64>,

    pub wallet_from: Option<String>,

    pub wallet_to: Option<String>,

    pub wallet_from_id: i64,

    pub wallet_to_id: i64,

    pub confirmed: Option<i32>,

    pub contract_type: Option<String>,

    #[column("contractType")]
    pub contract_type_alt: Option<i32>,

    pub revert: Option<i32>,

    pub contract_ret: Option<String>,

    pub event_type: Option<String>,

    pub issue_address: Option<String>,

    pub decimals: Option<i32>,

    pub exchange_from: Option<String>,

    pub exchange_to: Option<String>,

    pub is_sent_to_exchange: Option<i32>,

    pub direction: Option<i32>,

    pub updated_at: Option<i64>,
}

#[derive(Debug, Default, Model)]
#[table = "mv_user_wallets_groups_export"]
pub struct LegacyWallet {
    #[key]
    pub cc_addresses: String,
}
