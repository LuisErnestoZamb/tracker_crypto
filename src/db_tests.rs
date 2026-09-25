#[cfg(test)]
mod tests {
    use crate::db::Database;
    use crate::models::{Account, ProcessedArkhamData};
    use tempfile::NamedTempFile;

    async fn setup_db() -> (Database, NamedTempFile) {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let db = Database::new_with_url(&format!("{}?mode=rwc", path))
            .await
            .unwrap();
        (db, tmp)
    }

    #[tokio::test]
    async fn test_get_total_accounts_empty() {
        let (db, _tmp) = setup_db().await;
        let total = db.get_total_accounts().await.unwrap();
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_save_and_retrieve_account() {
        let (db, _tmp) = setup_db().await;

        let account = Account {
            id: 0,
            wallet: Some("TTestWallet".to_string()),
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
        let (db, _tmp) = setup_db().await;

        let account1 = Account {
            id: 0,
            wallet: Some("TTestWallet".to_string()),
            exchange_name: Some("Exchange1".to_string()),
            is_tracked: Some(1),
            ..Default::default()
        };
        db.save_account(&account1).await.unwrap();

        let account2 = Account {
            id: 0,
            wallet: Some("TTestWallet".to_string()),
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
        let (db, _tmp) = setup_db().await;

        let id1 = db.get_or_create_account_id("TWallet1", 0).await.unwrap();
        let id2 = db.get_or_create_account_id("TWallet2", 1).await.unwrap();

        assert_ne!(id1, id2);

        let id3 = db.get_or_create_account_id("TWallet1", 0).await.unwrap();
        assert_eq!(id1, id3);
    }

    #[tokio::test]
    async fn test_register_and_exist_transaction() {
        let (db, _tmp) = setup_db().await;

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
        let (db, _tmp) = setup_db().await;

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
        let (db, _tmp) = setup_db().await;

        assert!(!db.is_wallet_on_old_database("TUnknown").await.unwrap());

        // Insert directly via Diesel
        use crate::schema::mv_user_wallets_groups_export;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        let mut conn = db.pool().get().await.unwrap();
        diesel::insert_into(mv_user_wallets_groups_export::table)
            .values(mv_user_wallets_groups_export::cc_addresses.eq("TKnownWallet"))
            .execute(&mut conn)
            .await
            .unwrap();

        assert!(db.is_wallet_on_old_database("TKnownWallet").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_account_type() {
        let (db, _tmp) = setup_db().await;

        let account_type = db.get_account_type("TUnknown").await.unwrap();
        assert!(!account_type.exists);

        let account = Account {
            id: 0,
            wallet: Some("TExchange".to_string()),
            is_exchange: Some(1),
            is_contract: Some(0),
            is_tracked: Some(1),
            ..Default::default()
        };
        db.save_account(&account).await.unwrap();

        let account_type = db.get_account_type("TExchange").await.unwrap();
        assert!(account_type.exists);
        assert!(account_type.is_exchange);
        assert!(!account_type.is_contract);
    }

    #[tokio::test]
    async fn test_accounts_to_check() {
        let (db, _tmp) = setup_db().await;

        use crate::schema::accounts as acc;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        let mut conn = db.pool().get().await.unwrap();

        // TW1: is_tracked=NULL, is_tracked_arkm=1, is_receiver=1 -> should match
        diesel::insert_into(acc::table)
            .values((
                acc::wallet.eq("TW1"),
                acc::is_tracked.eq::<Option<i32>>(None),
                acc::is_tracked_arkm.eq(1),
                acc::is_receiver.eq(1),
            ))
            .execute(&mut conn)
            .await
            .unwrap();

        // TW2: is_tracked=1, is_tracked_arkm=1, is_receiver=1 -> should NOT match
        diesel::insert_into(acc::table)
            .values((
                acc::wallet.eq("TW2"),
                acc::is_tracked.eq(1),
                acc::is_tracked_arkm.eq(1),
                acc::is_receiver.eq(1),
            ))
            .execute(&mut conn)
            .await
            .unwrap();

        // TW3: is_tracked=NULL, is_tracked_arkm=NULL, is_receiver=1 -> should NOT match
        diesel::insert_into(acc::table)
            .values((
                acc::wallet.eq("TW3"),
                acc::is_tracked.eq::<Option<i32>>(None),
                acc::is_tracked_arkm.eq::<Option<i32>>(None),
                acc::is_receiver.eq(1),
            ))
            .execute(&mut conn)
            .await
            .unwrap();

        drop(conn);

        let result = db.accounts_to_check(10, 0).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].wallet.as_deref(), Some("TW1"));
    }

    #[tokio::test]
    async fn test_update_account_intelligence() {
        let (db, _tmp) = setup_db().await;

        let account = Account {
            id: 0,
            wallet: Some("TWallet".to_string()),
            is_tracked: Some(1),
            ..Default::default()
        };
        db.save_account(&account).await.unwrap();

        let intel = ProcessedArkhamData {
            wallet: "TWallet".to_string(),
            arkham_label: Some("Binance".to_string()),
            populated_tags: Some(r#"[{"label":"exchange"}]"#.to_string()),
            is_exchange_arkm: 1,
            is_contract_arkm: 0,
            payload_arkm: "{}".to_string(),
        };

        db.update_account_intelligence(&intel).await.unwrap();

        let account_type = db.get_account_type("TWallet").await.unwrap();
        assert!(account_type.is_exchange);
    }

    #[tokio::test]
    async fn test_update_amount_account() {
        let (db, _tmp) = setup_db().await;

        let account = Account {
            id: 0,
            wallet: Some("TWallet".to_string()),
            is_tracked: Some(1),
            ..Default::default()
        };
        db.save_account(&account).await.unwrap();

        db.update_amount_account("TWallet", 12345.67).await.unwrap();
    }

    #[tokio::test]
    async fn test_last_row_timestamp() {
        let (db, _tmp) = setup_db().await;

        let result = db.last_row_timestamp("TWallet").await.unwrap();
        assert!(result.is_none());
    }
}
