use std::str::FromStr;

use async_trait::async_trait;
use anyhow::Result;
use serde::Deserialize;
use serde_json::{json, Value};
use solana_sdk::pubkey::Pubkey;

use crate::actions::{Action, ActionExample, ActionMetadata, ActionRegistry};
use crate::agent::Agent;
use solana_actions_token::TokenActions;

// =============================================================================
// BALANCE_ACTION - Get SOL or SPL token balance
// =============================================================================

#[derive(Debug)]
pub struct GetBalanceAction {
    meta: ActionMetadata,
}

impl GetBalanceAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "tokenAddress": {
                    "type": "string",
                    "description": "Optional SPL token mint address; if omitted, SOL balance is returned",
                }
            },
            "required": [],
            "additionalProperties": false,
        });

        let examples = vec![
            ActionExample {
                input: json!({}),
                output: json!({
                    "status": "success",
                    "balance": "100",
                    "token": "SOL",
                }),
                explanation: "Get SOL balance of the wallet".to_string(),
            },
            ActionExample {
                input: json!({
                    "tokenAddress": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                }),
                output: json!({
                    "status": "success",
                    "balance": "1000",
                    "token": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                }),
                explanation: "Get USDC token balance".to_string(),
            },
        ];

        let meta = ActionMetadata {
            name: "BALANCE_ACTION".to_string(),
            similes: vec![
                "check balance".to_string(),
                "get wallet balance".to_string(),
                "view balance".to_string(),
                "show balance".to_string(),
                "check token balance".to_string(),
            ],
            description: "Get the balance of a Solana wallet or token account. If you want to get the balance of your wallet, you don't need to provide the tokenAddress. If no tokenAddress is provided, the balance will be in SOL.".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetBalanceAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            #[serde(default)]
            tokenAddress: Option<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let token_address = parsed.tokenAddress.clone();
        let mint_pubkey = if let Some(addr) = token_address.as_deref() {
            Some(Pubkey::from_str(addr)?)
        } else {
            None
        };

        let balance = agent.get_balance(mint_pubkey).await?;
        let token = token_address.unwrap_or_else(|| "SOL".to_string());

        Ok(json!({
            "status": "success",
            "balance": balance,
            "token": token,
        }))
    }
}

// =============================================================================
// TOKEN_BALANCE_ACTION - Get all token balances for a wallet
// =============================================================================

#[derive(Debug)]
pub struct TokenBalancesAction {
    meta: ActionMetadata,
}

impl TokenBalancesAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "walletAddress": {
                    "type": "string",
                    "description": "Optional wallet address to check; defaults to agent wallet",
                }
            },
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({}),
            output: json!({
                "status": "success",
                "balance": {
                    "sol": 5.5,
                    "tokens": [
                        {
                            "tokenAddress": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                            "symbol": "USDC",
                            "name": "USD Coin",
                            "balance": 100.0,
                            "decimals": 6,
                        }
                    ]
                }
            }),
            explanation: "Get all token balances for the agent's wallet".to_string(),
        }];

        let meta = ActionMetadata {
            name: "TOKEN_BALANCE_ACTION".to_string(),
            similes: vec![
                "check token balances".to_string(),
                "get wallet token balances".to_string(),
                "view token balances".to_string(),
                "show token balances".to_string(),
                "all balances".to_string(),
            ],
            description: "Get all token balances (SOL + SPL tokens) for a Solana wallet".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for TokenBalancesAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            walletAddress: Option<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let wallet_pubkey = if let Some(addr) = parsed.walletAddress {
            Pubkey::from_str(&addr)?
        } else {
            agent.wallet.pubkey()
        };

        let lamports = agent.client.get_balance(&wallet_pubkey)?;
        let sol_balance = lamports as f64 / 1_000_000_000.0;

        let rpc_url = agent.client.url();
        let client = reqwest::Client::new();

        let request = json!({
            "jsonrpc": "2.0",
            "id": "token-balances",
            "method": "getTokenAccountsByOwner",
            "params": [
                wallet_pubkey.to_string(),
                { "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" },
                { "encoding": "jsonParsed" }
            ]
        });

        let response = client
            .post(rpc_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let data: Value = response.json().await?;

        let mut tokens = Vec::new();
        if let Some(accounts) = data["result"]["value"].as_array() {
            for account in accounts {
                if let Some(info) = account["account"]["data"]["parsed"]["info"].as_object() {
                    let mint = info.get("mint").and_then(|m| m.as_str()).unwrap_or("");
                    let token_amount = info.get("tokenAmount");
                    
                    if let Some(amount_obj) = token_amount {
                        let ui_amount = amount_obj["uiAmount"].as_f64().unwrap_or(0.0);
                        let decimals = amount_obj["decimals"].as_u64().unwrap_or(0) as u8;
                        
                        if ui_amount > 0.0 {
                            tokens.push(json!({
                                "tokenAddress": mint,
                                "balance": ui_amount,
                                "decimals": decimals,
                                "account": account["pubkey"],
                            }));
                        }
                    }
                }
            }
        }

        Ok(json!({
            "status": "success",
            "balance": {
                "sol": sol_balance,
                "tokens": tokens,
            }
        }))
    }
}

// =============================================================================
// TRANSFER - Transfer SOL or SPL tokens
// =============================================================================

#[derive(Debug)]
pub struct TransferAction {
    meta: ActionMetadata,
}

impl TransferAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "to": {
                    "type": "string",
                    "description": "Destination Solana address",
                },
                "amount": {
                    "type": "number",
                    "description": "Amount of SOL or tokens to transfer",
                },
                "mint": {
                    "type": ["string", "null"],
                    "description": "SPL token mint address; null or omitted for native SOL",
                },
            },
            "required": ["to", "amount"],
            "additionalProperties": false,
        });

        let examples = vec![
            ActionExample {
                input: json!({
                    "to": "ExampleDestination1111111111111111111111111111",
                    "amount": 0.1,
                }),
                output: json!({ "signature": "example_signature" }),
                explanation: "Transfer 0.1 SOL to the given address".to_string(),
            },
            ActionExample {
                input: json!({
                    "to": "ExampleDestination1111111111111111111111111111",
                    "amount": 5.0,
                    "mint": "So11111111111111111111111111111111111111112",
                }),
                output: json!({ "signature": "example_token_signature" }),
                explanation: "Transfer 5 units of the given SPL token".to_string(),
            },
        ];

        let meta = ActionMetadata {
            name: "TRANSFER".to_string(),
            similes: vec![
                "send sol".to_string(),
                "send tokens".to_string(),
                "transfer to another wallet".to_string(),
            ],
            description: "Transfer SOL or SPL tokens from the agent's wallet to another address".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for TransferAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            to: String,
            amount: f64,
            #[serde(default)]
            mint: Option<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let to_pubkey = Pubkey::from_str(&parsed.to)?;
        let mint_pubkey = if let Some(mint_str) = parsed.mint {
            Some(Pubkey::from_str(&mint_str)?)
        } else {
            None
        };

        let signature = agent.transfer(to_pubkey, parsed.amount, mint_pubkey).await?;
        Ok(json!({ "signature": signature }))
    }
}

// =============================================================================
// WALLET_ADDRESS - Get wallet address
// =============================================================================

#[derive(Debug)]
pub struct WalletAddressAction {
    meta: ActionMetadata,
}

impl WalletAddressAction {
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
                "message": "Wallet address retrieved successfully",
                "address": "8x2dR8Mpzuz2YqyZyZjUbYWKSWesBo5jMx2Q9Y86udVk",
            }),
            explanation: "Get your wallet address".to_string(),
        }];

        let meta = ActionMetadata {
            name: "WALLET_ADDRESS".to_string(),
            similes: vec![
                "get wallet address".to_string(),
                "show wallet address".to_string(),
                "display wallet address".to_string(),
                "my wallet address".to_string(),
            ],
            description: "Get your wallet address.".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for WalletAddressAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, agent: &Agent, _input: Value) -> Result<Value> {
        let address = agent.wallet.pubkey().to_string();
        Ok(json!({
            "status": "success",
            "message": "Wallet address retrieved successfully",
            "address": address,
        }))
    }
}

// =============================================================================
// GET_TPS - Get network TPS
// =============================================================================

#[derive(Debug)]
pub struct GetTpsAction {
    meta: ActionMetadata,
}

impl GetTpsAction {
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
                "tps": 3500,
                "message": "Current network TPS: 3500",
            }),
            explanation: "Get the current TPS of the Solana network".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_TPS".to_string(),
            similes: vec![
                "get transactions per second".to_string(),
                "check network speed".to_string(),
                "network performance".to_string(),
                "transaction throughput".to_string(),
                "network tps".to_string(),
            ],
            description: "Get the current transactions per second (TPS) of the Solana network".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetTpsAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, agent: &Agent, _input: Value) -> Result<Value> {
        let perf_samples = agent.client.get_recent_performance_samples(Some(1))?;

        if perf_samples.is_empty() {
            return Ok(json!({
                "status": "error",
                "message": "No performance samples available",
            }));
        }

        let sample = &perf_samples[0];
        let tps = if sample.sample_period_secs > 0 {
            sample.num_transactions as f64 / sample.sample_period_secs as f64
        } else {
            0.0
        };

        Ok(json!({
            "status": "success",
            "tps": tps,
            "message": format!("Current network TPS: {:.0}", tps),
        }))
    }
}

// =============================================================================
// REQUEST_FUNDS - Request faucet funds (devnet/testnet)
// =============================================================================

#[derive(Debug)]
pub struct RequestFundsAction {
    meta: ActionMetadata,
}

impl RequestFundsAction {
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
                "message": "Successfully requested faucet funds",
                "signature": "5abc123...",
            }),
            explanation: "Request SOL from the devnet faucet".to_string(),
        }];

        let meta = ActionMetadata {
            name: "REQUEST_FUNDS".to_string(),
            similes: vec![
                "request sol".to_string(),
                "get test sol".to_string(),
                "use faucet".to_string(),
                "request test tokens".to_string(),
                "get devnet sol".to_string(),
            ],
            description: "Request SOL from Solana faucet (devnet/testnet only)".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for RequestFundsAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, agent: &Agent, _input: Value) -> Result<Value> {
        use solana_sdk::native_token::LAMPORTS_PER_SOL;

        let pubkey = agent.wallet.pubkey();
        let signature = agent
            .client
            .request_airdrop(&pubkey, 5 * LAMPORTS_PER_SOL)?;

        let latest_blockhash = agent.client.get_latest_blockhash()?;
        agent.client.confirm_transaction_with_spinner(
            &signature,
            &latest_blockhash,
            solana_sdk::commitment_config::CommitmentConfig::confirmed(),
        )?;

        Ok(json!({
            "status": "success",
            "message": "Successfully requested faucet funds",
            "signature": signature.to_string(),
        }))
    }
}

// =============================================================================
// FETCH_PRICE Action - Get token price from Jupiter
// =============================================================================

#[derive(Debug)]
pub struct FetchPriceAction {
    meta: ActionMetadata,
}

impl FetchPriceAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "tokenAddress": {
                    "type": "string",
                    "description": "The mint address of the token to fetch the price for",
                }
            },
            "required": ["tokenAddress"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "tokenAddress": "So11111111111111111111111111111111111111112",
            }),
            output: json!({
                "status": "success",
                "price": "23.45",
                "message": "Current price: $23.45 USDC",
            }),
            explanation: "Get the current price of SOL token in USDC".to_string(),
        }];

        let meta = ActionMetadata {
            name: "FETCH_PRICE".to_string(),
            similes: vec![
                "get token price".to_string(),
                "check price".to_string(),
                "token value".to_string(),
                "price check".to_string(),
                "get price in usd".to_string(),
            ],
            description: "Fetch the current price of a Solana token in USDC using Jupiter API".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for FetchPriceAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            tokenAddress: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let url = format!(
            "https://api.jup.ag/price/v2?ids={}",
            parsed.tokenAddress
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Failed to fetch price: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;
        let price = data["data"][&parsed.tokenAddress]["price"]
            .as_str()
            .or_else(|| data["data"][&parsed.tokenAddress]["price"].as_f64().map(|_| ""))
            .unwrap_or("");

        let price_str = if price.is_empty() {
            data["data"][&parsed.tokenAddress]["price"]
                .as_f64()
                .map(|p| p.to_string())
                .unwrap_or_else(|| "N/A".to_string())
        } else {
            price.to_string()
        };

        if price_str == "N/A" {
            return Ok(json!({
                "status": "error",
                "message": "Price data not available for the given token",
            }));
        }

        Ok(json!({
            "status": "success",
            "price": price_str,
            "message": format!("Current price: ${} USDC", price_str),
        }))
    }
}

// =============================================================================
// TRADE Action - Swap tokens via Jupiter
// =============================================================================

#[derive(Debug)]
pub struct TradeAction {
    meta: ActionMetadata,
}

impl TradeAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "outputMint": {
                    "type": "string",
                    "description": "Target token mint address to swap to",
                },
                "inputAmount": {
                    "type": "number",
                    "description": "Amount to swap (in token units, not lamports)",
                },
                "inputMint": {
                    "type": "string",
                    "description": "Source token mint address (defaults to SOL if omitted)",
                },
                "slippageBps": {
                    "type": "integer",
                    "description": "Slippage tolerance in basis points (e.g., 100 = 1%)",
                },
            },
            "required": ["outputMint", "inputAmount"],
            "additionalProperties": false,
        });

        let examples = vec![
            ActionExample {
                input: json!({
                    "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                    "inputAmount": 1,
                }),
                output: json!({
                    "status": "success",
                    "message": "Trade executed successfully",
                    "transaction": "5UfgJ5vV...",
                }),
                explanation: "Swap 1 SOL for USDC".to_string(),
            },
        ];

        let meta = ActionMetadata {
            name: "TRADE".to_string(),
            similes: vec![
                "swap tokens".to_string(),
                "exchange tokens".to_string(),
                "trade tokens".to_string(),
                "convert tokens".to_string(),
                "swap sol".to_string(),
            ],
            description: "Swap tokens using Jupiter Exchange. Defaults to SOL as input if inputMint is not specified.".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for TradeAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, agent: &Agent, input: Value) -> Result<Value> {
        use solana_sdk::transaction::VersionedTransaction;

        #[derive(Deserialize)]
        struct Input {
            outputMint: String,
            inputAmount: f64,
            inputMint: Option<String>,
            slippageBps: Option<u32>,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let input_mint = parsed
            .inputMint
            .clone()
            .unwrap_or_else(|| "So11111111111111111111111111111111111111112".to_string());

        let decimals = if input_mint == "So11111111111111111111111111111111111111112" {
            9
        } else {
            6
        };

        let scaled_amount = (parsed.inputAmount * 10f64.powi(decimals)) as u64;

        let quote_url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&dynamicSlippage=true",
            input_mint, parsed.outputMint, scaled_amount
        );

        let client = reqwest::Client::new();
        let quote_response: Value = client.get(&quote_url).send().await?.json().await?;

        let swap_request = json!({
            "quoteResponse": quote_response,
            "userPublicKey": agent.wallet.pubkey().to_string(),
            "wrapAndUnwrapSol": true,
            "dynamicComputeUnitLimit": true,
            "dynamicSlippage": true,
        });

        let swap_response: Value = client
            .post("https://quote-api.jup.ag/v6/swap")
            .header("Content-Type", "application/json")
            .json(&swap_request)
            .send()
            .await?
            .json()
            .await?;

        let swap_tx_b64 = swap_response["swapTransaction"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No swapTransaction in response"))?;

        let tx_bytes = base64_decode(swap_tx_b64)?;
        let mut transaction: VersionedTransaction = bincode::deserialize(&tx_bytes)?;

        let latest_blockhash = agent.client.get_latest_blockhash()?;
        transaction.message.set_recent_blockhash(latest_blockhash);

        let signed_tx = agent.wallet.sign_transaction(transaction).await?;
        let signature = agent.client.send_and_confirm_transaction(&signed_tx)?;

        Ok(json!({
            "status": "success",
            "message": "Trade executed successfully",
            "transaction": signature.to_string(),
        }))
    }
}

fn base64_decode(input: &str) -> Result<Vec<u8>> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    Ok(STANDARD.decode(input)?)
}

// =============================================================================
// GET_JUPITER_TOKEN_LIST Action
// =============================================================================

#[derive(Debug)]
pub struct GetJupiterTokenListAction {
    meta: ActionMetadata,
}

impl GetJupiterTokenListAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "tags": {
                    "type": "string",
                    "description": "Filter by tags (e.g. 'verified', 'strict')",
                }
            },
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({"tags": "strict"}),
            output: json!({
                "status": "success",
                "tokens": [],
            }),
            explanation: "Get strict token list from Jupiter".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_JUPITER_TOKEN_LIST".to_string(),
            similes: vec![
                "jupiter tokens".to_string(),
                "token list".to_string(),
                "all tokens".to_string(),
            ],
            description: "Get the full token list from Jupiter with optional tag filtering".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetJupiterTokenListAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            tags: Option<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let url = match parsed.tags.as_deref() {
            Some("strict") => "https://token.jup.ag/strict",
            _ => "https://token.jup.ag/all",
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
                "message": format!("Jupiter API error: {}", response.status()),
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
// SEARCH_JUPITER_TOKENS Action
// =============================================================================

#[derive(Debug)]
pub struct SearchJupiterTokensAction {
    meta: ActionMetadata,
}

impl SearchJupiterTokensAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query (symbol, name, or address)",
                }
            },
            "required": ["query"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({"query": "BONK"}),
            output: json!({
                "status": "success",
                "tokens": [],
                "count": 0,
            }),
            explanation: "Search for BONK token".to_string(),
        }];

        let meta = ActionMetadata {
            name: "SEARCH_JUPITER_TOKENS".to_string(),
            similes: vec![
                "find token".to_string(),
                "search token".to_string(),
                "lookup token".to_string(),
            ],
            description: "Search Jupiter token list by symbol, name, or address".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for SearchJupiterTokensAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            query: String,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let query_lower = parsed.query.to_lowercase();

        let client = reqwest::Client::new();
        let response = client
            .get("https://token.jup.ag/all")
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Jupiter API error: {}", response.status()),
            }));
        }

        let tokens: Vec<Value> = response.json().await?;

        let matching: Vec<&Value> = tokens.iter().filter(|t| {
            let symbol = t.get("symbol").and_then(|s| s.as_str()).unwrap_or("").to_lowercase();
            let name = t.get("name").and_then(|s| s.as_str()).unwrap_or("").to_lowercase();
            let address = t.get("address").and_then(|s| s.as_str()).unwrap_or("");
            
            symbol.contains(&query_lower) || name.contains(&query_lower) || address == parsed.query
        }).take(20).collect();

        Ok(json!({
            "status": "success",
            "tokens": matching,
            "count": matching.len(),
        }))
    }
}

// =============================================================================
// RUGCHECK Action
// =============================================================================

#[derive(Debug)]
pub struct RugcheckAction {
    meta: ActionMetadata,
}

impl RugcheckAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "mint": {
                    "type": "string",
                    "description": "The token mint address to check",
                }
            },
            "required": ["mint"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "mint": "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",
            }),
            output: json!({
                "status": "success",
                "response": {},
            }),
            explanation: "Check whether JUP is a rugpull".to_string(),
        }];

        let meta = ActionMetadata {
            name: "RUGCHECK".to_string(),
            similes: vec![
                "check rug pull".to_string(),
                "rug pull check".to_string(),
                "rug pull detector".to_string(),
                "token safety".to_string(),
            ],
            description: "Check if a token is a rug pull using rugcheck.xyz API".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for RugcheckAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            mint: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let url = format!(
            "https://api.rugcheck.xyz/v1/tokens/{}/report/summary",
            parsed.mint
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Failed to fetch rugcheck report: {}", response.status()),
            }));
        }

        let report: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "response": report,
        }))
    }
}

// =============================================================================
// PYTH_FETCH_PRICE Action
// =============================================================================

#[derive(Debug)]
pub struct PythFetchPriceAction {
    meta: ActionMetadata,
}

impl PythFetchPriceAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "tokenSymbol": {
                    "type": "string",
                    "description": "The token symbol to fetch price for (e.g., SOL, BTC, ETH)",
                }
            },
            "required": ["tokenSymbol"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({ "tokenSymbol": "SOL" }),
            output: json!({
                "status": "success",
                "price": "150.25",
                "message": "Current price: $150.25",
            }),
            explanation: "Get the current SOL/USD price from Pyth oracle".to_string(),
        }];

        let meta = ActionMetadata {
            name: "PYTH_FETCH_PRICE".to_string(),
            similes: vec![
                "get pyth price".to_string(),
                "pyth oracle price".to_string(),
                "oracle price".to_string(),
            ],
            description: "Fetch the current price from Pyth oracle price feed".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for PythFetchPriceAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            tokenSymbol: String,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let hermes_url = "https://hermes.pyth.network";

        let feed_url = format!(
            "{}/v2/price_feeds?query={}&asset_type=crypto",
            hermes_url, parsed.tokenSymbol
        );

        let client = reqwest::Client::new();
        let feed_response = client.get(&feed_url).send().await?;
        let feeds: Value = feed_response.json().await?;

        let feed_array = feeds.as_array().ok_or_else(|| anyhow::anyhow!("Invalid response"))?;
        if feed_array.is_empty() {
            return Ok(json!({
                "status": "error",
                "message": format!("No price feed found for {}", parsed.tokenSymbol),
            }));
        }

        let feed_id = feed_array
            .iter()
            .find(|f| {
                f["attributes"]["base"]
                    .as_str()
                    .map(|b| b.to_lowercase() == parsed.tokenSymbol.to_lowercase())
                    .unwrap_or(false)
            })
            .or_else(|| feed_array.first())
            .and_then(|f| f["id"].as_str())
            .ok_or_else(|| anyhow::anyhow!("No feed ID found"))?;

        let price_url = format!(
            "{}/v2/updates/price/latest?ids[]={}",
            hermes_url, feed_id
        );

        let price_response = client.get(&price_url).send().await?;
        let price_data: Value = price_response.json().await?;

        let parsed_data = price_data["parsed"]
            .as_array()
            .and_then(|a| a.first())
            .ok_or_else(|| anyhow::anyhow!("No price data"))?;

        let price = parsed_data["price"]["price"]
            .as_str()
            .or_else(|| parsed_data["price"]["price"].as_i64().map(|_| ""))
            .unwrap_or("0");
        let expo = parsed_data["price"]["expo"].as_i64().unwrap_or(0);

        let price_num: f64 = price.parse().unwrap_or(0.0);
        let actual_price = price_num * 10f64.powi(expo as i32);
        let formatted_price = format!("{:.2}", actual_price);

        Ok(json!({
            "status": "success",
            "price": formatted_price,
            "message": format!("Current price: ${}", formatted_price),
        }))
    }
}

// =============================================================================
// CREATE_LIMIT_ORDER Action
// =============================================================================

#[derive(Debug)]
pub struct CreateLimitOrderAction {
    meta: ActionMetadata,
}

impl CreateLimitOrderAction {
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
                "inAmount": {
                    "type": "string",
                    "description": "Input amount in base units",
                },
                "outAmount": {
                    "type": "string",
                    "description": "Expected output amount in base units",
                }
            },
            "required": ["inputMint", "outputMint", "inAmount", "outAmount"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "inputMint": "So11111111111111111111111111111111111111112",
                "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                "inAmount": "1000000000",
                "outAmount": "150000000",
            }),
            output: json!({
                "status": "success",
                "signature": "5K3N9...3J4",
            }),
            explanation: "Create a limit order to sell 1 SOL for 150 USDC".to_string(),
        }];

        let meta = ActionMetadata {
            name: "CREATE_LIMIT_ORDER".to_string(),
            similes: vec![
                "place limit order".to_string(),
                "submit limit order".to_string(),
                "jupiter limit order".to_string(),
            ],
            description: "Create a limit order on Jupiter Exchange".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for CreateLimitOrderAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, agent: &Agent, input: Value) -> Result<Value> {
        use solana_sdk::transaction::VersionedTransaction;

        #[derive(Deserialize)]
        struct Input {
            inputMint: String,
            outputMint: String,
            inAmount: String,
            outAmount: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        let url = "https://api.jup.ag/limit/v2/createOrder";

        let order_params = json!({
            "maker": agent.wallet.pubkey().to_string(),
            "payer": agent.wallet.pubkey().to_string(),
            "inputMint": parsed.inputMint,
            "outputMint": parsed.outputMint,
            "params": {
                "makingAmount": parsed.inAmount,
                "takingAmount": parsed.outAmount,
            }
        });

        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&order_params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Jupiter API error: {}", response.status()),
            }));
        }

        let data: Value = response.json().await?;
        let tx_b64 = data["tx"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No transaction in response"))?;

        let tx_bytes = base64_decode(tx_b64)?;
        let mut transaction: VersionedTransaction = bincode::deserialize(&tx_bytes)?;

        let latest_blockhash = agent.client.get_latest_blockhash()?;
        transaction.message.set_recent_blockhash(latest_blockhash);

        let signed_tx = agent.wallet.sign_transaction(transaction).await?;
        let signature = agent.client.send_and_confirm_transaction(&signed_tx)?;

        Ok(json!({
            "status": "success",
            "signature": signature.to_string(),
        }))
    }
}

// =============================================================================
// Register token actions
// =============================================================================

pub fn register_token_actions(registry: &mut ActionRegistry) {
    registry.register(GetBalanceAction::new());
    registry.register(TokenBalancesAction::new());
    registry.register(TransferAction::new());
    registry.register(WalletAddressAction::new());
    registry.register(GetTpsAction::new());
    registry.register(RequestFundsAction::new());
    registry.register(FetchPriceAction::new());
    registry.register(TradeAction::new());
    registry.register(GetJupiterTokenListAction::new());
    registry.register(SearchJupiterTokensAction::new());
    registry.register(RugcheckAction::new());
    registry.register(PythFetchPriceAction::new());
    registry.register(CreateLimitOrderAction::new());
}
