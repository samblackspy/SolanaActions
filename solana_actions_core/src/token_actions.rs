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
// Register token actions
// =============================================================================

pub fn register_token_actions(registry: &mut ActionRegistry) {
    registry.register(GetBalanceAction::new());
    registry.register(TokenBalancesAction::new());
    registry.register(TransferAction::new());
    registry.register(WalletAddressAction::new());
    registry.register(GetTpsAction::new());
    registry.register(RequestFundsAction::new());
}
