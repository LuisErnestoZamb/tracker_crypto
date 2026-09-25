// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (id) {
        id -> Integer,
        wallet -> Nullable<Text>,
        exchange_name -> Nullable<Text>,
        is_exchange -> Nullable<Integer>,
        is_contract -> Nullable<Integer>,
        is_tracked -> Nullable<Integer>,
        #[sql_name = "transferIn"]
        transfer_in -> Nullable<Integer>,
        #[sql_name = "transferOut"]
        transfer_out -> Nullable<Integer>,
        #[sql_name = "transactionsTron"]
        transactions_tron -> Nullable<Integer>,
        #[sql_name = "balanceTron"]
        balance_tron -> Nullable<Integer>,
        deep -> Nullable<Integer>,
        payload_tronscan -> Nullable<Text>,
        is_exchange_arkm -> Nullable<Integer>,
        is_contract_arkm -> Nullable<Integer>,
        is_tracked_arkm -> Nullable<Integer>,
        payload_arkm -> Nullable<Text>,
        arkham_label -> Nullable<Text>,
        populated_tags -> Nullable<Text>,
        mandatory_scan -> Nullable<Integer>,
        observations -> Nullable<Text>,
        is_receiver -> Nullable<Integer>,
        created_at -> Nullable<Integer>,
        updated_at -> Nullable<Integer>,
        is_amount_collected -> Nullable<Integer>,
        total_usd_amount -> Nullable<Double>,
    }
}

diesel::table! {
    mv_user_wallets_groups_export (cc_addresses) {
        cc_addresses -> Nullable<Text>,
    }
}

diesel::table! {
    transactions (hash_tx) {
        amount -> Nullable<Text>,
        status -> Nullable<Integer>,
        approval_amount -> Nullable<Text>,
        block_timestamp -> Nullable<Integer>,
        block -> Nullable<Integer>,
        wallet_from -> Nullable<Text>,
        wallet_to -> Nullable<Text>,
        hash_tx -> Text,
        confirmed -> Nullable<Integer>,
        contract_type -> Nullable<Text>,
        #[sql_name = "contractType"]
        contract_type_alt -> Nullable<Integer>,
        revert -> Nullable<Integer>,
        contract_ret -> Nullable<Text>,
        event_type -> Nullable<Text>,
        issue_address -> Nullable<Text>,
        decimals -> Nullable<Integer>,
        exchange_from -> Nullable<Text>,
        exchange_to -> Nullable<Text>,
        is_sent_to_exchange -> Nullable<Integer>,
        direction -> Nullable<Integer>,
        updated_at -> Nullable<Integer>,
        wallet_from_id -> Integer,
        wallet_to_id -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(accounts, mv_user_wallets_groups_export, transactions,);
