use crate::arkham::ArkhamClient;
use crate::config::Config;
use crate::db::Database;
use crate::models::Account;
use crate::tronscan::TronScanClient;
use anyhow::Result;
use std::path::Path;
use std::time::Duration;
use tracing::{error, info};

pub struct Tracker {
    pub db: Database,
    pub tronscan: TronScanClient,
    pub arkham: ArkhamClient,
    pub current_address: String,
    pub pagination: i64,
    pub block_timestamp: Option<i64>,
    pub should_continue: bool,
}

impl Tracker {
    pub fn new(config: Config, db: Database) -> Self {
        let tronscan = TronScanClient::new(&config);
        let arkham = ArkhamClient::new(&config);

        Self {
            db,
            tronscan,
            arkham,
            current_address: String::new(),
            pagination: 0,
            block_timestamp: None,
            should_continue: true,
        }
    }

    pub fn start_tracking(&mut self, address: &str) {
        self.pagination = 0;
        self.should_continue = true;
        self.current_address = address.to_string();
        self.block_timestamp = None;
        info!("wallet: {}", address);
    }

    pub async fn exists_accounts(&self) -> Result<bool> {
        let total = self.db.get_total_accounts().await?;
        Ok(total > 0)
    }

    pub async fn parser_account(&self) -> Result<()> {
        let data = self.tronscan.get_account_info(&self.current_address).await?;

        let exchange_name = data
            .address_tag
            .as_ref()
            .or(data.public_tag.as_ref())
            .or(data.name.as_ref())
            .filter(|s| s.as_str() != "CreatedByContract")
            .cloned()
            .unwrap_or_default();

        let is_exchange = if exchange_name.is_empty() { 0 } else { 1 };

        let is_contract = if data.account_type == Some(1) {
            1
        } else {
            match &data.contract_map {
                Some(map) => {
                    if let Some(obj) = map.as_object() {
                        if !obj.is_empty() {
                            1
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                }
                None => 0,
            }
        };

        let account = Account {
            id: 0,
            wallet: Some(self.current_address.clone()),
            exchange_name: Some(exchange_name),
            is_exchange: Some(is_exchange),
            is_contract: Some(is_contract),
            is_tracked: Some(1),
            transfer_in: Some(data.transactions_in.unwrap_or(0) as i32),
            transfer_out: Some(data.transactions_out.unwrap_or(0) as i32),
            transactions_tron: Some(
                data.total_transaction_count
                    .or(data.transactions)
                    .unwrap_or(0) as i32,
            ),
            balance_tron: Some(
                data.balance_str
                    .as_ref()
                    .and_then(|s| s.parse().ok())
                    .or(data.balance.as_ref().and_then(|b| b.as_i64()))
                    .unwrap_or(0) as i32,
            ),
            deep: Some(0),
            payload_tronscan: Some(serde_json::to_string(&data).unwrap_or_default()),
            is_exchange_arkm: None,
            is_contract_arkm: None,
            is_tracked_arkm: None,
            payload_arkm: None,
            arkham_label: None,
            populated_tags: None,
            mandatory_scan: None,
            observations: None,
            is_receiver: None,
            created_at: None,
            updated_at: None,
            is_amount_collected: None,
            total_usd_amount: None,
        };

        self.db.save_account(&account).await
    }

    pub async fn is_exchange(&self) -> Result<bool> {
        let account_type = self.db.get_account_type(&self.current_address).await?;
        Ok(account_type.is_exchange)
    }

    pub async fn is_registered_wallet(&self) -> Result<bool> {
        self.db
            .is_wallet_on_old_database(&self.current_address)
            .await
    }

    pub async fn track_wallet(&mut self) -> Result<()> {
        let data = self
            .tronscan
            .get_trc20_transfers(&self.current_address, self.pagination, self.block_timestamp)
            .await;

        let data = match data {
            Ok(d) => d,
            Err(e) => {
                error!("Error fetching transfers: {}", e);
                if self.block_timestamp.is_none() {
                    let last_ts = self.db.last_row_timestamp(&self.current_address).await?;
                    self.block_timestamp = last_ts.map(|v| v as i64);
                    self.pagination = 0;
                    self.should_continue = false;
                }
                return Ok(());
            }
        };

        if let Some(ref txs) = data.data {
            if txs.is_empty() && self.block_timestamp.is_none() {
                self.should_continue = false;
                info!("Aborting: {}", self.pagination);
                return Ok(());
            }
        }

        if let Some(txs) = &data.data {
            for tx in txs {
                let hash = tx.hash.as_deref().unwrap_or("");
                if !self.db.exist_transaction(hash).await? {
                    self.db
                        .register_transaction(
                            tx.from.as_deref().unwrap_or(""),
                            tx.to.as_deref().unwrap_or(""),
                            tx.amount,
                            tx.status,
                            tx.approval_amount,
                            tx.block_timestamp,
                            tx.block,
                            hash,
                            tx.confirmed,
                            tx.contract_type.as_deref(),
                            tx.contract_type_alt.as_deref(),
                            tx.revert,
                            tx.contract_ret.as_deref(),
                            tx.event_type.as_deref(),
                            tx.issue_address.as_deref(),
                            tx.decimals,
                            tx.direction,
                        )
                        .await?;
                }
            }
        }

        info!("Batch: {} {}", self.pagination, self.current_address);
        self.pagination += 50;
        self.should_continue = data.data.as_ref().is_some_and(|d| !d.is_empty());

        Ok(())
    }

    pub async fn get_total_amount_wallet(&self) -> Result<()> {
        let data = self
            .tronscan
            .get_wallet_amount(&self.current_address)
            .await?;

        if let Some(amount) = data.total_value_in_usd {
            self.db
                .update_amount_account(&self.current_address, amount)
                .await?;
        }

        Ok(())
    }

    pub async fn orchestrate_arkham_processing(&self) -> Result<()> {
        if !self.arkham.has_api_key() {
            return Ok(());
        }

        let mut has_more = true;

        while has_more {
            let addresses = self.db.accounts_to_check_for_arkm().await?;

            if addresses.is_empty() {
                has_more = false;
                info!("No more accounts pending for Arkham processing.");
                continue;
            }

            info!("Processing batch of {} accounts...", addresses.len());

            let payload_arkm = self.arkham.get_batch_intelligence(&addresses).await?;
            let parsed = ArkhamClient::parse_arkham_data(&payload_arkm);

            for data in &parsed {
                self.db.update_account_intelligence(data).await?;
            }

            info!("Processed {} accounts.", parsed.len());
        }

        Ok(())
    }

    pub async fn get_wallet_list(&mut self, limit: i64, offset: i64) -> Result<Vec<Account>> {
        if self.arkham.has_api_key() && file_exists("pull_arkm") {
            self.orchestrate_arkham_processing().await?;
            tokio::time::sleep(Duration::from_millis(1100)).await;
        }
        self.db.accounts_to_check(limit, offset).await
    }

    pub async fn get_untracked_amount_list(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Account>> {
        self.db.accounts_to_check_without_amount(limit, offset).await
    }
}

fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}
