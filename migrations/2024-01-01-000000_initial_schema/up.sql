CREATE TABLE accounts (
    id INTEGER PRIMARY KEY,
    wallet TEXT UNIQUE,
    exchange_name TEXT,
    is_exchange INTEGER,
    is_contract INTEGER,
    is_tracked INTEGER,
    transferIn INTEGER,
    transferOut INTEGER,
    transactionsTron INTEGER,
    balanceTron INTEGER,
    deep INTEGER,
    payload_tronscan TEXT,
    is_exchange_arkm INTEGER,
    is_contract_arkm INTEGER,
    is_tracked_arkm INTEGER,
    payload_arkm TEXT,
    arkham_label TEXT,
    populated_tags TEXT,
    mandatory_scan INTEGER,
    observations TEXT,
    is_receiver INTEGER,
    created_at INTEGER DEFAULT (strftime('%s','now')),
    updated_at INTEGER,
    is_amount_collected INTEGER DEFAULT 0,
    total_usd_amount REAL DEFAULT 0.0
);

CREATE TABLE transactions (
    amount TEXT,
    status INTEGER,
    approval_amount TEXT,
    block_timestamp INTEGER,
    block INTEGER,
    wallet_from TEXT,
    wallet_to TEXT,
    hash_tx TEXT PRIMARY KEY,
    confirmed INTEGER,
    contract_type TEXT,
    contractType INTEGER,
    revert INTEGER,
    contract_ret TEXT,
    event_type TEXT,
    issue_address TEXT,
    decimals INTEGER,
    exchange_from TEXT,
    exchange_to TEXT,
    is_sent_to_exchange INTEGER,
    direction INTEGER,
    updated_at INTEGER,
    wallet_from_id INTEGER NOT NULL,
    wallet_to_id INTEGER NOT NULL,
    FOREIGN KEY (wallet_from_id) REFERENCES accounts(id),
    FOREIGN KEY (wallet_to_id) REFERENCES accounts(id)
);

CREATE TABLE mv_user_wallets_groups_export (
    cc_addresses TEXT
);

CREATE TRIGGER accounts_set_updated_at
AFTER UPDATE ON accounts
FOR EACH ROW
BEGIN
    UPDATE accounts
    SET updated_at = strftime('%s','now')
    WHERE id = OLD.id;
END;

CREATE TRIGGER transac_set_updated_at
AFTER UPDATE ON transactions
FOR EACH ROW
BEGIN
    UPDATE transactions
    SET updated_at = strftime('%s','now')
    WHERE hash_tx = OLD.hash_tx;
END;

CREATE INDEX idx_accounts_wallet ON accounts(wallet);
CREATE INDEX idx_accounts_tracked ON accounts(is_tracked, id);
CREATE INDEX idx_accounts_tracked_arkm ON accounts(is_tracked_arkm, id);
CREATE INDEX idx_transactions_from_id ON transactions(wallet_from_id);
CREATE INDEX idx_transactions_to_id ON transactions(wallet_to_id);
CREATE INDEX idx_transactions_timestamp ON transactions(block_timestamp DESC);
