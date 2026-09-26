use crate::config::Config;
use crate::schema::{Account, LegacyWallet, Transaction};
use anyhow::{Context, Result};
use toasty_driver_turso::Turso;

#[derive(Clone)]
pub struct Database {
    db: toasty::Db,
}

impl Database {
    pub async fn new(config: &Config) -> Result<Self> {
        let url = if let Some(ref token) = config.turso_auth_token {
            format!("{}?authToken={}", config.turso_database_url, token)
        } else {
            config.turso_database_url.clone()
        };

        let builder =
            Turso::new(&url).context("Failed to create Turso connection")?;

        let db = toasty::Db::builder()
            .models(toasty::models![Account, Transaction, LegacyWallet])
            .build(builder)
            .await
            .context("Failed to build Toasty database")?;

        Self::ensure_schema(&db).await?;

        Ok(Self { db })
    }

    pub async fn new_local(db_path: &str) -> Result<Self> {
        let builder = Turso::file(db_path);

        let db = toasty::Db::builder()
            .models(toasty::models![Account, Transaction, LegacyWallet])
            .build(builder)
            .await
            .context("Failed to build local Toasty database")?;

        Self::ensure_schema(&db).await?;

        Ok(Self { db })
    }

    pub async fn new_in_memory() -> Result<Self> {
        let driver = Turso::in_memory();

        let db = toasty::Db::builder()
            .models(toasty::models![Account, Transaction, LegacyWallet])
            .build(driver)
            .await
            .context("Failed to build in-memory Toasty database")?;

        Self::ensure_schema(&db).await?;

        Ok(Self { db })
    }

    async fn ensure_schema(db: &toasty::Db) -> Result<()> {
        let mut conn = db.connection().await?;

        let rows = toasty::sql::query(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='accounts'",
        )
        .exec(&mut conn)
        .await
        .context("Failed to check for existing tables")?;

        if rows.is_empty() {
            db.push_schema().await.context("Failed to push schema")?;
        }

        Ok(())
    }

    pub fn db(&self) -> &toasty::Db {
        &self.db
    }

    pub fn db_mut(&mut self) -> &mut toasty::Db {
        &mut self.db
    }

    pub async fn push(&self) -> Result<()> {
        self.db.push_schema().await.context("Failed to push schema")?;
        Ok(())
    }

    pub async fn get_total_accounts(&self) -> Result<i64> {
        let mut db = self.db.clone();
        let rows = toasty::sql::query("SELECT COUNT(*) FROM accounts")
            .exec(&mut db)
            .await
            .context("Failed to count accounts")?;

        if let Some(toasty::stmt::Value::Record(cols)) = rows.first() {
            if let Some(toasty::stmt::Value::I64(count)) = cols.first() {
                return Ok(*count);
            }
        }

        Ok(0)
    }

    pub async fn save_account(&self, data: &Account) -> Result<()> {
        let mut db = self.db.clone();
        Account::upsert_by_wallet(&data.wallet)
            .on_create(|acct| {
                acct.exchange_name(data.exchange_name.clone())
                    .is_exchange(data.is_exchange)
                    .is_contract(data.is_contract)
                    .is_tracked(data.is_tracked)
                    .transfer_in(data.transfer_in)
                    .transfer_out(data.transfer_out)
                    .transactions_tron(data.transactions_tron)
                    .balance_tron(data.balance_tron)
                    .deep(data.deep)
                    .payload_tronscan(data.payload_tronscan.clone())
            })
            .on_update(|acct| {
                acct.exchange_name(data.exchange_name.clone())
                    .is_exchange(data.is_exchange)
                    .is_contract(data.is_contract)
                    .is_tracked(data.is_tracked)
                    .transfer_in(data.transfer_in)
                    .transfer_out(data.transfer_out)
                    .transactions_tron(data.transactions_tron)
                    .balance_tron(data.balance_tron)
                    .payload_tronscan(data.payload_tronscan.clone())
            })
            .exec(&mut db)
            .await
            .context("Failed to save account")?;
        Ok(())
    }

    pub async fn get_or_create_account_id(&self, wallet: &str, _is_receiver: i32) -> Result<i64> {
        let mut db = self.db.clone();

        if let Some(existing) = Account::filter_by_wallet(wallet)
            .select(Account::fields().id())
            .first()
            .exec(&mut db)
            .await?
        {
            return Ok(existing);
        }

        toasty::create!(Account { wallet: wallet.to_string() })
            .exec(&mut db)
            .await
            .context("Failed to create account")?;

        let id = Account::filter_by_wallet(wallet)
            .select(Account::fields().id())
            .first()
            .exec(&mut db)
            .await?
            .context("Failed to get new account ID")?;

        Ok(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn register_transaction(
        &self,
        wallet_from: &str,
        wallet_to: &str,
        amount: Option<f64>,
        status: Option<i32>,
        approval_amount: Option<f64>,
        block_timestamp: Option<i64>,
        block: Option<i64>,
        hash_tx: &str,
        confirmed: Option<i32>,
        contract_type: Option<&str>,
        contract_type_alt: Option<&str>,
        revert: Option<i32>,
        contract_ret: Option<&str>,
        event_type: Option<&str>,
        issue_address: Option<&str>,
        decimals: Option<i32>,
        direction: Option<i32>,
    ) -> Result<()> {
        let status_val = status.unwrap_or(-5);
        if status_val != -5 {
            return Ok(());
        }

        let wallet_from_id = self.get_or_create_account_id(wallet_from, 0).await?;
        let wallet_to_id = self.get_or_create_account_id(wallet_to, 1).await?;

        let amount_str = amount.map(|a| a.to_string());
        let approval_str = approval_amount.map(|a| a.to_string());
        let ct_alt = contract_type_alt.and_then(|s| s.parse::<i32>().ok());

        let mut db = self.db.clone();
        toasty::create!(Transaction {
            hash_tx: hash_tx.to_string(),
            wallet_from: Some(wallet_from.to_string()),
            wallet_to: Some(wallet_to.to_string()),
            amount: amount_str,
            status: Some(status_val),
            approval_amount: approval_str,
            block_timestamp,
            block,
            wallet_from_id,
            wallet_to_id,
            confirmed,
            contract_type: contract_type.map(|s| s.to_string()),
            contract_type_alt: ct_alt,
            revert,
            contract_ret: contract_ret.map(|s| s.to_string()),
            event_type: event_type.map(|s| s.to_string()),
            issue_address: issue_address.map(|s| s.to_string()),
            decimals,
            direction,
            exchange_from: Some(String::new()),
            exchange_to: Some(String::new()),
            is_sent_to_exchange: Some(0),
        })
        .exec(&mut db)
        .await
        .context("Failed to register transaction")?;

        Ok(())
    }

    pub async fn exist_transaction(&self, hash: &str) -> Result<bool> {
        let mut db = self.db.clone();
        let rows = toasty::sql::query("SELECT COUNT(*) FROM transactions WHERE hash_tx = ?1")
            .bind(hash)
            .exec(&mut db)
            .await
            .context("Failed to check transaction")?;

        if let Some(toasty::stmt::Value::Record(cols)) = rows.first() {
            if let Some(toasty::stmt::Value::I64(count)) = cols.first() {
                return Ok(*count > 0);
            }
        }

        Ok(false)
    }

    pub async fn last_row_timestamp(&self, wallet_address: &str) -> Result<Option<i64>> {
        let mut db = self.db.clone();
        let rows = toasty::sql::query(
            "SELECT t.block_timestamp
             FROM transactions t
             JOIN accounts a_from ON a_from.id = t.wallet_from_id
             JOIN accounts a_to   ON a_to.id   = t.wallet_to_id
             WHERE a_to.wallet = ?1 OR a_from.wallet = ?1
             ORDER BY t.block_timestamp DESC
             LIMIT 1",
        )
        .bind(wallet_address)
        .exec(&mut db)
        .await
        .context("Failed to get last timestamp")?;

        if let Some(toasty::stmt::Value::Record(cols)) = rows.first() {
            if let Some(toasty::stmt::Value::I64(ts)) = cols.first() {
                return Ok(Some(*ts));
            }
        }

        Ok(None)
    }

    pub async fn is_wallet_on_old_database(&self, wallet_address: &str) -> Result<bool> {
        let mut db = self.db.clone();
        let result = LegacyWallet::filter_by_cc_addresses(wallet_address)
            .first()
            .exec(&mut db)
            .await
            .context("Failed to check old database")?;

        Ok(result.is_some())
    }

    pub async fn get_account_type(
        &self,
        wallet: &str,
    ) -> Result<(bool, bool, bool)> {
        let mut db = self.db.clone();
        let rows = toasty::sql::query(
            "SELECT is_exchange, is_contract, is_exchange_arkm, is_contract_arkm
             FROM accounts WHERE wallet = ?1 LIMIT 1",
        )
        .bind(wallet)
        .exec(&mut db)
        .await
        .context("Failed to get account type")?;

        if let Some(toasty::stmt::Value::Record(cols)) = rows.first() {
            let is_exchange = cols.first().and_then(|v| match v {
                toasty::stmt::Value::I64(x) => Some(*x as i32),
                toasty::stmt::Value::I32(x) => Some(*x),
                _ => None,
            });
            let is_contract = cols.get(1).and_then(|v| match v {
                toasty::stmt::Value::I64(x) => Some(*x as i32),
                toasty::stmt::Value::I32(x) => Some(*x),
                _ => None,
            });
            let is_exchange_arkm = cols.get(2).and_then(|v| match v {
                toasty::stmt::Value::I64(x) => Some(*x as i32),
                toasty::stmt::Value::I32(x) => Some(*x),
                _ => None,
            });
            let is_contract_arkm = cols.get(3).and_then(|v| match v {
                toasty::stmt::Value::I64(x) => Some(*x as i32),
                toasty::stmt::Value::I32(x) => Some(*x),
                _ => None,
            });

            let is_ex = is_exchange.unwrap_or(0) == 1 || is_exchange_arkm.unwrap_or(0) == 1;
            let is_co = is_contract.unwrap_or(0) == 1 || is_contract_arkm.unwrap_or(0) == 1;

            return Ok((true, is_ex, is_co));
        }

        Ok((false, false, false))
    }

    pub async fn accounts_to_check(&self, limit: i64, offset: i64) -> Result<Vec<Account>> {
        let mut db = self.db.clone();
        let rows = toasty::sql::query(
            "SELECT id, wallet, exchange_name, is_exchange, is_contract, is_tracked,
                    transferIn, transferOut, transactionsTron, balanceTron, deep,
                    payload_tronscan, is_exchange_arkm, is_contract_arkm, is_tracked_arkm,
                    payload_arkm, arkham_label, populated_tags, mandatory_scan,
                    observations, is_receiver, created_at, updated_at,
                    is_amount_collected, total_usd_amount
             FROM accounts
             WHERE is_tracked IS NULL AND is_tracked_arkm = 1 AND is_receiver = 1
             ORDER BY id ASC
             LIMIT ?1 OFFSET ?2",
        )
        .bind(limit)
        .bind(offset)
        .exec(&mut db)
        .await
        .context("Failed to query accounts to check")?;

        let mut accounts = Vec::new();
        for row in rows {
            if let toasty::stmt::Value::Record(cols) = row {
                accounts.push(parse_account_from_cols(&cols)?);
            }
        }

        Ok(accounts)
    }

    pub async fn accounts_to_check_for_arkm(&self) -> Result<Vec<String>> {
        let mut db = self.db.clone();
        let rows = toasty::sql::query(
            "SELECT wallet FROM accounts
             WHERE is_tracked_arkm IS NULL OR is_tracked_arkm = 0
             ORDER BY id ASC
             LIMIT 1000",
        )
        .exec(&mut db)
        .await
        .context("Failed to query accounts for arkm")?;

        let mut wallets = Vec::new();
        for row in rows {
            if let toasty::stmt::Value::Record(cols) = row {
                if let Some(toasty::stmt::Value::String(w)) = cols.first() {
                    wallets.push(w.clone());
                }
            }
        }

        Ok(wallets)
    }

    pub async fn accounts_to_check_without_amount(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Account>> {
        let mut db = self.db.clone();
        let rows = toasty::sql::query(
            "SELECT id, wallet, exchange_name, is_exchange, is_contract, is_tracked,
                    transferIn, transferOut, transactionsTron, balanceTron, deep,
                    payload_tronscan, is_exchange_arkm, is_contract_arkm, is_tracked_arkm,
                    payload_arkm, arkham_label, populated_tags, mandatory_scan,
                    observations, is_receiver, created_at, updated_at,
                    is_amount_collected, total_usd_amount
             FROM accounts
             WHERE is_amount_collected = 0
             ORDER BY id ASC
             LIMIT ?1 OFFSET ?2",
        )
        .bind(limit)
        .bind(offset)
        .exec(&mut db)
        .await
        .context("Failed to query accounts without amount")?;

        let mut accounts = Vec::new();
        for row in rows {
            if let toasty::stmt::Value::Record(cols) = row {
                accounts.push(parse_account_from_cols(&cols)?);
            }
        }

        Ok(accounts)
    }

    pub async fn update_account_intelligence(
        &self,
        wallet: &str,
        arkham_label: &Option<String>,
        populated_tags: &Option<String>,
        is_exchange_arkm: i32,
        is_contract_arkm: i32,
        payload_arkm: &str,
    ) -> Result<()> {
        let mut db = self.db.clone();
        Account::filter_by_wallet(wallet)
            .update()
            .is_tracked_arkm(Some(1))
            .arkham_label(arkham_label.clone())
            .populated_tags(populated_tags.clone())
            .is_exchange_arkm(Some(is_exchange_arkm))
            .is_contract_arkm(Some(is_contract_arkm))
            .payload_arkm(Some(payload_arkm.to_string()))
            .exec(&mut db)
            .await
            .context("Failed to update account intelligence")?;
        Ok(())
    }

    pub async fn update_amount_account(&self, wallet: &str, amount: f64) -> Result<()> {
        let mut db = self.db.clone();
        Account::filter_by_wallet(wallet)
            .update()
            .is_amount_collected(Some(1))
            .total_usd_amount(Some(amount))
            .exec(&mut db)
            .await
            .context("Failed to update amount account")?;
        Ok(())
    }
}

fn parse_account_from_cols(cols: &[toasty::stmt::Value]) -> Result<Account> {
    let get_i64 = |idx: usize| -> Option<i64> {
        cols.get(idx).and_then(|v| match v {
            toasty::stmt::Value::I64(x) => Some(*x),
            toasty::stmt::Value::I32(x) => Some(*x as i64),
            toasty::stmt::Value::Null => None,
            _ => None,
        })
    };

    let get_i32 = |idx: usize| -> Option<i32> {
        cols.get(idx).and_then(|v| match v {
            toasty::stmt::Value::I64(x) => Some(*x as i32),
            toasty::stmt::Value::I32(x) => Some(*x),
            toasty::stmt::Value::Null => None,
            _ => None,
        })
    };

    let get_text = |idx: usize| -> Option<String> {
        cols.get(idx).and_then(|v| match v {
            toasty::stmt::Value::String(x) => Some(x.clone()),
            toasty::stmt::Value::Null => None,
            _ => None,
        })
    };

    let get_f64 = |idx: usize| -> Option<f64> {
        cols.get(idx).and_then(|v| match v {
            toasty::stmt::Value::F64(x) => Some(*x),
            toasty::stmt::Value::F32(x) => Some(*x as f64),
            toasty::stmt::Value::I64(x) => Some(*x as f64),
            toasty::stmt::Value::I32(x) => Some(*x as f64),
            toasty::stmt::Value::Null => None,
            _ => None,
        })
    };

    Ok(Account {
        id: get_i64(0).unwrap_or(0),
        wallet: get_text(1).unwrap_or_default(),
        exchange_name: get_text(2),
        is_exchange: get_i32(3),
        is_contract: get_i32(4),
        is_tracked: get_i32(5),
        transfer_in: get_i64(6),
        transfer_out: get_i64(7),
        transactions_tron: get_i64(8),
        balance_tron: get_i64(9),
        deep: get_i32(10),
        payload_tronscan: get_text(11),
        is_exchange_arkm: get_i32(12),
        is_contract_arkm: get_i32(13),
        is_tracked_arkm: get_i32(14),
        payload_arkm: get_text(15),
        arkham_label: get_text(16),
        populated_tags: get_text(17),
        mandatory_scan: get_i32(18),
        observations: get_text(19),
        is_receiver: get_i32(20),
        created_at: get_i64(21),
        updated_at: get_i64(22),
        is_amount_collected: get_i32(23),
        total_usd_amount: get_f64(24),
    })
}
