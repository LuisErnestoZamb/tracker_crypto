DROP INDEX IF EXISTS idx_accounts_wallet;
DROP INDEX IF EXISTS idx_accounts_tracked;
DROP INDEX IF EXISTS idx_accounts_tracked_arkm;
DROP INDEX IF EXISTS idx_transactions_from_id;
DROP INDEX IF EXISTS idx_transactions_to_id;
DROP INDEX IF EXISTS idx_transactions_timestamp;

DROP TRIGGER IF EXISTS accounts_set_updated_at;
DROP TRIGGER IF EXISTS transac_set_updated_at;

DROP TABLE IF EXISTS mv_user_wallets_groups_export;
DROP TABLE IF EXISTS transactions;
DROP TABLE IF EXISTS accounts;
