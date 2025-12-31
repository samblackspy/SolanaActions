# SolanaActions

A Rust implementation of the [Solana Agent Kit](https://github.com/sendaifun/solana-agent-kit) – enabling AI agents to interact with the Solana blockchain.

## Overview

This project provides a modular, plugin-based Rust framework for building AI agents that can:
- Check wallet balances and transfer tokens
- Trade tokens via Jupiter Exchange
- Fetch real-time token prices
- Request devnet/testnet funds
- Query network TPS
- And much more...

## Architecture

```
SolanaActions/
├── solana_actions_core/     # Core agent and action system
│   ├── src/
│   │   ├── agent.rs         # Agent struct (RPC client + wallet)
│   │   ├── wallet.rs        # Wallet trait + KeypairWallet
│   │   ├── actions.rs       # Action trait + ActionRegistry
│   │   └── token_actions.rs # Token-related actions
├── plugins/
│   ├── token/               # Token operations trait
│   ├── nft/                 # NFT operations (WIP)
│   ├── defi/                # DeFi integrations (WIP)
│   ├── misc/                # Miscellaneous actions (WIP)
│   └── blinks/              # Solana Blinks (WIP)
└── docs/                    # Scraped v2 documentation
```

## Available Actions

### Token Actions (18)

| Action | Description |
|--------|-------------|
| `BALANCE_ACTION` | Get SOL or SPL token balance |
| `TOKEN_BALANCE_ACTION` | Get all token balances for a wallet |
| `TRANSFER` | Transfer SOL or SPL tokens |
| `WALLET_ADDRESS` | Get the agent's wallet address |
| `GET_TPS` | Get current Solana network TPS |
| `REQUEST_FUNDS` | Request SOL from faucet (devnet/testnet) |
| `FETCH_PRICE` | Fetch token price in USDC via Jupiter |
| `TRADE` | Swap tokens using Jupiter Exchange |
| `RUGCHECK` | Check if a token is a rug pull via rugcheck.xyz |
| `STAKE_WITH_JUPITER` | Stake SOL to receive jupSOL |
| `PYTH_FETCH_PRICE` | Get oracle price from Pyth Network |
| `CREATE_LIMIT_ORDER` | Create Jupiter limit order |
| `GET_TOKEN_DATA` | Get comprehensive token data and metadata |
| `DEPLOY_TOKEN` | Deploy new SPL token with Metaplex metadata |
| `GET_JUPITER_TOKEN_LIST` | Get full token list from Jupiter |
| `SEARCH_JUPITER_TOKENS` | Search tokens by symbol/name/address |
| `LAUNCH_PUMPFUN_TOKEN` | Launch a token on Pump.fun (WIP) |
| `CLOSE_EMPTY_TOKEN_ACCOUNTS` | Close empty token accounts (WIP) |

### DeFi Actions (11)

| Action | Description |
|--------|-------------|
| `GET_SANCTUM_PRICE` | Fetch LST prices from Sanctum |
| `GET_SANCTUM_LST_APY` | Get APY for Liquid Staking Tokens |
| `STAKE_WITH_SOLAYER` | Stake SOL to receive sSOL via Solayer |
| `GET_DRIFT_MARKETS` | Get available Drift spot/perp markets |
| `GET_DEFI_RATES` | Get lending/borrowing rates across protocols |
| `SWAP_ON_RAYDIUM` | Swap tokens via Raydium (routes to Jupiter) |
| `GET_ORCA_WHIRLPOOLS` | Get Orca whirlpool liquidity pools data |
| `GET_RAYDIUM_POOLS` | Get Raydium AMM pool data (standard/concentrated) |
| `GET_METEORA_POOLS` | Get Meteora DLMM pool data |
| `GET_JUPITER_ROUTE_MAP` | Get Jupiter's indexed route map |
| `LULO_LEND` | Lend tokens using Lulo protocol (WIP) |

### NFT Actions (13)

| Action | Description |
|--------|-------------|
| `GET_ASSET` | Fetch NFT asset details via Metaplex DAS API |
| `SEARCH_ASSETS` | Search for NFT assets by owner, creator, or collection |
| `GET_ASSETS_BY_CREATOR` | Get NFT assets created by a specific address |
| `GET_ASSETS_BY_AUTHORITY` | Get NFT assets owned by a specific address |
| `DEPLOY_COLLECTION` | Deploy a new NFT collection with master edition |
| `MINT_NFT` | Mint a new NFT with metadata, optionally into a collection |
| `GET_MAGICEDEN_COLLECTION_STATS` | Get collection stats (floor price, volume) |
| `GET_POPULAR_MAGICEDEN_COLLECTIONS` | Fetch trending NFT collections |
| `GET_MAGICEDEN_COLLECTION_LISTINGS` | Get current NFT listings for a collection |
| `LIST_NFT_FOR_SALE` | List an NFT for sale on Tensor (WIP) |
| `CANCEL_NFT_LISTING` | Cancel an NFT listing on Tensor (WIP) |
| `LIST_MAGICEDEN_NFT` | List an NFT for sale on MagicEden |
| `BID_ON_MAGICEDEN_NFT` | Place a bid on an NFT on MagicEden |

### Misc Actions (24)

| Action | Description |
|--------|-------------|
| `GET_COINGECKO_TRENDING_TOKENS` | Get trending tokens from CoinGecko |
| `GET_COINGECKO_TOKEN_INFO` | Get detailed token information |
| `GET_COINGECKO_TOKEN_PRICE` | Get current token prices |
| `GET_COINGECKO_TOP_GAINERS` | Get top gaining tokens |
| `PARSE_TRANSACTION` | Parse transaction via Helius API |
| `RESOLVE_SOL_DOMAIN` | Resolve .sol domain to wallet address |
| `GET_DOMAIN_FOR_WALLET` | Reverse lookup - get domain for wallet |
| `CREATE_HELIUS_WEBHOOK` | Create a Helius webhook to monitor addresses |
| `GET_HELIUS_WEBHOOK` | Get details of a Helius webhook |
| `DELETE_HELIUS_WEBHOOK` | Delete a Helius webhook |
| `SEND_TRANSACTION_WITH_PRIORITY` | Send transaction with priority fees via Helius |
| `GET_DEXSCREENER_TOKEN_PROFILES` | Get latest token profiles from Dexscreener |
| `GET_DEXSCREENER_BOOSTED_TOKENS` | Get boosted/trending tokens from Dexscreener |
| `GET_DEXSCREENER_PAIRS_BY_TOKEN` | Get all trading pairs for a token |
| `SEARCH_DEXSCREENER_PAIRS` | Search trading pairs by symbol/name |
| `GET_DEXSCREENER_PAIR_BY_ADDRESS` | Get pair details by pool address |
| `GET_ALL_DOMAINS_TLDS` | Get all supported TLDs from AllDomains |
| `GET_OWNED_DOMAINS` | Get all .sol domains owned by a wallet |
| `GET_FAVORITE_DOMAIN` | Get the favorite/primary domain for a wallet |
| `GET_BIRDEYE_TOKEN_OVERVIEW` | Get comprehensive token data from Birdeye |
| `GET_BIRDEYE_TOKEN_SECURITY` | Get token security info from Birdeye |
| `GET_BIRDEYE_TRENDING_TOKENS` | Get trending tokens from Birdeye |
| `GET_BIRDEYE_OHLCV` | Get OHLCV price history from Birdeye |
| `GET_BIRDEYE_TRADES` | Get recent trades for a token from Birdeye |

## Quick Start

```rust
use std::sync::Arc;
use solana_actions_core::{
    agent::Agent,
    wallet::KeypairWallet,
    ActionRegistry,
    register_all_actions,
};
use solana_sdk::signature::Keypair;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Create agent with wallet
    let keypair = Keypair::new();
    let wallet = Arc::new(KeypairWallet::new(keypair));
    let agent = Agent::new(wallet, "https://devnet.helius-rpc.com/?api-key=1ddccce4-3e55-49b5-a48f-a0efdfe94153");

    // 2. Build action registry
    let mut registry = ActionRegistry::new();
    register_all_actions(&mut registry);

    // 3. List available tools (for AI integration)
    for meta in registry.metadata() {
        println!("Action: {} - {}", meta.name, meta.description);
    }

    // 4. Execute an action
    let result = registry
        .execute("BALANCE_ACTION", &agent, json!({}))
        .await?;
    println!("Balance: {}", result);

    Ok(())
}
```

## Building

```bash
cargo build
```

## Documentation

See the `docs/v2/` directory for scraped Solana Agent Kit v2 documentation.

## License

MIT
