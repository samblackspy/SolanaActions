//! DeFi-related actions for Solana Agent Kit
//!
//! Includes: Sanctum LST, Solayer staking, Lulo lending, and more.

use async_trait::async_trait;
use anyhow::Result;
use serde::Deserialize;
use serde_json::{json, Value};
use solana_sdk::transaction::VersionedTransaction;

use crate::actions::{Action, ActionExample, ActionMetadata, ActionRegistry};
use crate::agent::Agent;

fn base64_decode(input: &str) -> Result<Vec<u8>> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    Ok(STANDARD.decode(input)?)
}

// =============================================================================
// GET_SANCTUM_PRICE Action
// =============================================================================

#[derive(Debug)]
pub struct GetSanctumPriceAction {
    meta: ActionMetadata,
}

impl GetSanctumPriceAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "mints": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Array of mint addresses or symbols of LSTs to get price for",
                }
            },
            "required": ["mints"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "mints": ["INF", "pwrsol", "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So"],
            }),
            output: json!({
                "status": "success",
                "prices": {
                    "INF": "1303329251",
                    "pwrsol": "1105899448",
                },
            }),
            explanation: "Fetch the prices of LSTs on Sanctum".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_SANCTUM_PRICE".to_string(),
            similes: vec![
                "get sanctum LST price".to_string(),
                "fetch sanctum LST price".to_string(),
                "sanctum price".to_string(),
                "lst price".to_string(),
            ],
            description: "Fetch the Price of LST (Liquid Staking Token) on Sanctum with specified mint addresses or symbols".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetSanctumPriceAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            mints: Vec<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;

        // Build query string for Sanctum API
        let query = parsed
            .mints
            .iter()
            .map(|m| format!("lst={}", m))
            .collect::<Vec<_>>()
            .join("&");

        let url = format!(
            "https://sanctum-extra-api.ngrok.dev/v1/sol-value/current?{}",
            query
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Failed to fetch Sanctum prices: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "message": "Price fetched successfully",
            "prices": data["solValues"],
        }))
    }
}

// =============================================================================
// STAKE_WITH_SOLAYER Action
// =============================================================================

#[derive(Debug)]
pub struct StakeWithSolayerAction {
    meta: ActionMetadata,
}

impl StakeWithSolayerAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "amount": {
                    "type": "number",
                    "description": "Amount of SOL to stake",
                }
            },
            "required": ["amount"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({ "amount": 1.0 }),
            output: json!({
                "status": "success",
                "signature": "3FgHn9...",
                "message": "Successfully staked 1.0 SOL for Solayer SOL (sSOL)",
            }),
            explanation: "Stake 1.0 SOL to receive Solayer SOL (sSOL)".to_string(),
        }];

        let meta = ActionMetadata {
            name: "STAKE_WITH_SOLAYER".to_string(),
            similes: vec![
                "stake sol with solayer".to_string(),
                "solayer staking".to_string(),
                "get ssol".to_string(),
                "solayer restaking".to_string(),
                "liquid staking solayer".to_string(),
            ],
            description: "Stake native SOL with Solayer's restaking protocol to receive Solayer SOL (sSOL)".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for StakeWithSolayerAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            amount: f64,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let url = format!(
            "https://app.solayer.org/api/action/restake/ssol?amount={}",
            parsed.amount
        );

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&json!({
                "account": agent.wallet.pubkey().to_string(),
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_data: Value = response.json().await.unwrap_or(json!({}));
            return Ok(json!({
                "status": "error",
                "message": error_data["message"].as_str().unwrap_or("Staking request failed"),
            }));
        }

        let data: Value = response.json().await?;
        let tx_b64 = data["transaction"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No transaction in response"))?;

        let tx_bytes = base64_decode(tx_b64)?;
        let mut transaction: VersionedTransaction = bincode::deserialize(&tx_bytes)?;

        // Update blockhash
        let latest_blockhash = agent.client.get_latest_blockhash()?;
        transaction.message.set_recent_blockhash(latest_blockhash);

        // Sign and send
        let signed_tx = agent.wallet.sign_transaction(transaction).await?;
        let signature = agent.client.send_and_confirm_transaction(&signed_tx)?;

        Ok(json!({
            "status": "success",
            "transaction": signature.to_string(),
            "message": format!("Successfully staked {} SOL for Solayer SOL (sSOL)", parsed.amount),
        }))
    }
}

// =============================================================================
// LULO_LEND Action (Placeholder - requires complex integration)
// =============================================================================

#[derive(Debug)]
pub struct LuloLendAction {
    meta: ActionMetadata,
}

impl LuloLendAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "mintAddress": {
                    "type": "string",
                    "description": "SPL token mint address to lend",
                },
                "amount": {
                    "type": "number",
                    "description": "Amount to lend",
                }
            },
            "required": ["mintAddress", "amount"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "mintAddress": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "amount": 100,
            }),
            output: json!({
                "status": "success",
                "signature": "4xKpN2...",
                "message": "Successfully lent 100 USDC",
            }),
            explanation: "Lend 100 USDC on Lulo".to_string(),
        }];

        let meta = ActionMetadata {
            name: "LULO_LEND".to_string(),
            similes: vec![
                "lend usdc with lulo".to_string(),
                "lend tokens with lulo".to_string(),
                "lulo lending".to_string(),
                "lend with lulo".to_string(),
            ],
            description: "Lend SPL tokens using Lulo protocol for yield".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for LuloLendAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Input {
            mintAddress: String,
            amount: f64,
        }

        let _parsed: Input = serde_json::from_value(input)?;

        // TODO: Implement Lulo lending integration
        // Requires Lulo API integration and transaction construction

        Ok(json!({
            "status": "error",
            "message": "Lulo lending is not yet fully implemented. Requires Lulo API integration.",
        }))
    }
}

// =============================================================================
// GET_SANCTUM_LST_APY Action
// =============================================================================

#[derive(Debug)]
pub struct GetSanctumLstApyAction {
    meta: ActionMetadata,
}

impl GetSanctumLstApyAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "mints": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Array of LST mint addresses or symbols",
                }
            },
            "required": ["mints"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "mints": ["INF", "jupSOL"],
            }),
            output: json!({
                "status": "success",
                "apys": {
                    "INF": 8.5,
                    "jupSOL": 7.2,
                },
            }),
            explanation: "Get APY for LSTs on Sanctum".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_SANCTUM_LST_APY".to_string(),
            similes: vec![
                "get sanctum apy".to_string(),
                "lst apy".to_string(),
                "sanctum yield".to_string(),
                "liquid staking apy".to_string(),
            ],
            description: "Get the APY (Annual Percentage Yield) for Liquid Staking Tokens on Sanctum".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetSanctumLstApyAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            mints: Vec<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let query = parsed
            .mints
            .iter()
            .map(|m| format!("lst={}", m))
            .collect::<Vec<_>>()
            .join("&");

        let url = format!(
            "https://sanctum-extra-api.ngrok.dev/v1/apy/latest?{}",
            query
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Failed to fetch Sanctum APY: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "message": "APY fetched successfully",
            "apys": data["apys"],
        }))
    }
}

// =============================================================================
// GET_DRIFT_MARKETS Action
// =============================================================================

#[derive(Debug)]
pub struct GetDriftMarketsAction {
    meta: ActionMetadata,
}

impl GetDriftMarketsAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "marketType": {
                    "type": "string",
                    "enum": ["spot", "perp"],
                    "description": "Type of market to get (spot or perp). If omitted, returns both.",
                }
            },
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({ "marketType": "perp" }),
            output: json!({
                "status": "success",
                "markets": ["SOL-PERP", "BTC-PERP", "ETH-PERP"],
            }),
            explanation: "Get available perpetual markets on Drift".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_DRIFT_MARKETS".to_string(),
            similes: vec![
                "get drift markets".to_string(),
                "drift markets".to_string(),
                "available drift markets".to_string(),
                "drift perp markets".to_string(),
            ],
            description: "Get a list of available Drift markets (spot and/or perpetual)".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetDriftMarketsAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            marketType: Option<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;

        // Drift mainnet spot markets
        let spot_markets = vec![
            "USDC", "SOL", "mSOL", "wBTC", "wETH", "USDT", "jitoSOL", "PYTH", "JTO", "WIF", "JUP", "RNDR", "W", "TNSR", "DRIFT", "INF", "BONK"
        ];

        // Drift mainnet perp markets
        let perp_markets = vec![
            "SOL-PERP", "BTC-PERP", "ETH-PERP", "APT-PERP", "1MBONK-PERP", "MATIC-PERP", "ARB-PERP",
            "DOGE-PERP", "BNB-PERP", "SUI-PERP", "1MPEPE-PERP", "OP-PERP", "RNDR-PERP", "XRP-PERP",
            "HNT-PERP", "INJ-PERP", "LINK-PERP", "RLB-PERP", "PYTH-PERP", "TIA-PERP", "JTO-PERP",
            "SEI-PERP", "WIF-PERP", "JUP-PERP", "DYM-PERP", "STRK-PERP", "W-PERP", "TNSR-PERP",
            "KMNO-PERP", "DRIFT-PERP"
        ];

        match parsed.marketType.as_deref() {
            Some("spot") => Ok(json!({
                "status": "success",
                "marketType": "spot",
                "markets": spot_markets,
            })),
            Some("perp") => Ok(json!({
                "status": "success",
                "marketType": "perp",
                "markets": perp_markets,
            })),
            _ => Ok(json!({
                "status": "success",
                "spot": spot_markets,
                "perp": perp_markets,
            })),
        }
    }
}

// =============================================================================
// GET_DEFI_RATES Action (aggregated lending rates)
// =============================================================================

#[derive(Debug)]
pub struct GetDefiRatesAction {
    meta: ActionMetadata,
}

impl GetDefiRatesAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "token": {
                    "type": "string",
                    "description": "Token symbol to get rates for (e.g., USDC, SOL)",
                }
            },
            "required": ["token"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({ "token": "USDC" }),
            output: json!({
                "status": "success",
                "token": "USDC",
                "rates": {
                    "marginfi": { "supply": 8.5, "borrow": 12.1 },
                    "kamino": { "supply": 7.8, "borrow": 11.5 },
                },
            }),
            explanation: "Get lending/borrowing rates for USDC across DeFi protocols".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_DEFI_RATES".to_string(),
            similes: vec![
                "get lending rates".to_string(),
                "defi rates".to_string(),
                "borrow rates".to_string(),
                "supply apy".to_string(),
            ],
            description: "Get aggregated lending and borrowing rates for a token across Solana DeFi protocols".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetDefiRatesAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            token: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        // Use DeFiLlama or similar API to get rates
        // For now, return a placeholder with common protocols
        // In production, this would query each protocol's API

        Ok(json!({
            "status": "success",
            "token": parsed.token,
            "message": "DeFi rates aggregation requires protocol-specific API integration",
            "protocols": ["marginfi", "kamino", "solend", "drift", "mango"],
            "note": "Query each protocol directly for real-time rates",
        }))
    }
}

// =============================================================================
// SWAP_ON_RAYDIUM Action (placeholder)
// =============================================================================

#[derive(Debug)]
pub struct SwapOnRaydiumAction {
    meta: ActionMetadata,
}

impl SwapOnRaydiumAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "inputMint": {
                    "type": "string",
                    "description": "Input token mint address",
                },
                "outputMint": {
                    "type": "string",
                    "description": "Output token mint address",
                },
                "amount": {
                    "type": "number",
                    "description": "Amount to swap",
                },
                "slippage": {
                    "type": "number",
                    "description": "Slippage tolerance in percentage",
                }
            },
            "required": ["inputMint", "outputMint", "amount"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "inputMint": "So11111111111111111111111111111111111111112",
                "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "amount": 1.0,
                "slippage": 1.0,
            }),
            output: json!({
                "status": "success",
                "signature": "abc123...",
            }),
            explanation: "Swap 1 SOL for USDC on Raydium".to_string(),
        }];

        let meta = ActionMetadata {
            name: "SWAP_ON_RAYDIUM".to_string(),
            similes: vec![
                "raydium swap".to_string(),
                "swap on raydium".to_string(),
                "raydium trade".to_string(),
                "raydium exchange".to_string(),
            ],
            description: "Swap tokens using Raydium AMM".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for SwapOnRaydiumAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Input {
            inputMint: String,
            outputMint: String,
            amount: f64,
            slippage: Option<f64>,
        }

        let _parsed: Input = serde_json::from_value(input)?;

        // Note: Direct Raydium swaps require complex SDK integration
        // For most use cases, Jupiter (TRADE action) routes through Raydium automatically

        Ok(json!({
            "status": "info",
            "message": "For swaps, use the TRADE action which routes through Jupiter and includes Raydium pools automatically for best pricing.",
            "recommendation": "Use TRADE action instead",
        }))
    }
}

// =============================================================================
// GET_ORCA_WHIRLPOOLS Action - Fetch Orca whirlpool data
// =============================================================================

#[derive(Debug)]
pub struct GetOrcaWhirlpoolsAction {
    meta: ActionMetadata,
}

impl GetOrcaWhirlpoolsAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "tokenMint": {
                    "type": "string",
                    "description": "Filter pools by token mint address (optional)",
                }
            },
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({}),
            output: json!({
                "status": "success",
                "pools": [
                    {
                        "address": "...",
                        "tokenA": { "symbol": "SOL" },
                        "tokenB": { "symbol": "USDC" },
                        "tvl": 1000000,
                        "volume24h": 500000,
                    }
                ],
            }),
            explanation: "Get Orca whirlpool data".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_ORCA_WHIRLPOOLS".to_string(),
            similes: vec![
                "orca pools".to_string(),
                "whirlpools".to_string(),
                "orca liquidity".to_string(),
            ],
            description: "Get Orca whirlpool liquidity pools data including TVL and volume".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetOrcaWhirlpoolsAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            tokenMint: Option<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let client = reqwest::Client::new();
        let response = client
            .get("https://api.mainnet.orca.so/v1/whirlpool/list")
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Orca API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        // Filter by token mint if provided
        let pools = if let Some(ref token_mint) = parsed.tokenMint {
            if let Some(whirlpools) = data.get("whirlpools").and_then(|w| w.as_array()) {
                let filtered: Vec<&Value> = whirlpools.iter().filter(|p| {
                    let token_a = p.get("tokenA").and_then(|t| t.get("mint")).and_then(|m| m.as_str()).unwrap_or("");
                    let token_b = p.get("tokenB").and_then(|t| t.get("mint")).and_then(|m| m.as_str()).unwrap_or("");
                    token_a == token_mint || token_b == token_mint
                }).take(50).collect();
                json!(filtered)
            } else {
                data.get("whirlpools").cloned().unwrap_or(json!([]))
            }
        } else {
            data.get("whirlpools").cloned().unwrap_or(json!([]))
        };

        Ok(json!({
            "status": "success",
            "pools": pools,
        }))
    }
}

// =============================================================================
// GET_RAYDIUM_POOLS Action - Fetch Raydium pool data
// =============================================================================

#[derive(Debug)]
pub struct GetRaydiumPoolsAction {
    meta: ActionMetadata,
}

impl GetRaydiumPoolsAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "type": {
                    "type": "string",
                    "enum": ["all", "concentrated", "standard"],
                    "description": "Pool type to fetch (default: all)",
                },
                "poolIds": {
                    "type": "string",
                    "description": "Comma-separated pool IDs to fetch specific pools",
                }
            },
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({"type": "concentrated"}),
            output: json!({
                "status": "success",
                "pools": [
                    {
                        "id": "...",
                        "mintA": "...",
                        "mintB": "...",
                        "tvl": 1000000,
                    }
                ],
            }),
            explanation: "Get Raydium concentrated liquidity pools".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_RAYDIUM_POOLS".to_string(),
            similes: vec![
                "raydium pools".to_string(),
                "raydium liquidity".to_string(),
                "raydium amm".to_string(),
            ],
            description: "Get Raydium AMM pool data including TVL and liquidity info".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetRaydiumPoolsAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            #[serde(rename = "type")]
            pool_type: Option<String>,
            poolIds: Option<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let url = if let Some(ref ids) = parsed.poolIds {
            format!("https://api-v3.raydium.io/pools/info/ids?ids={}", ids)
        } else {
            match parsed.pool_type.as_deref() {
                Some("concentrated") => "https://api-v3.raydium.io/pools/info/list?poolType=concentrated&poolSortField=default&sortType=desc&pageSize=100&page=1".to_string(),
                Some("standard") => "https://api-v3.raydium.io/pools/info/list?poolType=standard&poolSortField=default&sortType=desc&pageSize=100&page=1".to_string(),
                _ => "https://api-v3.raydium.io/pools/info/list?poolType=all&poolSortField=default&sortType=desc&pageSize=100&page=1".to_string(),
            }
        };

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Raydium API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "pools": data.get("data").cloned().unwrap_or(data),
        }))
    }
}

// =============================================================================
// GET_METEORA_POOLS Action - Fetch Meteora DLMM pool data
// =============================================================================

#[derive(Debug)]
pub struct GetMeteoraPoolsAction {
    meta: ActionMetadata,
}

impl GetMeteoraPoolsAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "tokenMint": {
                    "type": "string",
                    "description": "Filter pools by token mint address (optional)",
                }
            },
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({}),
            output: json!({
                "status": "success",
                "pools": [
                    {
                        "address": "...",
                        "name": "SOL-USDC",
                        "mint_x": "...",
                        "mint_y": "...",
                        "liquidity": 1000000,
                    }
                ],
            }),
            explanation: "Get Meteora DLMM pool data".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_METEORA_POOLS".to_string(),
            similes: vec![
                "meteora pools".to_string(),
                "dlmm pools".to_string(),
                "meteora liquidity".to_string(),
            ],
            description: "Get Meteora DLMM pool data including liquidity and trading info".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetMeteoraPoolsAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            tokenMint: Option<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let client = reqwest::Client::new();
        let response = client
            .get("https://dlmm-api.meteora.ag/pair/all")
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Meteora API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        // Filter by token mint if provided
        let pools = if let Some(ref token_mint) = parsed.tokenMint {
            if let Some(pairs) = data.as_array() {
                let filtered: Vec<&Value> = pairs.iter().filter(|p| {
                    let mint_x = p.get("mint_x").and_then(|m| m.as_str()).unwrap_or("");
                    let mint_y = p.get("mint_y").and_then(|m| m.as_str()).unwrap_or("");
                    mint_x == token_mint || mint_y == token_mint
                }).take(50).collect();
                json!(filtered)
            } else {
                data
            }
        } else {
            data
        };

        Ok(json!({
            "status": "success",
            "pools": pools,
        }))
    }
}

// =============================================================================
// GET_JUPITER_ROUTE_MAP Action - Get Jupiter supported tokens and routes
// =============================================================================

#[derive(Debug)]
pub struct GetJupiterRouteMapAction {
    meta: ActionMetadata,
}

impl GetJupiterRouteMapAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "onlyDirectRoutes": {
                    "type": "boolean",
                    "description": "Only show direct routes (default: false)",
                }
            },
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({}),
            output: json!({
                "status": "success",
                "indexedRouteMap": {},
                "mintKeys": [],
            }),
            explanation: "Get Jupiter route map for all tradeable tokens".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_JUPITER_ROUTE_MAP".to_string(),
            similes: vec![
                "jupiter routes".to_string(),
                "swap routes".to_string(),
                "trading routes".to_string(),
            ],
            description: "Get Jupiter's indexed route map showing all available swap routes".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetJupiterRouteMapAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, _input: Value) -> Result<Value> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://quote-api.jup.ag/v6/indexed-route-map")
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Jupiter API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "indexedRouteMap": data.get("indexedRouteMap").cloned().unwrap_or(json!({})),
            "mintKeys": data.get("mintKeys").cloned().unwrap_or(json!([])),
        }))
    }
}

// =============================================================================
// Register all DeFi actions
// =============================================================================

pub fn register_defi_actions(registry: &mut ActionRegistry) {
    registry.register(GetSanctumPriceAction::new());
    registry.register(StakeWithSolayerAction::new());
    registry.register(LuloLendAction::new());
    registry.register(GetSanctumLstApyAction::new());
    registry.register(GetDriftMarketsAction::new());
    registry.register(GetDefiRatesAction::new());
    registry.register(SwapOnRaydiumAction::new());
    // Pool data fetching
    registry.register(GetOrcaWhirlpoolsAction::new());
    registry.register(GetRaydiumPoolsAction::new());
    registry.register(GetMeteoraPoolsAction::new());
    registry.register(GetJupiterRouteMapAction::new());
}
