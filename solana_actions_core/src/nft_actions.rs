//! NFT-related actions for Solana Agent Kit
//!
//! Includes: Metaplex DAS API, MagicEden marketplace, Tensor trade.

use async_trait::async_trait;
use anyhow::Result;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::actions::{Action, ActionExample, ActionMetadata, ActionRegistry};
use crate::agent::Agent;

const MAGIC_EDEN_API_URL: &str = "https://api-mainnet.magiceden.dev/v2";

// =============================================================================
// GET_ASSET Action (Metaplex DAS API)
// =============================================================================

#[derive(Debug)]
pub struct GetAssetAction {
    meta: ActionMetadata,
}

impl GetAssetAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "assetId": {
                    "type": "string",
                    "description": "The asset ID (mint address) to fetch details for",
                }
            },
            "required": ["assetId"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "assetId": "8x2dR8Mpzuz2YqyZyZjUbYWKSWesBo5jMx2Q9Y86udVk",
            }),
            output: json!({
                "status": "success",
                "message": "Asset retrieved successfully",
                "result": {
                    "name": "Example NFT",
                    "symbol": "ENFT",
                    "uri": "https://example.com/asset.json",
                },
            }),
            explanation: "Fetch details of an NFT asset using its ID".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_ASSET".to_string(),
            similes: vec![
                "fetch asset".to_string(),
                "retrieve asset".to_string(),
                "get asset details".to_string(),
                "get nft details".to_string(),
            ],
            description: "Fetch asset details using the Metaplex DAS API".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetAssetAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            assetId: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        // Use Helius DAS API (or similar RPC with DAS support)
        // The agent's RPC URL should point to a DAS-enabled endpoint
        let rpc_url = agent.client.url();
        
        let request = json!({
            "jsonrpc": "2.0",
            "id": "get-asset",
            "method": "getAsset",
            "params": {
                "id": parsed.assetId,
            },
        });

        let client = reqwest::Client::new();
        let response = client
            .post(rpc_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let data: Value = response.json().await?;

        if let Some(error) = data.get("error") {
            return Ok(json!({
                "status": "error",
                "message": format!("DAS API error: {}", error),
            }));
        }

        Ok(json!({
            "status": "success",
            "message": "Asset retrieved successfully",
            "result": data["result"],
        }))
    }
}

// =============================================================================
// GET_MAGICEDEN_COLLECTION_STATS Action
// =============================================================================

#[derive(Debug)]
pub struct GetMagicEdenCollectionStatsAction {
    meta: ActionMetadata,
}

impl GetMagicEdenCollectionStatsAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "collectionSymbol": {
                    "type": "string",
                    "description": "The symbol/ID of the NFT collection",
                },
                "timeWindow": {
                    "type": "string",
                    "enum": ["24h", "7d", "30d"],
                    "description": "Time window for stats (default: 24h)",
                }
            },
            "required": ["collectionSymbol"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "collectionSymbol": "degods",
                "timeWindow": "24h",
            }),
            output: json!({
                "status": "success",
                "stats": {
                    "symbol": "degods",
                    "floorPrice": 25.5,
                    "listedCount": 150,
                    "volumeAll": 50000,
                },
            }),
            explanation: "Fetch statistics for the DeGods collection on MagicEden".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_MAGICEDEN_COLLECTION_STATS".to_string(),
            similes: vec![
                "get collection stats".to_string(),
                "magiceden collection stats".to_string(),
                "nft collection statistics".to_string(),
                "floor price".to_string(),
            ],
            description: "Fetch statistics for a specific NFT collection on MagicEden including floor price, volume, and listed count".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetMagicEdenCollectionStatsAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            collectionSymbol: String,
            timeWindow: Option<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let time_window = parsed.timeWindow.unwrap_or_else(|| "24h".to_string());

        let url = format!(
            "{}/collections/{}/stats?timeWindow={}",
            MAGIC_EDEN_API_URL, parsed.collectionSymbol, time_window
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Failed to fetch collection stats: {}", response.status()),
            }));
        }

        let stats: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "message": "Collection stats fetched successfully",
            "stats": stats,
        }))
    }
}

// =============================================================================
// GET_POPULAR_MAGICEDEN_COLLECTIONS Action
// =============================================================================

#[derive(Debug)]
pub struct GetPopularMagicEdenCollectionsAction {
    meta: ActionMetadata,
}

impl GetPopularMagicEdenCollectionsAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "timeRange": {
                    "type": "string",
                    "enum": ["1h", "1d", "7d", "30d"],
                    "description": "Time range for popularity ranking (default: 1d)",
                }
            },
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({ "timeRange": "1d" }),
            output: json!({
                "status": "success",
                "collections": [
                    {
                        "symbol": "degods",
                        "name": "DeGods",
                        "floorPrice": 25.5,
                        "volumeAll": 50000,
                    }
                ],
            }),
            explanation: "Fetch popular NFT collections from MagicEden for the last day".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_POPULAR_MAGICEDEN_COLLECTIONS".to_string(),
            similes: vec![
                "get popular collections".to_string(),
                "trending nft collections".to_string(),
                "top nft collections".to_string(),
                "magiceden popular".to_string(),
            ],
            description: "Fetch popular/trending NFT collections from MagicEden marketplace".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetPopularMagicEdenCollectionsAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            timeRange: Option<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let time_range = parsed.timeRange.unwrap_or_else(|| "1d".to_string());

        let url = format!(
            "{}/marketplace/popular_collections?timeRange={}",
            MAGIC_EDEN_API_URL, time_range
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Failed to fetch popular collections: {}", response.status()),
            }));
        }

        let collections: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "message": "Popular collections fetched successfully",
            "collections": collections,
        }))
    }
}

// =============================================================================
// GET_MAGICEDEN_COLLECTION_LISTINGS Action
// =============================================================================

#[derive(Debug)]
pub struct GetMagicEdenCollectionListingsAction {
    meta: ActionMetadata,
}

impl GetMagicEdenCollectionListingsAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "collectionSymbol": {
                    "type": "string",
                    "description": "The symbol/ID of the NFT collection",
                },
                "limit": {
                    "type": "integer",
                    "description": "Number of listings to fetch (default: 20)",
                },
                "offset": {
                    "type": "integer",
                    "description": "Offset for pagination (default: 0)",
                }
            },
            "required": ["collectionSymbol"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "collectionSymbol": "degods",
                "limit": 10,
            }),
            output: json!({
                "status": "success",
                "listings": [
                    {
                        "tokenMint": "abc123...",
                        "price": 25.5,
                        "seller": "xyz789...",
                    }
                ],
            }),
            explanation: "Fetch current listings for the DeGods collection".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_MAGICEDEN_COLLECTION_LISTINGS".to_string(),
            similes: vec![
                "get collection listings".to_string(),
                "nft listings".to_string(),
                "magiceden listings".to_string(),
                "collection for sale".to_string(),
            ],
            description: "Fetch current NFT listings for a collection on MagicEden marketplace".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetMagicEdenCollectionListingsAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            collectionSymbol: String,
            limit: Option<u32>,
            offset: Option<u32>,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let limit = parsed.limit.unwrap_or(20);
        let offset = parsed.offset.unwrap_or(0);

        let url = format!(
            "{}/collections/{}/listings?limit={}&offset={}",
            MAGIC_EDEN_API_URL, parsed.collectionSymbol, limit, offset
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(json!({
                "status": "error",
                "message": format!("Failed to fetch listings: {}", response.status()),
            }));
        }

        let listings: Value = response.json().await?;

        Ok(json!({
            "status": "success",
            "message": "Listings fetched successfully",
            "listings": listings,
        }))
    }
}

// =============================================================================
// SEARCH_ASSETS Action (Metaplex DAS API)
// =============================================================================

#[derive(Debug)]
pub struct SearchAssetsAction {
    meta: ActionMetadata,
}

impl SearchAssetsAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "ownerAddress": {
                    "type": "string",
                    "description": "Owner wallet address to search assets for",
                },
                "creatorAddress": {
                    "type": "string",
                    "description": "Creator address to filter by",
                },
                "groupKey": {
                    "type": "string",
                    "description": "Group key (e.g., 'collection')",
                },
                "groupValue": {
                    "type": "string",
                    "description": "Group value (e.g., collection address)",
                },
                "page": {
                    "type": "integer",
                    "description": "Page number for pagination",
                },
                "limit": {
                    "type": "integer",
                    "description": "Number of results per page",
                }
            },
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "ownerAddress": "8x2dR8Mpzuz2YqyZyZjUbYWKSWesBo5jMx2Q9Y86udVk",
                "limit": 10,
            }),
            output: json!({
                "status": "success",
                "assets": [
                    { "id": "abc123", "name": "My NFT" }
                ],
            }),
            explanation: "Search for NFT assets owned by a wallet".to_string(),
        }];

        let meta = ActionMetadata {
            name: "SEARCH_ASSETS".to_string(),
            similes: vec![
                "search nfts".to_string(),
                "find assets".to_string(),
                "get wallet nfts".to_string(),
                "list owned nfts".to_string(),
            ],
            description: "Search for NFT assets using the Metaplex DAS API with various filters".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for SearchAssetsAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Input {
            ownerAddress: Option<String>,
            creatorAddress: Option<String>,
            groupKey: Option<String>,
            groupValue: Option<String>,
            page: Option<u32>,
            limit: Option<u32>,
        }

        let parsed: Input = serde_json::from_value(input.clone())?;

        let rpc_url = agent.client.url();
        
        let mut params = json!({});
        if let Some(owner) = parsed.ownerAddress {
            params["ownerAddress"] = json!(owner);
        }
        if let Some(creator) = parsed.creatorAddress {
            params["creatorAddress"] = json!(creator);
        }
        if let Some(key) = parsed.groupKey {
            params["groupKey"] = json!(key);
        }
        if let Some(val) = parsed.groupValue {
            params["groupValue"] = json!(val);
        }
        params["page"] = json!(parsed.page.unwrap_or(1));
        params["limit"] = json!(parsed.limit.unwrap_or(10));

        let request = json!({
            "jsonrpc": "2.0",
            "id": "search-assets",
            "method": "searchAssets",
            "params": params,
        });

        let client = reqwest::Client::new();
        let response = client
            .post(rpc_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let data: Value = response.json().await?;

        if let Some(error) = data.get("error") {
            return Ok(json!({
                "status": "error",
                "message": format!("DAS API error: {}", error),
            }));
        }

        Ok(json!({
            "status": "success",
            "message": "Assets retrieved successfully",
            "assets": data["result"]["items"],
            "total": data["result"]["total"],
        }))
    }
}

// =============================================================================
// GET_ASSETS_BY_CREATOR Action
// =============================================================================

#[derive(Debug)]
pub struct GetAssetsByCreatorAction {
    meta: ActionMetadata,
}

impl GetAssetsByCreatorAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "creator": {
                    "type": "string",
                    "description": "Creator wallet address",
                },
                "onlyVerified": {
                    "type": "boolean",
                    "description": "Only return verified creator assets",
                },
                "limit": {
                    "type": "integer",
                    "description": "Max number of results (default 10)",
                },
                "page": {
                    "type": "integer",
                    "description": "Page number for pagination",
                }
            },
            "required": ["creator"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "creator": "D3XrkNZz6wx6cofot7Zohsf2KSsu2ArngNk8VqU9cTY3",
                "onlyVerified": true,
                "limit": 10,
            }),
            output: json!({
                "status": "success",
                "assets": [],
                "total": 0,
            }),
            explanation: "Get NFT assets created by a specific address".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_ASSETS_BY_CREATOR".to_string(),
            similes: vec![
                "fetch assets by creator".to_string(),
                "get creator assets".to_string(),
                "creator nfts".to_string(),
            ],
            description: "Fetch a list of assets created by a specific address using Metaplex DAS API".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetAssetsByCreatorAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            creator: String,
            onlyVerified: Option<bool>,
            limit: Option<u32>,
            page: Option<u32>,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let rpc_url = agent.client.url();

        let request = json!({
            "jsonrpc": "2.0",
            "id": "get-assets-by-creator",
            "method": "getAssetsByCreator",
            "params": {
                "creatorAddress": parsed.creator,
                "onlyVerified": parsed.onlyVerified.unwrap_or(false),
                "limit": parsed.limit.unwrap_or(10),
                "page": parsed.page.unwrap_or(1),
            }
        });

        let client = reqwest::Client::new();
        let response = client
            .post(rpc_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let data: Value = response.json().await?;

        if let Some(error) = data.get("error") {
            return Ok(json!({
                "status": "error",
                "message": format!("DAS API error: {}", error),
            }));
        }

        Ok(json!({
            "status": "success",
            "assets": data["result"]["items"],
            "total": data["result"]["total"],
        }))
    }
}

// =============================================================================
// GET_ASSETS_BY_AUTHORITY Action
// =============================================================================

#[derive(Debug)]
pub struct GetAssetsByAuthorityAction {
    meta: ActionMetadata,
}

impl GetAssetsByAuthorityAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "authority": {
                    "type": "string",
                    "description": "Authority wallet address",
                },
                "limit": {
                    "type": "integer",
                    "description": "Max number of results (default 10)",
                },
                "page": {
                    "type": "integer",
                    "description": "Page number for pagination",
                },
                "before": {
                    "type": "string",
                    "description": "Cursor for pagination (before)",
                },
                "after": {
                    "type": "string",
                    "description": "Cursor for pagination (after)",
                }
            },
            "required": ["authority"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "authority": "mRdta4rc2RtsxEUDYuvKLamMZAdW6qHcwuq866Skxxv",
                "limit": 10,
            }),
            output: json!({
                "status": "success",
                "assets": [],
                "total": 0,
            }),
            explanation: "Get NFT assets owned by a specific authority address".to_string(),
        }];

        let meta = ActionMetadata {
            name: "GET_ASSETS_BY_AUTHORITY".to_string(),
            similes: vec![
                "fetch assets by authority".to_string(),
                "get authority assets".to_string(),
                "authority nfts".to_string(),
                "owned assets".to_string(),
            ],
            description: "Fetch a list of assets owned by a specific address using Metaplex DAS API".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for GetAssetsByAuthorityAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            authority: String,
            limit: Option<u32>,
            page: Option<u32>,
            before: Option<String>,
            after: Option<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let rpc_url = agent.client.url();

        let mut params = json!({
            "authorityAddress": parsed.authority,
            "limit": parsed.limit.unwrap_or(10),
            "page": parsed.page.unwrap_or(1),
        });

        if let Some(before) = parsed.before {
            params["before"] = json!(before);
        }
        if let Some(after) = parsed.after {
            params["after"] = json!(after);
        }

        let request = json!({
            "jsonrpc": "2.0",
            "id": "get-assets-by-authority",
            "method": "getAssetsByAuthority",
            "params": params,
        });

        let client = reqwest::Client::new();
        let response = client
            .post(rpc_url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let data: Value = response.json().await?;

        if let Some(error) = data.get("error") {
            return Ok(json!({
                "status": "error",
                "message": format!("DAS API error: {}", error),
            }));
        }

        Ok(json!({
            "status": "success",
            "assets": data["result"]["items"],
            "total": data["result"]["total"],
        }))
    }
}

// =============================================================================
// DEPLOY_COLLECTION Action (Metaplex)
// =============================================================================

#[derive(Debug)]
pub struct DeployCollectionAction {
    meta: ActionMetadata,
}

impl DeployCollectionAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the NFT collection",
                },
                "uri": {
                    "type": "string",
                    "description": "Metadata URI for the collection (must be a valid URL)",
                },
                "royaltyBasisPoints": {
                    "type": "integer",
                    "description": "Royalty in basis points (100 = 1%)",
                }
            },
            "required": ["name", "uri"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "name": "My Collection",
                "uri": "https://example.com/collection.json",
                "royaltyBasisPoints": 500,
            }),
            output: json!({
                "status": "success",
                "collectionAddress": "7nE9Gvc...",
                "name": "My Collection",
            }),
            explanation: "Deploy an NFT collection with 5% royalty".to_string(),
        }];

        let meta = ActionMetadata {
            name: "DEPLOY_COLLECTION".to_string(),
            similes: vec![
                "create collection".to_string(),
                "launch collection".to_string(),
                "deploy nft collection".to_string(),
                "mint collection".to_string(),
            ],
            description: "Deploy a new NFT collection on Solana blockchain using Metaplex".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for DeployCollectionAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, agent: &Agent, input: Value) -> Result<Value> {
        use mpl_token_metadata::instructions::{
            CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs,
            CreateMasterEditionV3, CreateMasterEditionV3InstructionArgs,
        };
        use mpl_token_metadata::types::{DataV2, Creator};
        use solana_sdk::pubkey::Pubkey;
        use solana_sdk::signature::Keypair;
        use solana_sdk::signer::Signer;
        use solana_sdk::system_instruction;
        use solana_sdk::instruction::Instruction;
        use solana_sdk::message::{self, VersionedMessage};
        use solana_sdk::transaction::VersionedTransaction;
        use solana_sdk::program_pack::Pack;
        use spl_token::instruction as token_instruction;
        use spl_associated_token_account::get_associated_token_address;

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Input {
            name: String,
            uri: String,
            symbol: Option<String>,
            royaltyBasisPoints: Option<u16>,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let payer = agent.wallet.pubkey();
        let symbol = parsed.symbol.unwrap_or_else(|| "".to_string());
        let royalty_basis_points = parsed.royaltyBasisPoints.unwrap_or(500); // Default 5%

        // Generate a new keypair for the collection mint
        let mint_keypair = Keypair::new();
        let mint_pubkey = mint_keypair.pubkey();

        // Calculate rent for mint account
        let mint_rent = agent.client.get_minimum_balance_for_rent_exemption(
            spl_token::state::Mint::LEN
        )?;

        let mut instructions: Vec<Instruction> = Vec::new();

        // 1. Create mint account
        instructions.push(system_instruction::create_account(
            &payer,
            &mint_pubkey,
            mint_rent,
            spl_token::state::Mint::LEN as u64,
            &spl_token::id(),
        ));

        // 2. Initialize mint (0 decimals for NFT)
        instructions.push(token_instruction::initialize_mint(
            &spl_token::id(),
            &mint_pubkey,
            &payer,        // mint authority
            Some(&payer),  // freeze authority
            0,             // 0 decimals for NFT
        )?);

        // 3. Create associated token account
        let ata = get_associated_token_address(&payer, &mint_pubkey);
        instructions.push(
            spl_associated_token_account::instruction::create_associated_token_account(
                &payer,
                &payer,
                &mint_pubkey,
                &spl_token::id(),
            )
        );

        // 4. Mint 1 token (the collection NFT)
        instructions.push(token_instruction::mint_to(
            &spl_token::id(),
            &mint_pubkey,
            &ata,
            &payer,
            &[],
            1,
        )?);

        // 5. Create metadata account
        let metadata_seeds = &[
            b"metadata",
            mpl_token_metadata::ID.as_ref(),
            mint_pubkey.as_ref(),
        ];
        let (metadata_pda, _) = Pubkey::find_program_address(
            metadata_seeds,
            &mpl_token_metadata::ID,
        );

        let create_metadata_ix = CreateMetadataAccountV3 {
            metadata: metadata_pda,
            mint: mint_pubkey,
            mint_authority: payer,
            payer,
            update_authority: (payer, true),
            system_program: solana_sdk::system_program::id(),
            rent: None,
        };

        let metadata_args = CreateMetadataAccountV3InstructionArgs {
            data: DataV2 {
                name: parsed.name.clone(),
                symbol: symbol.clone(),
                uri: parsed.uri.clone(),
                seller_fee_basis_points: royalty_basis_points,
                creators: Some(vec![Creator {
                    address: payer,
                    verified: true,
                    share: 100,
                }]),
                collection: None,
                uses: None,
            },
            is_mutable: true,
            collection_details: Some(mpl_token_metadata::types::CollectionDetails::V1 { size: 0 }),
        };

        instructions.push(create_metadata_ix.instruction(metadata_args));

        // 6. Create master edition (makes it a proper NFT with limited supply of 1)
        let edition_seeds = &[
            b"metadata",
            mpl_token_metadata::ID.as_ref(),
            mint_pubkey.as_ref(),
            b"edition",
        ];
        let (edition_pda, _) = Pubkey::find_program_address(
            edition_seeds,
            &mpl_token_metadata::ID,
        );

        let create_edition_ix = CreateMasterEditionV3 {
            edition: edition_pda,
            mint: mint_pubkey,
            update_authority: payer,
            mint_authority: payer,
            payer,
            metadata: metadata_pda,
            token_program: spl_token::id(),
            system_program: solana_sdk::system_program::id(),
            rent: None,
        };

        let edition_args = CreateMasterEditionV3InstructionArgs {
            max_supply: Some(0), // 0 means unlimited editions can be printed (for collection)
        };

        instructions.push(create_edition_ix.instruction(edition_args));

        // Build and sign transaction
        let latest_blockhash = agent.client.get_latest_blockhash()?;
        let message = VersionedMessage::V0(message::v0::Message::try_compile(
            &payer,
            &instructions,
            &[],
            latest_blockhash,
        )?);

        let tx = VersionedTransaction {
            signatures: vec![],
            message,
        };

        // Sign with mint keypair
        let message_bytes = tx.message.serialize();
        let mint_sig = mint_keypair.sign_message(&message_bytes);
        
        // Sign with wallet
        let signed_tx = agent.wallet.sign_transaction(tx).await?;
        
        // Add mint signature
        let mut final_tx = signed_tx;
        if final_tx.signatures.len() >= 2 {
            final_tx.signatures[1] = mint_sig;
        } else {
            final_tx.signatures.push(mint_sig);
        }

        let signature = agent.client.send_and_confirm_transaction(&final_tx)?;

        Ok(json!({
            "status": "success",
            "collectionAddress": mint_pubkey.to_string(),
            "metadata": metadata_pda.to_string(),
            "masterEdition": edition_pda.to_string(),
            "signature": signature.to_string(),
            "name": parsed.name,
            "symbol": symbol,
            "royaltyBasisPoints": royalty_basis_points,
        }))
    }
}

// =============================================================================
// MINT_NFT Action - Mint an NFT (optionally into a collection)
// =============================================================================

#[derive(Debug)]
pub struct MintNftAction {
    meta: ActionMetadata,
}

impl MintNftAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the NFT",
                },
                "uri": {
                    "type": "string",
                    "description": "Metadata URI for the NFT (must be a valid URL pointing to JSON)",
                },
                "symbol": {
                    "type": "string",
                    "description": "Symbol for the NFT (optional)",
                },
                "sellerFeeBasisPoints": {
                    "type": "integer",
                    "description": "Royalty in basis points (100 = 1%, default 500 = 5%)",
                },
                "collectionMint": {
                    "type": "string",
                    "description": "Collection mint address to add NFT to (optional)",
                },
                "recipient": {
                    "type": "string",
                    "description": "Recipient wallet address (defaults to payer)",
                }
            },
            "required": ["name", "uri"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "name": "My NFT #1",
                "uri": "https://example.com/nft1.json",
                "symbol": "MNFT",
                "sellerFeeBasisPoints": 500,
            }),
            output: json!({
                "status": "success",
                "mint": "7nE9Gvc...",
                "metadata": "8mF0Hwd...",
                "signature": "5xY2Abc...",
            }),
            explanation: "Mint a new NFT with 5% royalty".to_string(),
        }];

        let meta = ActionMetadata {
            name: "MINT_NFT".to_string(),
            similes: vec![
                "create nft".to_string(),
                "mint nft".to_string(),
                "create collectible".to_string(),
                "mint collectible".to_string(),
            ],
            description: "Mint a new NFT on Solana with Metaplex metadata, optionally as part of a collection".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for MintNftAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, agent: &Agent, input: Value) -> Result<Value> {
        use mpl_token_metadata::instructions::{
            CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs,
            CreateMasterEditionV3, CreateMasterEditionV3InstructionArgs,
        };
        use mpl_token_metadata::types::{DataV2, Creator, Collection};
        use solana_sdk::pubkey::Pubkey;
        use solana_sdk::signature::Keypair;
        use solana_sdk::signer::Signer;
        use solana_sdk::system_instruction;
        use solana_sdk::instruction::Instruction;
        use solana_sdk::message::{self, VersionedMessage};
        use solana_sdk::transaction::VersionedTransaction;
        use solana_sdk::program_pack::Pack;
        use spl_token::instruction as token_instruction;
        use spl_associated_token_account::get_associated_token_address;
        use std::str::FromStr;

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Input {
            name: String,
            uri: String,
            symbol: Option<String>,
            sellerFeeBasisPoints: Option<u16>,
            collectionMint: Option<String>,
            recipient: Option<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let payer = agent.wallet.pubkey();
        let symbol = parsed.symbol.unwrap_or_else(|| "".to_string());
        let seller_fee_basis_points = parsed.sellerFeeBasisPoints.unwrap_or(500);
        
        // Determine recipient (default to payer)
        let recipient = if let Some(ref addr) = parsed.recipient {
            Pubkey::from_str(addr).map_err(|e| anyhow::anyhow!("Invalid recipient address: {}", e))?
        } else {
            payer
        };

        // Parse collection mint if provided
        let collection = if let Some(ref coll_mint) = parsed.collectionMint {
            let coll_pubkey = Pubkey::from_str(coll_mint)
                .map_err(|e| anyhow::anyhow!("Invalid collection mint: {}", e))?;
            Some(Collection {
                verified: false, // Will need to be verified separately by collection authority
                key: coll_pubkey,
            })
        } else {
            None
        };

        // Generate a new keypair for the NFT mint
        let mint_keypair = Keypair::new();
        let mint_pubkey = mint_keypair.pubkey();

        // Calculate rent for mint account
        let mint_rent = agent.client.get_minimum_balance_for_rent_exemption(
            spl_token::state::Mint::LEN
        )?;

        let mut instructions: Vec<Instruction> = Vec::new();

        // 1. Create mint account
        instructions.push(system_instruction::create_account(
            &payer,
            &mint_pubkey,
            mint_rent,
            spl_token::state::Mint::LEN as u64,
            &spl_token::id(),
        ));

        // 2. Initialize mint (0 decimals for NFT)
        instructions.push(token_instruction::initialize_mint(
            &spl_token::id(),
            &mint_pubkey,
            &payer,        // mint authority
            Some(&payer),  // freeze authority
            0,             // 0 decimals for NFT
        )?);

        // 3. Create associated token account for recipient
        let ata = get_associated_token_address(&recipient, &mint_pubkey);
        instructions.push(
            spl_associated_token_account::instruction::create_associated_token_account(
                &payer,
                &recipient,
                &mint_pubkey,
                &spl_token::id(),
            )
        );

        // 4. Mint 1 token (the NFT)
        instructions.push(token_instruction::mint_to(
            &spl_token::id(),
            &mint_pubkey,
            &ata,
            &payer,
            &[],
            1,
        )?);

        // 5. Create metadata account
        let metadata_seeds = &[
            b"metadata",
            mpl_token_metadata::ID.as_ref(),
            mint_pubkey.as_ref(),
        ];
        let (metadata_pda, _) = Pubkey::find_program_address(
            metadata_seeds,
            &mpl_token_metadata::ID,
        );

        let create_metadata_ix = CreateMetadataAccountV3 {
            metadata: metadata_pda,
            mint: mint_pubkey,
            mint_authority: payer,
            payer,
            update_authority: (payer, true),
            system_program: solana_sdk::system_program::id(),
            rent: None,
        };

        let metadata_args = CreateMetadataAccountV3InstructionArgs {
            data: DataV2 {
                name: parsed.name.clone(),
                symbol: symbol.clone(),
                uri: parsed.uri.clone(),
                seller_fee_basis_points,
                creators: Some(vec![Creator {
                    address: payer,
                    verified: true,
                    share: 100,
                }]),
                collection,
                uses: None,
            },
            is_mutable: true,
            collection_details: None,
        };

        instructions.push(create_metadata_ix.instruction(metadata_args));

        // 6. Create master edition (makes it a proper NFT with supply of 1)
        let edition_seeds = &[
            b"metadata",
            mpl_token_metadata::ID.as_ref(),
            mint_pubkey.as_ref(),
            b"edition",
        ];
        let (edition_pda, _) = Pubkey::find_program_address(
            edition_seeds,
            &mpl_token_metadata::ID,
        );

        let create_edition_ix = CreateMasterEditionV3 {
            edition: edition_pda,
            mint: mint_pubkey,
            update_authority: payer,
            mint_authority: payer,
            payer,
            metadata: metadata_pda,
            token_program: spl_token::id(),
            system_program: solana_sdk::system_program::id(),
            rent: None,
        };

        let edition_args = CreateMasterEditionV3InstructionArgs {
            max_supply: Some(0), // 0 = non-fungible (only 1 can exist)
        };

        instructions.push(create_edition_ix.instruction(edition_args));

        // Build and sign transaction
        let latest_blockhash = agent.client.get_latest_blockhash()?;
        let message = VersionedMessage::V0(message::v0::Message::try_compile(
            &payer,
            &instructions,
            &[],
            latest_blockhash,
        )?);

        let tx = VersionedTransaction {
            signatures: vec![],
            message,
        };

        // Sign with mint keypair
        let message_bytes = tx.message.serialize();
        let mint_sig = mint_keypair.sign_message(&message_bytes);
        
        // Sign with wallet
        let signed_tx = agent.wallet.sign_transaction(tx).await?;
        
        // Add mint signature
        let mut final_tx = signed_tx;
        if final_tx.signatures.len() >= 2 {
            final_tx.signatures[1] = mint_sig;
        } else {
            final_tx.signatures.push(mint_sig);
        }

        let signature = agent.client.send_and_confirm_transaction(&final_tx)?;

        Ok(json!({
            "status": "success",
            "mint": mint_pubkey.to_string(),
            "metadata": metadata_pda.to_string(),
            "masterEdition": edition_pda.to_string(),
            "tokenAccount": ata.to_string(),
            "recipient": recipient.to_string(),
            "signature": signature.to_string(),
            "name": parsed.name,
            "symbol": symbol,
        }))
    }
}

// =============================================================================
// LIST_NFT_FOR_SALE Action (Tensor)
// =============================================================================

#[derive(Debug)]
pub struct ListNftForSaleAction {
    meta: ActionMetadata,
}

impl ListNftForSaleAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "nftMint": {
                    "type": "string",
                    "description": "Mint address of the NFT to list",
                },
                "price": {
                    "type": "number",
                    "description": "Price in SOL to list the NFT for",
                }
            },
            "required": ["nftMint", "price"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "nftMint": "DGxe4rqLMK9qvUK4LwwUBYyqUwxVXoWNUF9LdePpJzrh",
                "price": 2.5,
            }),
            output: json!({
                "status": "success",
                "signature": "2ZE7Rz...",
                "message": "Successfully listed NFT for 2.5 SOL",
            }),
            explanation: "List an NFT for sale on Tensor for 2.5 SOL".to_string(),
        }];

        let meta = ActionMetadata {
            name: "LIST_NFT_FOR_SALE".to_string(),
            similes: vec![
                "list nft".to_string(),
                "sell nft".to_string(),
                "tensor list".to_string(),
                "put nft for sale".to_string(),
            ],
            description: "List an NFT for sale on Tensor marketplace".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for ListNftForSaleAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            nftMint: String,
            price: f64,
        }

        let parsed: Input = serde_json::from_value(input)?;

        // Tensor API requires authentication and specific SDK integration
        Ok(json!({
            "status": "info",
            "message": "Tensor listing requires Tensor SDK integration",
            "input": {
                "nftMint": parsed.nftMint,
                "price": parsed.price,
            },
            "requirements": [
                "Tensor API key",
                "NFT ownership verification",
                "Transaction signing",
            ],
        }))
    }
}

// =============================================================================
// CANCEL_NFT_LISTING Action (Tensor)
// =============================================================================

#[derive(Debug)]
pub struct CancelNftListingAction {
    meta: ActionMetadata,
}

impl CancelNftListingAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "nftMint": {
                    "type": "string",
                    "description": "Mint address of the NFT listing to cancel",
                }
            },
            "required": ["nftMint"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "nftMint": "DGxe4rqLMK9qvUK4LwwUBYyqUwxVXoWNUF9LdePpJzrh",
            }),
            output: json!({
                "status": "success",
                "signature": "3YKpM1...",
                "message": "Successfully cancelled NFT listing",
            }),
            explanation: "Cancel an existing NFT listing on Tensor".to_string(),
        }];

        let meta = ActionMetadata {
            name: "CANCEL_NFT_LISTING".to_string(),
            similes: vec![
                "cancel nft listing".to_string(),
                "delist nft".to_string(),
                "remove nft listing".to_string(),
                "stop selling nft".to_string(),
            ],
            description: "Cancel an existing NFT listing on Tensor marketplace".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for CancelNftListingAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, _agent: &Agent, input: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Input {
            nftMint: String,
        }

        let parsed: Input = serde_json::from_value(input)?;

        Ok(json!({
            "status": "info",
            "message": "Tensor cancel listing requires Tensor SDK integration",
            "input": {
                "nftMint": parsed.nftMint,
            },
        }))
    }
}

// =============================================================================
// LIST_MAGICEDEN_NFT Action
// =============================================================================

#[derive(Debug)]
pub struct ListMagicEdenNftAction {
    meta: ActionMetadata,
}

impl ListMagicEdenNftAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "tokenMint": {
                    "type": "string",
                    "description": "Token mint address of the NFT",
                },
                "tokenAccount": {
                    "type": "string",
                    "description": "Token account address",
                },
                "price": {
                    "type": "number",
                    "description": "Price in SOL",
                },
                "magicEdenApiKey": {
                    "type": "string",
                    "description": "MagicEden API key for authentication",
                },
                "auctionHouseAddress": {
                    "type": "string",
                    "description": "Optional auction house address",
                }
            },
            "required": ["tokenMint", "tokenAccount", "price", "magicEdenApiKey"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "tokenMint": "TOKEN_MINT_ADDRESS",
                "tokenAccount": "TOKEN_ACCOUNT_ADDRESS",
                "price": 1.5,
                "magicEdenApiKey": "YOUR_API_KEY",
            }),
            output: json!({
                "status": "success",
                "message": "NFT listed successfully",
                "signature": "TRANSACTION_SIGNATURE",
            }),
            explanation: "List an NFT for sale on MagicEden".to_string(),
        }];

        let meta = ActionMetadata {
            name: "LIST_MAGICEDEN_NFT".to_string(),
            similes: vec![
                "list nft magiceden".to_string(),
                "sell nft magiceden".to_string(),
                "magiceden list".to_string(),
            ],
            description: "List an NFT for sale on MagicEden marketplace".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for ListMagicEdenNftAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, agent: &Agent, input: Value) -> Result<Value> {
        use solana_sdk::transaction::VersionedTransaction;

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Input {
            tokenMint: String,
            tokenAccount: String,
            price: f64,
            magicEdenApiKey: String,
            auctionHouseAddress: Option<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let seller = agent.wallet.pubkey();

        // Build MagicEden API request
        let client = reqwest::Client::new();
        let mut url = format!(
            "https://api-mainnet.magiceden.dev/v2/instructions/list?seller={}&tokenMint={}&price={}&tokenAccount={}",
            seller,
            parsed.tokenMint,
            parsed.price,
            parsed.tokenAccount
        );

        if let Some(ref auction_house) = parsed.auctionHouseAddress {
            url.push_str(&format!("&auctionHouseAddress={}", auction_house));
        }

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", parsed.magicEdenApiKey))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Ok(json!({
                "status": "error",
                "message": format!("MagicEden API error: {}", error_text),
            }));
        }

        let data: Value = response.json().await?;

        // Extract the signed transaction bytes
        let tx_signed = data.get("txSigned")
            .and_then(|t| t.get("data"))
            .ok_or_else(|| anyhow::anyhow!("Invalid response from MagicEden API: missing txSigned.data"))?;

        let tx_bytes: Vec<u8> = tx_signed
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("txSigned.data is not an array"))?
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n as u8))
            .collect();

        // Deserialize the transaction
        let tx: VersionedTransaction = bincode::deserialize(&tx_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize transaction: {}", e))?;

        // Sign and send
        let signed_tx = agent.wallet.sign_transaction(tx).await?;
        let signature = agent.client.send_and_confirm_transaction(&signed_tx)?;

        Ok(json!({
            "status": "success",
            "message": "NFT listed successfully on MagicEden",
            "signature": signature.to_string(),
            "tokenMint": parsed.tokenMint,
            "price": parsed.price,
        }))
    }
}

// =============================================================================
// BID_ON_MAGICEDEN_NFT Action
// =============================================================================

#[derive(Debug)]
pub struct BidOnMagicEdenNftAction {
    meta: ActionMetadata,
}

impl BidOnMagicEdenNftAction {
    pub fn new() -> Self {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "tokenMint": {
                    "type": "string",
                    "description": "Token mint address of the NFT",
                },
                "price": {
                    "type": "number",
                    "description": "Bid price in SOL",
                },
                "magicEdenApiKey": {
                    "type": "string",
                    "description": "MagicEden API key for authentication",
                },
                "auctionHouseAddress": {
                    "type": "string",
                    "description": "Optional auction house address",
                }
            },
            "required": ["tokenMint", "price", "magicEdenApiKey"],
            "additionalProperties": false,
        });

        let examples = vec![ActionExample {
            input: json!({
                "tokenMint": "TOKEN_MINT_ADDRESS",
                "price": 0.5,
                "magicEdenApiKey": "YOUR_API_KEY",
            }),
            output: json!({
                "status": "success",
                "message": "Bid placed successfully",
                "signature": "TRANSACTION_SIGNATURE",
            }),
            explanation: "Place a bid of 0.5 SOL on an NFT".to_string(),
        }];

        let meta = ActionMetadata {
            name: "BID_ON_MAGICEDEN_NFT".to_string(),
            similes: vec![
                "bid nft".to_string(),
                "offer on nft".to_string(),
                "magiceden bid".to_string(),
                "place bid".to_string(),
            ],
            description: "Place a bid on an NFT listed on MagicEden".to_string(),
            examples,
            input_schema,
        };

        Self { meta }
    }
}

#[async_trait]
impl Action for BidOnMagicEdenNftAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.meta
    }

    async fn call(&self, agent: &Agent, input: Value) -> Result<Value> {
        use solana_sdk::transaction::VersionedTransaction;

        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Input {
            tokenMint: String,
            price: f64,
            magicEdenApiKey: String,
            auctionHouseAddress: Option<String>,
        }

        let parsed: Input = serde_json::from_value(input)?;
        let buyer = agent.wallet.pubkey();

        // Build MagicEden API request for buy/bid
        let client = reqwest::Client::new();
        let mut url = format!(
            "https://api-mainnet.magiceden.dev/v2/instructions/buy?buyer={}&tokenMint={}&price={}",
            buyer,
            parsed.tokenMint,
            parsed.price
        );

        if let Some(ref auction_house) = parsed.auctionHouseAddress {
            url.push_str(&format!("&auctionHouseAddress={}", auction_house));
        }

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", parsed.magicEdenApiKey))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Ok(json!({
                "status": "error",
                "message": format!("MagicEden API error: {}", error_text),
            }));
        }

        let data: Value = response.json().await?;

        // Extract the signed transaction bytes
        let tx_signed = data.get("txSigned")
            .and_then(|t| t.get("data"))
            .ok_or_else(|| anyhow::anyhow!("Invalid response from MagicEden API: missing txSigned.data"))?;

        let tx_bytes: Vec<u8> = tx_signed
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("txSigned.data is not an array"))?
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n as u8))
            .collect();

        // Deserialize the transaction
        let tx: VersionedTransaction = bincode::deserialize(&tx_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize transaction: {}", e))?;

        // Sign and send
        let signed_tx = agent.wallet.sign_transaction(tx).await?;
        let signature = agent.client.send_and_confirm_transaction(&signed_tx)?;

        Ok(json!({
            "status": "success",
            "message": "Bid placed successfully on MagicEden",
            "signature": signature.to_string(),
            "tokenMint": parsed.tokenMint,
            "bidPrice": parsed.price,
        }))
    }
}

// =============================================================================
// Register all NFT actions
// =============================================================================

pub fn register_nft_actions(registry: &mut ActionRegistry) {
    registry.register(GetAssetAction::new());
    registry.register(GetMagicEdenCollectionStatsAction::new());
    registry.register(GetPopularMagicEdenCollectionsAction::new());
    registry.register(GetMagicEdenCollectionListingsAction::new());
    registry.register(SearchAssetsAction::new());
    registry.register(GetAssetsByCreatorAction::new());
    registry.register(GetAssetsByAuthorityAction::new());
    registry.register(DeployCollectionAction::new());
    registry.register(MintNftAction::new());
    registry.register(ListNftForSaleAction::new());
    registry.register(CancelNftListingAction::new());
    registry.register(ListMagicEdenNftAction::new());
    registry.register(BidOnMagicEdenNftAction::new());
}
