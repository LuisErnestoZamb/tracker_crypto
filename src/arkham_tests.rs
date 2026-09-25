#[cfg(test)]
mod tests {
    use crate::arkham::ArkhamClient;
    use crate::models::ArkhamBatchResponse;
    use serde_json::json;

    #[test]
    fn test_parse_arkham_data_empty() {
        let response = ArkhamBatchResponse {
            addresses: Some(json!({})),
        };

        let parsed = ArkhamClient::parse_arkham_data(&response);
        assert!(parsed.is_empty());
    }

    #[test]
    fn test_parse_arkham_data_with_exchange() {
        let response = ArkhamBatchResponse {
            addresses: Some(json!({
                "TTestWallet123": {
                    "arkhamLabel": {"name": "Binance"},
                    "populatedTags": [{"label": "exchange"}],
                    "contract": false,
                    "arkhamEntity": {"type": "cex"}
                }
            })),
        };

        let parsed = ArkhamClient::parse_arkham_data(&response);
        assert_eq!(parsed.len(), 1);

        let data = &parsed[0];
        assert_eq!(data.wallet, "TTestWallet123");
        assert_eq!(data.arkham_label.as_deref(), Some("Binance"));
        assert_eq!(data.is_exchange_arkm, 1);
        assert_eq!(data.is_contract_arkm, 0);
    }

    #[test]
    fn test_parse_arkham_data_with_contract() {
        let response = ArkhamBatchResponse {
            addresses: Some(json!({
                "TContractAddr": {
                    "arkhamLabel": {"name": "Uniswap Router"},
                    "populatedTags": [],
                    "contract": true,
                    "arkhamEntity": {"type": "contract"}
                }
            })),
        };

        let parsed = ArkhamClient::parse_arkham_data(&response);
        assert_eq!(parsed.len(), 1);

        let data = &parsed[0];
        assert_eq!(data.wallet, "TContractAddr");
        assert_eq!(data.is_exchange_arkm, 0);
        assert_eq!(data.is_contract_arkm, 1);
    }

    #[test]
    fn test_parse_arkham_data_hot_wallet_label() {
        let response = ArkhamBatchResponse {
            addresses: Some(json!({
                "THotWallet": {
                    "arkhamLabel": {"name": "Unknown"},
                    "populatedTags": [{"label": "hot wallet"}],
                    "contract": false
                }
            })),
        };

        let parsed = ArkhamClient::parse_arkham_data(&response);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].is_exchange_arkm, 1);
    }

    #[test]
    fn test_parse_arkham_data_none_addresses() {
        let response = ArkhamBatchResponse { addresses: None };

        let parsed = ArkhamClient::parse_arkham_data(&response);
        assert!(parsed.is_empty());
    }

    #[test]
    fn test_parse_arkham_data_high_transacting() {
        let response = ArkhamBatchResponse {
            addresses: Some(json!({
                "THighTxWallet": {
                    "arkhamLabel": {"name": "Market Maker"},
                    "populatedTags": [{"label": "high transacting"}],
                    "contract": false
                }
            })),
        };

        let parsed = ArkhamClient::parse_arkham_data(&response);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].is_exchange_arkm, 1);
    }
}
