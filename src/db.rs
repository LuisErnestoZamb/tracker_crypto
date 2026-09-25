use crate::config::Config;
use crate::models::{Account, AccountType, NewAccount, NewTransaction, ProcessedArkhamData};
use crate::schema::{accounts, mv_user_wallets_groups_export, transactions};
use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_async::RunQueryDsl;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type DbConn = SyncConnectionWrapper<SqliteConnection>;
pub type DbPool = Pool<DbConn>;

type AccountTypeRow = (Option<i32>, Option<i32>, Option<i32>, Option<i32>);

#[derive(Clone)]
pub struct Database {
    pool: DbPool,
}

impl Database {
    pub async fn new(config: &Config) -> Result<Self> {
        let db_url = format!("{}?mode=rwc", config.turso_url);
        Self::new_with_url(&db_url).await
    }

    pub async fn new_with_url(db_url: &str) -> Result<Self> {
        let is_in_memory = db_url.contains(":memory:");

        if is_in_memory {
            // For in-memory: run migrations on a direct sync connection first,
            // then use shared cache so pool connections see the same DB.
            let mut sync_conn = SqliteConnection::establish(db_url)
                .context("Failed to establish migration connection")?;
            sync_conn
                .run_pending_migrations(MIGRATIONS)
                .map_err(|e| anyhow::anyhow!("Failed to run pending migrations: {}", e))?;
            drop(sync_conn);

            // Reconnect with shared cache for the pool
            let shared_url = if db_url.contains('?') {
                format!("{}&cache=shared", db_url)
            } else {
                format!("{}?cache=shared", db_url)
            };
            let manager = AsyncDieselConnectionManager::<DbConn>::new(&shared_url);
            let pool = Pool::builder()
                .max_size(5)
                .build(manager)
                .await
                .context("Failed to create connection pool")?;
            Ok(Self { pool })
        } else {
            Self::run_migrations_sync(db_url)?;
            let manager = AsyncDieselConnectionManager::<DbConn>::new(db_url);
            let pool = Pool::builder()
                .max_size(5)
                .build(manager)
                .await
                .context("Failed to create connection pool")?;
            Ok(Self { pool })
        }
    }

    fn run_migrations_sync(db_url: &str) -> Result<()> {
        let mut conn = SqliteConnection::establish(db_url)
            .context("Failed to establish migration connection")?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| anyhow::anyhow!("Failed to run pending migrations: {}", e))?;
        Ok(())
    }

    pub async fn get_total_accounts(&self) -> Result<i64> {
        let mut conn = self.pool.get().await?;
        let count: i64 = accounts::table.count().get_result(&mut conn).await?;
        Ok(count)
    }

    pub async fn save_account(&self, data: &Account) -> Result<()> {
        let mut conn = self.pool.get().await?;

        let new_account = NewAccount {
            wallet: data.wallet.clone().unwrap_or_default(),
            exchange_name: data.exchange_name.clone(),
            is_exchange: data.is_exchange,
            is_contract: data.is_contract,
            is_tracked: data.is_tracked,
            transfer_in: data.transfer_in,
            transfer_out: data.transfer_out,
            transactions_tron: data.transactions_tron,
            balance_tron: data.balance_tron,
            deep: data.deep,
            payload_tronscan: data.payload_tronscan.clone(),
        };

        diesel::insert_into(accounts::table)
            .values(&new_account)
            .on_conflict(accounts::wallet)
            .do_update()
            .set((
                accounts::exchange_name.eq(&new_account.exchange_name),
                accounts::is_exchange.eq(new_account.is_exchange),
                accounts::is_contract.eq(new_account.is_contract),
                accounts::is_tracked.eq(new_account.is_tracked),
                accounts::transfer_in.eq(new_account.transfer_in),
                accounts::transfer_out.eq(new_account.transfer_out),
                accounts::transactions_tron.eq(new_account.transactions_tron),
                accounts::balance_tron.eq(new_account.balance_tron),
                accounts::payload_tronscan.eq(&new_account.payload_tronscan),
            ))
            .execute(&mut conn)
            .await
            .context("Failed to save account")?;
        Ok(())
    }

    pub async fn get_or_create_account_id(&self, wallet: &str, _is_receiver: i32) -> Result<i32> {
        let mut conn = self.pool.get().await?;

        let existing: Option<i32> = accounts::table
            .filter(accounts::wallet.eq(wallet))
            .select(accounts::id)
            .first(&mut conn)
            .await
            .optional()?;

        if let Some(id) = existing {
            return Ok(id);
        }

        let new_account = NewAccount {
            wallet: wallet.to_string(),
            exchange_name: None,
            is_exchange: None,
            is_contract: None,
            is_tracked: None,
            transfer_in: None,
            transfer_out: None,
            transactions_tron: None,
            balance_tron: None,
            deep: None,
            payload_tronscan: None,
        };

        diesel::insert_into(accounts::table)
            .values(&new_account)
            .execute(&mut conn)
            .await
            .context("Failed to create account")?;

        let id: i32 = accounts::table
            .filter(accounts::wallet.eq(wallet))
            .select(accounts::id)
            .first(&mut conn)
            .await?;

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

        let mut conn = self.pool.get().await?;

        let wallet_from_id = self.get_or_create_account_id(wallet_from, 0).await?;
        let wallet_to_id = self.get_or_create_account_id(wallet_to, 1).await?;

        let new_tx = NewTransaction {
            amount: amount.map(|a| a.to_string()),
            status: Some(status_val),
            approval_amount: approval_amount.map(|a| a.to_string()),
            block_timestamp: block_timestamp.map(|v| v as i32),
            block: block.map(|v| v as i32),
            wallet_from: Some(wallet_from.to_string()),
            wallet_to: Some(wallet_to.to_string()),
            hash_tx: hash_tx.to_string(),
            confirmed,
            contract_type: contract_type.map(|s| s.to_string()),
            contract_type_alt: contract_type_alt.and_then(|s| s.parse().ok()),
            revert,
            contract_ret: contract_ret.map(|s| s.to_string()),
            event_type: event_type.map(|s| s.to_string()),
            issue_address: issue_address.map(|s| s.to_string()),
            decimals,
            exchange_from: Some(String::new()),
            exchange_to: Some(String::new()),
            is_sent_to_exchange: Some(0),
            direction,
            wallet_from_id,
            wallet_to_id,
        };

        diesel::insert_into(transactions::table)
            .values(&new_tx)
            .execute(&mut conn)
            .await
            .context("Failed to register transaction")?;
        Ok(())
    }

    pub async fn exist_transaction(&self, hash: &str) -> Result<bool> {
        let mut conn = self.pool.get().await?;
        let count: i64 = transactions::table
            .filter(transactions::hash_tx.eq(hash))
            .count()
            .get_result(&mut conn)
            .await?;
        Ok(count > 0)
    }

    pub async fn last_row_timestamp(&self, wallet_address: &str) -> Result<Option<i32>> {
        let mut conn = self.pool.get().await?;

        let result: Option<Option<i32>> = transactions::table
            .filter(
                transactions::wallet_from
                    .eq(wallet_address)
                    .or(transactions::wallet_to.eq(wallet_address)),
            )
            .select(transactions::block_timestamp)
            .order_by(transactions::block_timestamp.desc())
            .first(&mut conn)
            .await
            .optional()?;

        Ok(result.flatten())
    }

    pub async fn is_wallet_on_old_database(&self, wallet_address: &str) -> Result<bool> {
        let mut conn = self.pool.get().await?;

        let exists: bool = diesel::select(diesel::dsl::exists(
            mv_user_wallets_groups_export::table
                .filter(mv_user_wallets_groups_export::cc_addresses.eq(wallet_address)),
        ))
        .get_result(&mut conn)
        .await?;

        Ok(exists)
    }

    pub async fn get_account_type(&self, wallet: &str) -> Result<AccountType> {
        let mut conn = self.pool.get().await?;

        let result: Option<AccountTypeRow> = accounts::table
            .filter(accounts::wallet.eq(wallet))
            .select((
                accounts::is_exchange,
                accounts::is_contract,
                accounts::is_exchange_arkm,
                accounts::is_contract_arkm,
            ))
            .first(&mut conn)
            .await
            .optional()?;

        match result {
            Some((is_exchange, is_contract, is_exchange_arkm, is_contract_arkm)) => {
                let is_ex = is_exchange.unwrap_or(0) == 1 || is_exchange_arkm.unwrap_or(0) == 1;
                let is_co = is_contract.unwrap_or(0) == 1 || is_contract_arkm.unwrap_or(0) == 1;
                Ok(AccountType {
                    exists: true,
                    is_exchange: is_ex,
                    is_contract: is_co,
                })
            }
            None => Ok(AccountType::default()),
        }
    }

    pub async fn accounts_to_check(&self, limit: i64, offset: i64) -> Result<Vec<Account>> {
        let mut conn = self.pool.get().await?;

        let results: Vec<Account> = accounts::table
            .filter(accounts::is_tracked.is_null())
            .filter(accounts::is_tracked_arkm.eq(1))
            .filter(accounts::is_receiver.eq(1))
            .order_by(accounts::id.asc())
            .limit(limit)
            .offset(offset)
            .select(Account::as_select())
            .load(&mut conn)
            .await?;

        Ok(results)
    }

    pub async fn accounts_to_check_for_arkm(&self) -> Result<Vec<String>> {
        let mut conn = self.pool.get().await?;

        let results: Vec<Option<String>> = accounts::table
            .filter(
                accounts::is_tracked_arkm
                    .is_null()
                    .or(accounts::is_tracked_arkm.eq(0)),
            )
            .order_by(accounts::id.asc())
            .limit(1000)
            .select(accounts::wallet)
            .load(&mut conn)
            .await?;

        Ok(results.into_iter().flatten().collect())
    }

    pub async fn accounts_to_check_without_amount(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Account>> {
        let mut conn = self.pool.get().await?;

        let results: Vec<Account> = accounts::table
            .filter(accounts::is_amount_collected.eq(0))
            .order_by(accounts::id.asc())
            .limit(limit)
            .offset(offset)
            .select(Account::as_select())
            .load(&mut conn)
            .await?;

        Ok(results)
    }

    pub async fn update_account_intelligence(&self, data: &ProcessedArkhamData) -> Result<()> {
        let mut conn = self.pool.get().await?;

        diesel::update(accounts::table.filter(accounts::wallet.eq(&data.wallet)))
            .set((
                accounts::is_tracked_arkm.eq(1),
                accounts::arkham_label.eq(&data.arkham_label),
                accounts::populated_tags.eq(&data.populated_tags),
                accounts::is_exchange_arkm.eq(data.is_exchange_arkm),
                accounts::is_contract_arkm.eq(data.is_contract_arkm),
                accounts::payload_arkm.eq(&data.payload_arkm),
            ))
            .execute(&mut conn)
            .await
            .context("Failed to update account intelligence")?;
        Ok(())
    }

    pub async fn update_amount_account(&self, wallet: &str, amount: f64) -> Result<()> {
        let mut conn = self.pool.get().await?;

        diesel::update(accounts::table.filter(accounts::wallet.eq(wallet)))
            .set((
                accounts::is_amount_collected.eq(1),
                accounts::total_usd_amount.eq(amount),
            ))
            .execute(&mut conn)
            .await
            .context("Failed to update amount account")?;
        Ok(())
    }

    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
}
