//! Miscellaneous actions for Solana Agent Kit
//!
//! Includes: CoinGecko market data, Helius transaction parsing, SNS domain resolution, and more.

use async_trait::async_trait;
use anyhow::Result;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::actions::{Action, ActionExample, ActionMetadata, ActionRegistry};
use crate::agent::Agent;

// =============================================================================
// GET_COINGECKO_TRENDING_TOKENS Action
// =============================================================================

#[derive(Debug)]
pub struct GetCoingeckoTrendingTokensAction {
    meta: ActionMetadata,
}

impl GetCoingeckoTrendingTokensAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({}),
            output: json!({
                "status": "success",
                "coins": [
                    {
                        "id": "solana",
                        "name": "Solana",
                        "symbol": "SOL",
                        "market_cap_rank": 5,
                    }
                ],
            }),
            explanation: "Get the trending tokens on CoinGecko".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_COINGECKO_TRENDING_TOKENS".to_string(),
            similes: vec![
                "trending tokens".to_string(),
                "hot tokens".to_string(),
                "popular tokens".to_string(),
                "coingecko trending".to_string(),
            ],
            description: "Get the trending tokens on CoinGecko - shows what's hot in the market".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetCoingeckoTrendingTokensAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, _input: Value) -> Result<Value> {
        let url = "https://api.coingecko.com/api/v3/search/trending";

        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("CoinGecko API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "result": data,
        }))
    }
}

// =============================================================================
// GET_COINGECKO_TOKEN_INFO Action
// =============================================================================

#[derive(Debug)]
pub struct GetCoingeckoTokenInfoAction {
    meta: ActionMetadata,
}

impl GetCoingeckoTokenInfoAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "tokenAddress": {
                    "type": "string",
                    "description": "The Solana token mint address",
                }
            },
            "required": ["tokenAddress"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "tokenAddress": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            }),
            output: json!({
                "status": "success",
                "data": {
                    "name": "USD Coin",
                    "symbol": "USDC",
                    "decimals": 6,
                },
            }),
            explanation: "Get USDC token info from CoinGecko".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_COINGECKO_TOKEN_INFO".to_string(),
            similes: vec![
                "get token info".to_string(),
                "token details".to_string(),
                "coingecko token".to_string(),
                "token information".to_string(),
            ],
            description: "Get detailed token information from CoinGecko including name, symbol, description, and social links".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetCoingeckoTokenInfoAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            tokenAddress: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        // Use the free CoinGecko API endpoint for Solana tokens
        let url = format!(
            "https://api.coingecko.com/api/v3/coins/solana/contract/{}",
            parsed.tokenAddress
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Failed to fetch token info: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "result": data,
        }))
    }
}

// =============================================================================
// GET_COINGECKO_TOKEN_PRICE Action
// =============================================================================

#[derive(Debug)]
pub struct GetCoingeckoTokenPriceAction {
    meta: ActionMetadata,
}

impl GetCoingeckoTokenPriceAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "tokenIds": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "CoinGecko token IDs (e.g., 'solana', 'bitcoin')",
                },
                "vsCurrencies": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Currencies to get price in (e.g., 'usd', 'eur')",
                }
            },
            "required": ["tokenIds"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "tokenIds": ["solana", "bitcoin"],
                "vsCurrencies": ["usd"],
            }),
            output: json!({
                "status": "success",
                "prices": {
                    "solana": { "usd": 150.25 },
                    "bitcoin": { "usd": 45000 },
                },
            }),
            explanation: "Get SOL and BTC prices in USD".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_COINGECKO_TOKEN_PRICE".to_string(),
            similes: vec![
                "get token price".to_string(),
                "check price".to_string(),
                "coingecko price".to_string(),
                "crypto price".to_string(),
            ],
            description: "Get current token prices from CoinGecko in various currencies".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetCoingeckoTokenPriceAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            tokenIds: Vec<String>,
            vsCurrencies: Option<Vec<String>>,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let vs_currencies = parsed.vsCurrencies.unwrap_or_else(|| vec!["usd".to_string()]);

        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies={}",
            parsed.tokenIds.join(","),
            vs_currencies.join(",")
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("CoinGecko API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "prices": data,
        }))
    }
}

// =============================================================================
// PARSE_TRANSACTION Action (Helius)
// =============================================================================

#[derive(Debug)]
pub struct ParseTransactionAction {
    meta: ActionMetadata,
}

impl ParseTransactionAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "transactionId": {
                    "type": "string",
                    "description": "The Solana transaction signature to parse",
                },
                "heliusApiKey": {
                    "type": "string",
                    "description": "Helius API key (optional if configured in agent)",
                }
            },
            "required": ["transactionId"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "transactionId": "4zZVvbgzcriyjAeEiK1w7CeDCt7gYThUCZat3ULTNerzKHF4WLfRG2YUjbRovfFJ639TAyARB4oyRDcLVUvrakq7",
            }),
            output: json!({
                "status": "success",
                "transaction": {
                    "type": "TRANSFER",
                    "source": "SYSTEM_PROGRAM",
                },
            }),
            explanation: "Parse a Solana transaction into human-readable format".to_string(),
        }];

        let meta = ActionMetadata {
            name: "PARSE_TRANSACTION".to_string(),
            similes: vec![
                "parse transaction".to_string(),
                "analyze transaction".to_string(),
                "decode transaction".to_string(),
                "inspect tx".to_string(),
            ],
            description: "Parse a Solana transaction to retrieve detailed, human-readable information using Helius Enhanced Transactions API".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for ParseTransactionAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            transactionId: String,
            heliusApiKey: Option<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let api_key = parsed.heliusApiKey.ok_or_else(|| {
            anyhow::anyhow!("Helius API key required. Pass heliusApiKey in input.")
        })?;

        let url = format!("https://api.helius.xyz/v0/transactions/?api-key={}", api_key);

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&json!({
                "transactions": [parsed.transactionId],
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Helius API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "transaction": data,
            "message": format!("Successfully parsed transaction: {}", parsed.transactionId),
        }))
    }
}

// =============================================================================
// RESOLVE_SOL_DOMAIN Action (Bonfida SNS)
// =============================================================================

#[derive(Debug)]
pub struct ResolveSolDomainAction {
    meta: ActionMetadata,
}

impl ResolveSolDomainAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "domain": {
                    "type": "string",
                    "description": "The .sol domain to resolve (with or without .sol suffix)",
                }
            },
            "required": ["domain"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({ "domain": "toly.sol" }),
            output: json!({
                "status": "success",
                "owner": "7nxQB...",
                "message": "Successfully resolved toly.sol",
            }),
            explanation: "Resolve a .sol domain to get the owner's wallet address".to_string(),
        }];

        let meta = ActionMetadata {
            name: "RESOLVE_SOL_DOMAIN".to_string(),
            similes: vec![
                "resolve sol domain".to_string(),
                "lookup sol domain".to_string(),
                "get domain owner".to_string(),
                "find sol address".to_string(),
                "resolve .sol".to_string(),
            ],
            description: "Resolve a .sol domain to its corresponding Solana wallet address using Bonfida Name Service".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for ResolveSolDomainAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            domain: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        // Strip .sol suffix if present
        let domain_name = parsed.domain.trim_end_matches(".sol");

        // Use Bonfida's public API
        let url = format!(
            "https://sns-sdk-proxy.bonfida.workers.dev/resolve/{}",
            domain_name
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Failed to resolve domain: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        if let Some(result) = data.get("result").and_then(|r| r.as_str()) {
            Ok(json!({
                "status": "success",
                "owner": result,
                "message": format!("Successfully resolved {}.sol", domain_name),
            }))
        } else {
            Ok(json!({
                "status": "error",
                "message": "Domain not found or invalid response",
            }))
        }
    }
}

// =============================================================================
// GET_DOMAIN_FOR_WALLET Action (Reverse SNS lookup)
// =============================================================================

#[derive(Debug)]
pub struct GetDomainForWalletAction {
    meta: ActionMetadata,
}

impl GetDomainForWalletAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "walletAddress": {
                    "type": "string",
                    "description": "The Solana wallet address to look up",
                }
            },
            "required": ["walletAddress"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "walletAddress": "86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY",
            }),
            output: json!({
                "status": "success",
                "domain": "toly.sol",
            }),
            explanation: "Get the .sol domain associated with a wallet address".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_DOMAIN_FOR_WALLET".to_string(),
            similes: vec![
                "reverse domain lookup".to_string(),
                "get wallet domain".to_string(),
                "find domain for address".to_string(),
                "wallet to domain".to_string(),
            ],
            description: "Get the primary .sol domain associated with a Solana wallet address (reverse lookup)".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetDomainForWalletAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            walletAddress: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        // Use Bonfida's reverse lookup API
        let url = format!(
            "https://sns-sdk-proxy.bonfida.workers.dev/favorite-domain/{}",
            parsed.walletAddress
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Failed to lookup domain: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        if let Some(result) = data.get("result").and_then(|r| r.as_str()) {
            Ok(json!({
                "status": "success",
                "domain": format!("{}.sol", result),
            }))
        } else {
            Ok(json!({
                "status": "success",
                "domain": null,
                "message": "No .sol domain associated with this wallet",
            }))
        }
    }
}

// =============================================================================
// GET_COINGECKO_TOP_GAINERS Action
// =============================================================================

#[derive(Debug)]
pub struct GetCoingeckoTopGainersAction {
    meta: ActionMetadata,
}

impl GetCoingeckoTopGainersAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "duration": {
                    "type": "string",
                    "enum": ["1h", "24h", "7d", "14d", "30d", "60d", "1y"],
                    "description": "Time duration for price change (default: 24h)",
                }
            },
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({ "duration": "24h" }),
            output: json!({
                "status": "success",
                "gainers": [
                    { "name": "Token1", "price_change_percentage": 150.5 }
                ],
            }),
            explanation: "Get top gaining tokens in the last 24 hours".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_COINGECKO_TOP_GAINERS".to_string(),
            similes: vec![
                "top gainers".to_string(),
                "biggest gainers".to_string(),
                "best performing tokens".to_string(),
                "movers".to_string(),
            ],
            description: "Get the top gaining tokens from CoinGecko over a specified time period".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetCoingeckoTopGainersAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            duration: Option<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let _duration = parsed.duration.unwrap_or_else(|| "24h".to_string());

        // Use CoinGecko's coins/markets endpoint sorted by price change
        let url = "https://api.coingecko.com/api/v3/coins/markets?vs_currency=usd&order=price_change_percentage_24h_desc&per_page=20&page=1&sparkline=false";

        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("CoinGecko API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "gainers": data,
        }))
    }
}

// =============================================================================
// CREATE_HELIUS_WEBHOOK Action
// =============================================================================

#[derive(Debug)]
pub struct CreateHeliusWebhookAction {
    meta: ActionMetadata,
}

impl CreateHeliusWebhookAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "accountAddresses": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of Solana addresses to monitor",
                },
                "webhookURL": {
                    "type": "string",
                    "description": "URL to receive webhook notifications",
                },
                "heliusApiKey": {
                    "type": "string",
                    "description": "Helius API key",
                }
            },
            "required": ["accountAddresses", "webhookURL", "heliusApiKey"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "accountAddresses": ["86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY"],
                "webhookURL": "https://my-server.com/webhook",
                "heliusApiKey": "your-api-key",
            }),
            output: json!({
                "status": "success",
                "webhookID": "webhook-id-123",
                "webhookURL": "https://my-server.com/webhook",
            }),
            explanation: "Create a Helius webhook to monitor an address".to_string(),
        }];

        let meta = ActionMetadata {
            name: "CREATE_HELIUS_WEBHOOK".to_string(),
            similes: vec![
                "create webhook".to_string(),
                "helius webhook".to_string(),
                "monitor address".to_string(),
                "transaction notifications".to_string(),
            ],
            description: "Create a Helius webhook to receive notifications for transactions on specified addresses".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for CreateHeliusWebhookAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            accountAddresses: Vec<String>,
            webhookURL: String,
            heliusApiKey: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let url = format!(
            "https://api.helius.xyz/v0/webhooks?api-key={}",
            parsed.heliusApiKey
        );

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&json!({
                "webhookURL": parsed.webhookURL,
                "transactionTypes": ["Any"],
                "accountAddresses": parsed.accountAddresses,
                "webhookType": "enhanced",
                "txnStatus": "all",
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Ok(json!({
                "status": "error",
                "message": format!("Failed to create webhook: {}", error_text),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "webhookID": data["webhookID"],
            "webhookURL": data["webhookURL"],
            "message": "Webhook created successfully",
        }))
    }
}

// =============================================================================
// GET_HELIUS_WEBHOOK Action
// =============================================================================

#[derive(Debug)]
pub struct GetHeliusWebhookAction {
    meta: ActionMetadata,
}

impl GetHeliusWebhookAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "webhookID": {
                    "type": "string",
                    "description": "The webhook ID to retrieve",
                },
                "heliusApiKey": {
                    "type": "string",
                    "description": "Helius API key",
                }
            },
            "required": ["webhookID", "heliusApiKey"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "webhookID": "webhook-id-123",
                "heliusApiKey": "your-api-key",
            }),
            output: json!({
                "status": "success",
                "webhook": {
                    "webhookURL": "https://my-server.com/webhook",
                    "accountAddresses": ["86xCnPeV..."],
                },
            }),
            explanation: "Get details of a Helius webhook".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_HELIUS_WEBHOOK".to_string(),
            similes: vec![
                "get webhook".to_string(),
                "fetch webhook".to_string(),
                "webhook details".to_string(),
            ],
            description: "Retrieve details of a Helius webhook by its ID".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetHeliusWebhookAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            webhookID: String,
            heliusApiKey: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let url = format!(
            "https://api.helius.xyz/v0/webhooks/{}?api-key={}",
            parsed.webhookID, parsed.heliusApiKey
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Failed to fetch webhook: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "webhook": {
                "webhookURL": data["webhookURL"],
                "accountAddresses": data["accountAddresses"],
                "transactionTypes": data["transactionTypes"],
                "webhookType": data["webhookType"],
            },
        }))
    }
}

// =============================================================================
// DELETE_HELIUS_WEBHOOK Action
// =============================================================================

#[derive(Debug)]
pub struct DeleteHeliusWebhookAction {
    meta: ActionMetadata,
}

impl DeleteHeliusWebhookAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "webhookID": {
                    "type": "string",
                    "description": "The webhook ID to delete",
                },
                "heliusApiKey": {
                    "type": "string",
                    "description": "Helius API key",
                }
            },
            "required": ["webhookID", "heliusApiKey"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "webhookID": "webhook-id-123",
                "heliusApiKey": "your-api-key",
            }),
            output: json!({
                "status": "success",
                "message": "Webhook deleted successfully",
            }),
            explanation: "Delete a Helius webhook".to_string(),
        }];

        let meta = ActionMetadata {
            name: "DELETE_HELIUS_WEBHOOK".to_string(),
            similes: vec![
                "delete webhook".to_string(),
                "remove webhook".to_string(),
                "stop monitoring".to_string(),
            ],
            description: "Delete a Helius webhook by its ID".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for DeleteHeliusWebhookAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            webhookID: String,
            heliusApiKey: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let url = format!(
            "https://api.helius.xyz/v0/webhooks/{}?api-key={}",
            parsed.webhookID, parsed.heliusApiKey
        );

        let client = reqwest::Client::new();
        let response = client.delete(&url).send().await?;

        if !response.status().is_success() && response.status().as_u16() != 204 {
            return Ok(json!({
                "status": "error",
                "message": format!("Failed to delete webhook: {}", response.status()),
            }));
        }

        Ok(json!({
            "status": "success",
            "message": "Webhook deleted successfully",
        }))
    }
}

// =============================================================================
// SEND_TRANSACTION_WITH_PRIORITY Action (Helius)
// =============================================================================

#[derive(Debug)]
pub struct SendTransactionWithPriorityAction {
    meta: ActionMetadata,
}

impl SendTransactionWithPriorityAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "transaction": {
                    "type": "string",
                    "description": "Base64 encoded signed transaction",
                },
                "priorityLevel": {
                    "type": "string",
                    "enum": ["low", "medium", "high", "veryHigh"],
                    "description": "Priority level for the transaction",
                },
                "heliusApiKey": {
                    "type": "string",
                    "description": "Helius API key",
                }
            },
            "required": ["transaction", "heliusApiKey"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "transaction": "base64-encoded-tx...",
                "priorityLevel": "high",
                "heliusApiKey": "your-api-key",
            }),
            output: json!({
                "status": "success",
                "signature": "tx-signature...",
            }),
            explanation: "Send a transaction with high priority via Helius".to_string(),
        }];

        let meta = ActionMetadata {
            name: "SEND_TRANSACTION_WITH_PRIORITY".to_string(),
            similes: vec![
                "priority send".to_string(),
                "fast transaction".to_string(),
                "helius send".to_string(),
                "priority fee".to_string(),
            ],
            description: "Send a Solana transaction with priority fees via Helius for faster confirmation".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for SendTransactionWithPriorityAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Input {
            transaction: String,
            priorityLevel: Option<String>,
            heliusApiKey: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let url = format!(
            "https://mainnet.helius-rpc.com/?api-key={}",
            parsed.heliusApiKey
        );

        let priority_level = parsed.priorityLevel.unwrap_or_else(|| "medium".to_string());

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&json!({
                "jsonrpc": "2.0",
                "id": "send-tx",
                "method": "sendTransaction",
                "params": [
                    parsed.transaction,
                    {
                        "encoding": "base64",
                        "skipPreflight": false,
                        "preflightCommitment": "confirmed",
                        "maxRetries": 3,
                    }
                ],
            }))
            .send()
            .await?;

        let data: Value = response.json().await?;

        if let Some(error) = data.get("error") {
            return Ok(json!({
                "status": "error",
                "message": format!("Transaction failed: {}", error),
            }));
        }

        Ok(json!({
            "status": "success",
            "signature": data["result"],
            "priorityLevel": priority_level,
        }))
    }
}

// =============================================================================
// GET_DEXSCREENER_TOKEN_PROFILES Action
// =============================================================================

#[derive(Debug)]
pub struct GetDexscreenerTokenProfilesAction {
    meta: ActionMetadata,
}

impl GetDexscreenerTokenProfilesAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({}),
            output: json!({
                "status": "success",
                "profiles": [
                    {
                        "chainId": "solana",
                        "tokenAddress": "...",
                        "description": "...",
                    }
                ],
            }),
            explanation: "Get the latest token profiles from Dexscreener".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_DEXSCREENER_TOKEN_PROFILES".to_string(),
            similes: vec![
                "dexscreener profiles".to_string(),
                "token profiles".to_string(),
                "dex token info".to_string(),
            ],
            description: "Get the latest token profiles from Dexscreener including metadata and social links".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetDexscreenerTokenProfilesAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, _input: Value) -> Result<Value> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.dexscreener.com/token-profiles/latest/v1")
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Dexscreener API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "profiles": data,
        }))
    }
}

// =============================================================================
// GET_DEXSCREENER_BOOSTED_TOKENS Action
// =============================================================================

#[derive(Debug)]
pub struct GetDexscreenerBoostedTokensAction {
    meta: ActionMetadata,
}

impl GetDexscreenerBoostedTokensAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "type": {
                    "type": "string",
                    "enum": ["latest", "top"],
                    "description": "Get 'latest' boosted tokens or 'top' most active boosts",
                }
            },
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({"type": "top"}),
            output: json!({
                "status": "success",
                "tokens": [
                    {
                        "chainId": "solana",
                        "tokenAddress": "...",
                        "amount": 100,
                        "totalAmount": 500,
                    }
                ],
            }),
            explanation: "Get the tokens with most active boosts on Dexscreener".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_DEXSCREENER_BOOSTED_TOKENS".to_string(),
            similes: vec![
                "boosted tokens".to_string(),
                "dexscreener boosts".to_string(),
                "trending dex tokens".to_string(),
            ],
            description: "Get boosted tokens from Dexscreener (latest or top active)".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetDexscreenerBoostedTokensAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            #[serde(rename = "type")]
            boost_type: Option<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let boost_type = parsed.boost_type.unwrap_or_else(|| "latest".to_string());

        let url = if boost_type == "top" {
            "https://api.dexscreener.com/token-boosts/top/v1"
        } else {
            "https://api.dexscreener.com/token-boosts/latest/v1"
        };

        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Dexscreener API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "tokens": data,
        }))
    }
}

// =============================================================================
// GET_DEXSCREENER_PAIRS_BY_TOKEN Action
// =============================================================================

#[derive(Debug)]
pub struct GetDexscreenerPairsByTokenAction {
    meta: ActionMetadata,
}

impl GetDexscreenerPairsByTokenAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "tokenAddresses": {
                    "type": "string",
                    "description": "One or more comma-separated token addresses (up to 30)",
                }
            },
            "required": ["tokenAddresses"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "tokenAddresses": "So11111111111111111111111111111111111111112"
            }),
            output: json!({
                "status": "success",
                "pairs": [
                    {
                        "chainId": "solana",
                        "pairAddress": "...",
                        "baseToken": { "symbol": "SOL" },
                        "quoteToken": { "symbol": "USDC" },
                        "priceUsd": "150.00",
                        "liquidity": { "usd": 1000000 },
                    }
                ],
            }),
            explanation: "Get trading pairs for SOL token".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_DEXSCREENER_PAIRS_BY_TOKEN".to_string(),
            similes: vec![
                "token pairs".to_string(),
                "dex pairs".to_string(),
                "trading pairs".to_string(),
                "token markets".to_string(),
            ],
            description: "Get all trading pairs for one or more token addresses from Dexscreener".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetDexscreenerPairsByTokenAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            tokenAddresses: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let url = format!(
            "https://api.dexscreener.com/tokens/v1/solana/{}",
            parsed.tokenAddresses
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Dexscreener API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "pairs": data,
        }))
    }
}

// =============================================================================
// SEARCH_DEXSCREENER_PAIRS Action
// =============================================================================

#[derive(Debug)]
pub struct SearchDexscreenerPairsAction {
    meta: ActionMetadata,
}

impl SearchDexscreenerPairsAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query (e.g. 'SOL/USDC', 'BONK', token address)",
                }
            },
            "required": ["query"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "query": "SOL/USDC"
            }),
            output: json!({
                "status": "success",
                "pairs": [
                    {
                        "chainId": "solana",
                        "pairAddress": "...",
                        "baseToken": { "symbol": "SOL" },
                        "quoteToken": { "symbol": "USDC" },
                        "priceUsd": "150.00",
                    }
                ],
            }),
            explanation: "Search for SOL/USDC trading pairs".to_string(),
        }];

        let meta = ActionMetadata {
            name: "SEARCH_DEXSCREENER_PAIRS".to_string(),
            similes: vec![
                "search pairs".to_string(),
                "find pairs".to_string(),
                "search dex".to_string(),
                "find markets".to_string(),
            ],
            description: "Search for trading pairs on Dexscreener by symbol, name, or address".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for SearchDexscreenerPairsAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            query: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let url = format!(
            "https://api.dexscreener.com/latest/dex/search?q={}",
            urlencoding::encode(&parsed.query)
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Dexscreener API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "pairs": data.get("pairs").cloned().unwrap_or(json!([])),
        }))
    }
}

// =============================================================================
// GET_DEXSCREENER_PAIR_BY_ADDRESS Action
// =============================================================================

#[derive(Debug)]
pub struct GetDexscreenerPairByAddressAction {
    meta: ActionMetadata,
}

impl GetDexscreenerPairByAddressAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "pairAddress": {
                    "type": "string",
                    "description": "The pair/pool address on Solana",
                }
            },
            "required": ["pairAddress"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "pairAddress": "7qbRF6YsyGuLUVs6Y1q64bdVrfe4ZcUUz1JRdoVNUJnm"
            }),
            output: json!({
                "status": "success",
                "pair": {
                    "chainId": "solana",
                    "pairAddress": "7qbRF6YsyGuLUVs6Y1q64bdVrfe4ZcUUz1JRdoVNUJnm",
                    "baseToken": { "symbol": "SOL" },
                    "quoteToken": { "symbol": "USDC" },
                    "priceUsd": "150.00",
                    "liquidity": { "usd": 1000000 },
                    "volume": { "h24": 5000000 },
                },
            }),
            explanation: "Get details for a specific trading pair".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_DEXSCREENER_PAIR_BY_ADDRESS".to_string(),
            similes: vec![
                "pair info".to_string(),
                "pool info".to_string(),
                "get pair".to_string(),
                "pair details".to_string(),
            ],
            description: "Get detailed info for a specific trading pair by its address".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetDexscreenerPairByAddressAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            pairAddress: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let url = format!(
            "https://api.dexscreener.com/latest/dex/pairs/solana/{}",
            parsed.pairAddress
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Dexscreener API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "pair": data.get("pair").cloned().unwrap_or(data.get("pairs").and_then(|p| p.get(0)).cloned().unwrap_or(json!(null))),
        }))
    }
}

// =============================================================================
// GET_ALL_DOMAINS_TLDS Action - Get all supported TLDs from AllDomains
// =============================================================================

#[derive(Debug)]
pub struct GetAllDomainsTldsAction {
    meta: ActionMetadata,
}

impl GetAllDomainsTldsAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({}),
            output: json!({
                "status": "success",
                "tlds": [".sol", ".abc", ".bonk"],
            }),
            explanation: "Get all supported TLDs".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_ALL_DOMAINS_TLDS".to_string(),
            similes: vec![
                "domain tlds".to_string(),
                "all tlds".to_string(),
                "supported domains".to_string(),
            ],
            description: "Get all supported top-level domains from AllDomains".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetAllDomainsTldsAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, _input: Value) -> Result<Value> {
        // AllDomains supported TLDs - these are the main ones on Solana
        let tlds = vec![
            ".sol",      // Bonfida SNS
            ".abc",      // AllDomains
            ".backpack", // Backpack
            ".bonk",     // Bonk domains
            ".poor",     // Poor domains
            ".glow",     // Glow
        ];

        Ok(json!({
            "status": "success",
            "tlds": tlds,
            "note": "Use RESOLVE_SOL_DOMAIN to resolve .sol domains",
        }))
    }
}

// =============================================================================
// GET_OWNED_DOMAINS Action - Get domains owned by a wallet
// =============================================================================

#[derive(Debug)]
pub struct GetOwnedDomainsAction {
    meta: ActionMetadata,
}

impl GetOwnedDomainsAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "owner": {
                    "type": "string",
                    "description": "Wallet address to get domains for",
                }
            },
            "required": ["owner"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({"owner": "YOUR_WALLET_ADDRESS"}),
            output: json!({
                "status": "success",
                "domains": ["mydomain.sol", "anotherdomain.sol"],
            }),
            explanation: "Get all .sol domains owned by a wallet".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_OWNED_DOMAINS".to_string(),
            similes: vec![
                "my domains".to_string(),
                "owned domains".to_string(),
                "wallet domains".to_string(),
            ],
            description: "Get all .sol domains owned by a wallet address".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetOwnedDomainsAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            owner: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        // Use Bonfida SNS API to get domains
        let client = reqwest::Client::new();
        let url = format!(
            "https://sns-sdk-proxy.bonfida.workers.dev/domains/{}",
            parsed.owner
        );

        let response = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("SNS API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "domains": data,
            "owner": parsed.owner,
        }))
    }
}

// =============================================================================
// GET_FAVORITE_DOMAIN Action - Get the favorite/primary domain for a wallet
// =============================================================================

#[derive(Debug)]
pub struct GetFavoriteDomainAction {
    meta: ActionMetadata,
}

impl GetFavoriteDomainAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "owner": {
                    "type": "string",
                    "description": "Wallet address to get favorite domain for",
                }
            },
            "required": ["owner"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({"owner": "YOUR_WALLET_ADDRESS"}),
            output: json!({
                "status": "success",
                "domain": "mydomain.sol",
            }),
            explanation: "Get the primary .sol domain for a wallet".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_FAVORITE_DOMAIN".to_string(),
            similes: vec![
                "primary domain".to_string(),
                "main domain".to_string(),
                "favorite domain".to_string(),
            ],
            description: "Get the favorite/primary .sol domain for a wallet address".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetFavoriteDomainAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            owner: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let client = reqwest::Client::new();
        let url = format!(
            "https://sns-sdk-proxy.bonfida.workers.dev/favorite-domain/{}",
            parsed.owner
        );

        let response = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("SNS API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "domain": data.get("result").cloned().unwrap_or(json!(null)),
            "owner": parsed.owner,
        }))
    }
}

// =============================================================================
// GET_BIRDEYE_TOKEN_OVERVIEW Action - Get token overview from Birdeye
// =============================================================================

#[derive(Debug)]
pub struct GetBirdeyeTokenOverviewAction {
    meta: ActionMetadata,
}

impl GetBirdeyeTokenOverviewAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "address": {
                    "type": "string",
                    "description": "Token mint address",
                },
                "apiKey": {
                    "type": "string",
                    "description": "Birdeye API key",
                }
            },
            "required": ["address", "apiKey"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "address": "So11111111111111111111111111111111111111112",
                "apiKey": "YOUR_API_KEY"
            }),
            output: json!({
                "status": "success",
                "data": {
                    "price": 150.0,
                    "priceChange24h": 5.2,
                    "volume24h": 1000000,
                },
            }),
            explanation: "Get SOL token overview from Birdeye".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_BIRDEYE_TOKEN_OVERVIEW".to_string(),
            similes: vec![
                "birdeye token".to_string(),
                "token overview".to_string(),
                "token analytics".to_string(),
            ],
            description: "Get comprehensive token overview from Birdeye including price, volume, and market data".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetBirdeyeTokenOverviewAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            address: String,
            apiKey: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let client = reqwest::Client::new();
        let url = format!(
            "https://public-api.birdeye.so/defi/token_overview?address={}",
            parsed.address
        );

        let response = client
            .get(&url)
            .header("X-API-KEY", &parsed.apiKey)
            .header("x-chain", "solana")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Birdeye API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "data": data.get("data").cloned().unwrap_or(data),
        }))
    }
}

// =============================================================================
// GET_BIRDEYE_TOKEN_SECURITY Action - Get token security info from Birdeye
// =============================================================================

#[derive(Debug)]
pub struct GetBirdeyeTokenSecurityAction {
    meta: ActionMetadata,
}

impl GetBirdeyeTokenSecurityAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "address": {
                    "type": "string",
                    "description": "Token mint address",
                },
                "apiKey": {
                    "type": "string",
                    "description": "Birdeye API key",
                }
            },
            "required": ["address", "apiKey"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "address": "TOKEN_ADDRESS",
                "apiKey": "YOUR_API_KEY"
            }),
            output: json!({
                "status": "success",
                "data": {
                    "isToken2022": false,
                    "transferFeeEnable": false,
                    "mutableMetadata": false,
                },
            }),
            explanation: "Get token security info from Birdeye".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_BIRDEYE_TOKEN_SECURITY".to_string(),
            similes: vec![
                "token security".to_string(),
                "token safety".to_string(),
                "token audit".to_string(),
            ],
            description: "Get token security information from Birdeye including freeze authority, mutable metadata, etc.".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetBirdeyeTokenSecurityAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            address: String,
            apiKey: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let client = reqwest::Client::new();
        let url = format!(
            "https://public-api.birdeye.so/defi/token_security?address={}",
            parsed.address
        );

        let response = client
            .get(&url)
            .header("X-API-KEY", &parsed.apiKey)
            .header("x-chain", "solana")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Birdeye API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "data": data.get("data").cloned().unwrap_or(data),
        }))
    }
}

// =============================================================================
// GET_BIRDEYE_TRENDING_TOKENS Action - Get trending tokens from Birdeye
// =============================================================================

#[derive(Debug)]
pub struct GetBirdeyeTrendingTokensAction {
    meta: ActionMetadata,
}

impl GetBirdeyeTrendingTokensAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "apiKey": {
                    "type": "string",
                    "description": "Birdeye API key",
                },
                "sortBy": {
                    "type": "string",
                    "enum": ["rank", "volume24hUSD", "liquidity"],
                    "description": "Sort by field (default: rank)",
                },
                "limit": {
                    "type": "integer",
                    "description": "Number of tokens to return (default: 20)",
                }
            },
            "required": ["apiKey"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "apiKey": "YOUR_API_KEY",
                "sortBy": "volume24hUSD",
                "limit": 10
            }),
            output: json!({
                "status": "success",
                "tokens": [],
            }),
            explanation: "Get top 10 tokens by 24h volume".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_BIRDEYE_TRENDING_TOKENS".to_string(),
            similes: vec![
                "trending tokens".to_string(),
                "hot tokens".to_string(),
                "top tokens".to_string(),
            ],
            description: "Get trending tokens from Birdeye sorted by volume, liquidity, or rank".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetBirdeyeTrendingTokensAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            apiKey: String,
            sortBy: Option<String>,
            limit: Option<u32>,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let sort_by = parsed.sortBy.unwrap_or_else(|| "rank".to_string());
        let limit = parsed.limit.unwrap_or(20);

        let client = reqwest::Client::new();
        let url = format!(
            "https://public-api.birdeye.so/defi/tokenlist?sort_by={}&sort_type=desc&offset=0&limit={}",
            sort_by, limit
        );

        let response = client
            .get(&url)
            .header("X-API-KEY", &parsed.apiKey)
            .header("x-chain", "solana")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Birdeye API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "tokens": data.get("data").and_then(|d| d.get("tokens")).cloned().unwrap_or(json!([])),
        }))
    }
}

// =============================================================================
// GET_BIRDEYE_OHLCV Action - Get OHLCV data from Birdeye
// =============================================================================

#[derive(Debug)]
pub struct GetBirdeyeOhlcvAction {
    meta: ActionMetadata,
}

impl GetBirdeyeOhlcvAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "address": {
                    "type": "string",
                    "description": "Token mint address",
                },
                "apiKey": {
                    "type": "string",
                    "description": "Birdeye API key",
                },
                "type": {
                    "type": "string",
                    "enum": ["1m", "3m", "5m", "15m", "30m", "1H", "2H", "4H", "6H", "8H", "12H", "1D", "3D", "1W", "1M"],
                    "description": "Time interval (default: 1H)",
                },
                "timeFrom": {
                    "type": "integer",
                    "description": "Start timestamp in seconds (optional)",
                },
                "timeTo": {
                    "type": "integer",
                    "description": "End timestamp in seconds (optional)",
                }
            },
            "required": ["address", "apiKey"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "address": "So11111111111111111111111111111111111111112",
                "apiKey": "YOUR_API_KEY",
                "type": "1H"
            }),
            output: json!({
                "status": "success",
                "data": [],
            }),
            explanation: "Get hourly OHLCV data for SOL".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_BIRDEYE_OHLCV".to_string(),
            similes: vec![
                "price history".to_string(),
                "ohlcv".to_string(),
                "candles".to_string(),
                "chart data".to_string(),
            ],
            description: "Get OHLCV (Open, High, Low, Close, Volume) price data from Birdeye".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetBirdeyeOhlcvAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            address: String,
            apiKey: String,
            #[serde(rename = "type")]
            interval_type: Option<String>,
            timeFrom: Option<i64>,
            timeTo: Option<i64>,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let interval = parsed.interval_type.unwrap_or_else(|| "1H".to_string());

        let mut url = format!(
            "https://public-api.birdeye.so/defi/ohlcv?address={}&type={}",
            parsed.address, interval
        );

        if let Some(from) = parsed.timeFrom {
            url.push_str(&format!("&time_from={}", from));
        }
        if let Some(to) = parsed.timeTo {
            url.push_str(&format!("&time_to={}", to));
        }

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("X-API-KEY", &parsed.apiKey)
            .header("x-chain", "solana")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Birdeye API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "data": data.get("data").and_then(|d| d.get("items")).cloned().unwrap_or(json!([])),
        }))
    }
}

// =============================================================================
// GET_BIRDEYE_TRADES Action - Get recent trades from Birdeye
// =============================================================================

#[derive(Debug)]
pub struct GetBirdeyeTradesAction {
    meta: ActionMetadata,
}

impl GetBirdeyeTradesAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "address": {
                    "type": "string",
                    "description": "Token mint address",
                },
                "apiKey": {
                    "type": "string",
                    "description": "Birdeye API key",
                },
                "limit": {
                    "type": "integer",
                    "description": "Number of trades to return (default: 50)",
                }
            },
            "required": ["address", "apiKey"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "address": "TOKEN_ADDRESS",
                "apiKey": "YOUR_API_KEY",
                "limit": 20
            }),
            output: json!({
                "status": "success",
                "trades": [],
            }),
            explanation: "Get last 20 trades for a token".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_BIRDEYE_TRADES".to_string(),
            similes: vec![
                "recent trades".to_string(),
                "trade history".to_string(),
                "token trades".to_string(),
            ],
            description: "Get recent trades for a token from Birdeye".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetBirdeyeTradesAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            address: String,
            apiKey: String,
            limit: Option<u32>,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let limit = parsed.limit.unwrap_or(50);

        let url = format!(
            "https://public-api.birdeye.so/defi/txs/token?address={}&limit={}",
            parsed.address, limit
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("X-API-KEY", &parsed.apiKey)
            .header("x-chain", "solana")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Birdeye API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "trades": data.get("data").and_then(|d| d.get("items")).cloned().unwrap_or(json!([])),
        }))
    }
}

// =============================================================================
// Register all Misc actions
// =============================================================================

pub fn register_misc_actions(registry: &mut ActionRegistry) {
    registry.register(GetCoingeckoTrendingTokensAction::new());
    registry.register(GetCoingeckoTokenInfoAction::new());
    registry.register(GetCoingeckoTokenPriceAction::new());
    registry.register(GetCoingeckoTopGainersAction::new());
    registry.register(ParseTransactionAction::new());
    registry.register(ResolveSolDomainAction::new());
    registry.register(GetDomainForWalletAction::new());
    registry.register(CreateHeliusWebhookAction::new());
    registry.register(GetHeliusWebhookAction::new());
    registry.register(DeleteHeliusWebhookAction::new());
    registry.register(SendTransactionWithPriorityAction::new());
    // Dexscreener actions
    registry.register(GetDexscreenerTokenProfilesAction::new());
    registry.register(GetDexscreenerBoostedTokensAction::new());
    registry.register(GetDexscreenerPairsByTokenAction::new());
    registry.register(SearchDexscreenerPairsAction::new());
    registry.register(GetDexscreenerPairByAddressAction::new());
    // Domain actions
    registry.register(GetAllDomainsTldsAction::new());
    registry.register(GetOwnedDomainsAction::new());
    registry.register(GetFavoriteDomainAction::new());
    // Birdeye actions
    registry.register(GetBirdeyeTokenOverviewAction::new());
    registry.register(GetBirdeyeTokenSecurityAction::new());
    registry.register(GetBirdeyeTrendingTokensAction::new());
    registry.register(GetBirdeyeOhlcvAction::new());
    registry.register(GetBirdeyeTradesAction::new());
}
