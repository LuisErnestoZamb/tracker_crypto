#[cfg(test)]
mod tests {
    use crate::db::Database;
    use crate::schema::Account;

    async fn setup_db() -> Database {
        Database::new_in_memory().await.unwrap()
    }

    #[tokio::test]
    async fn test_get_total_accounts_empty() {
        let db = setup_db().await;
        let total = db.get_total_accounts().await.unwrap();
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_save_and_retrieve_account() {
        let db = setup_db().await;

        let account = Account {
            wallet: "TTestWallet".to_string(),
            exchange_name: Some("TestExchange".to_string()),
            is_exchange: Some(1),
            is_contract: Some(0),
            is_tracked: Some(1),
            transfer_in: Some(10),
            transfer_out: Some(5),
            transactions_tron: Some(100),
            balance_tron: Some(5000000),
            deep: Some(0),
            payload_tronscan: Some("{}".to_string()),
            ..Default::default()
        };

        db.save_account(&account).await.unwrap();

        let total = db.get_total_accounts().await.unwrap();
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn test_save_account_upsert() {
        let db = setup_db().await;

        let account1 = Account {
            wallet: "TTestWallet".to_string(),
            exchange_name: Some("Exchange1".to_string()),
            is_tracked: Some(1),
            ..Default::default()
        };
        db.save_account(&account1).await.unwrap();

        let account2 = Account {
            wallet: "TTestWallet".to_string(),
            exchange_name: Some("Exchange2".to_string()),
            is_tracked: Some(1),
            ..Default::default()
        };
        db.save_account(&account2).await.unwrap();

        let total = db.get_total_accounts().await.unwrap();
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn test_get_or_create_account_id() {
        let db = setup_db().await;

        let id1 = db.get_or_create_account_id("TWallet1", 0).await.unwrap();
        let id2 = db.get_or_create_account_id("TWallet2", 1).await.unwrap();

        assert_ne!(id1, id2);

        let id3 = db.get_or_create_account_id("TWallet1", 0).await.unwrap();
        assert_eq!(id1, id3);
    }

    #[tokio::test]
    async fn test_register_and_exist_transaction() {
        let db = setup_db().await;

        db.register_transaction(
            "TSender",
            "TReceiver",
            Some(100.0),
            Some(-5),
            None,
            None,
            None,
            "tx_hash_123",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        assert!(db.exist_transaction("tx_hash_123").await.unwrap());
        assert!(!db.exist_transaction("nonexistent").await.unwrap());
    }

    #[tokio::test]
    async fn test_register_transaction_skips_non_minus5_status() {
        let db = setup_db().await;

        db.register_transaction(
            "TSender",
            "TReceiver",
            Some(100.0),
            Some(1),
            None,
            None,
            None,
            "tx_hash_456",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        assert!(!db.exist_transaction("tx_hash_456").await.unwrap());
    }

    #[tokio::test]
    async fn test_is_wallet_on_old_database() {
        let mut db = setup_db().await;

        assert!(!db.is_wallet_on_old_database("TUnknown").await.unwrap());

        // Insert directly via raw SQL
        toasty::sql::query(
            "INSERT INTO mv_user_wallets_groups_export (cc_addresses) VALUES (?1)",
        )
        .bind("TKnownWallet")
        .exec(db.db_mut())
        .await
        .unwrap();

        assert!(db.is_wallet_on_old_database("TKnownWallet").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_account_type() {
        let db = setup_db().await;

        let (exists, _, _) = db.get_account_type("TUnknown").await.unwrap();
        assert!(!exists);

        let account = Account {
            wallet: "TExchange".to_string(),
            is_exchange: Some(1),
            is_contract: Some(0),
            is_tracked: Some(1),
            ..Default::default()
        };
        db.save_account(&account).await.unwrap();

        let (exists, is_ex, is_co) = db.get_account_type("TExchange").await.unwrap();
        assert!(exists);
        assert!(is_ex);
        assert!(!is_co);
    }

    #[tokio::test]
    async fn test_accounts_to_check() {
        let mut db = setup_db().await;

        // Insert test accounts directly via raw SQL
        toasty::sql::query(
            "INSERT INTO accounts (wallet, is_tracked, is_tracked_arkm, is_receiver) VALUES (?1, NULL, ?2, ?3)",
        )
        .bind("TW1")
        .bind(1)
        .bind(1)
        .exec(db.db_mut())
        .await
        .unwrap();

        toasty::sql::query(
            "INSERT INTO accounts (wallet, is_tracked, is_tracked_arkm, is_receiver) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind("TW2")
        .bind(1)
        .bind(1)
        .bind(1)
        .exec(db.db_mut())
        .await
        .unwrap();

        toasty::sql::query(
            "INSERT INTO accounts (wallet, is_tracked, is_tracked_arkm, is_receiver) VALUES (?1, NULL, NULL, ?2)",
        )
        .bind("TW3")
        .bind(1)
        .exec(db.db_mut())
        .await
        .unwrap();

        let result = db.accounts_to_check(10, 0).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].wallet, "TW1");
    }

    #[tokio::test]
    async fn test_update_account_intelligence() {
        let db = setup_db().await;

        let account = Account {
            wallet: "TWallet".to_string(),
            is_tracked: Some(1),
            ..Default::default()
        };
        db.save_account(&account).await.unwrap();

        db.update_account_intelligence(
            "TWallet",
            &Some("Binance".to_string()),
            &Some(r#"[{"label":"exchange"}]"#.to_string()),
            1,
            0,
            "{}",
        )
        .await
        .unwrap();

        let (_, is_ex, _) = db.get_account_type("TWallet").await.unwrap();
        assert!(is_ex);
    }

    #[tokio::test]
    async fn test_update_amount_account() {
        let db = setup_db().await;

        let account = Account {
            wallet: "TWallet".to_string(),
            is_tracked: Some(1),
            ..Default::default()
        };
        db.save_account(&account).await.unwrap();

        db.update_amount_account("TWallet", 12345.67).await.unwrap();
    }

    #[tokio::test]
    async fn test_last_row_timestamp() {
        let db = setup_db().await;

        let result = db.last_row_timestamp("TWallet").await.unwrap();
        assert!(result.is_none());
    }
}
