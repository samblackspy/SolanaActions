use std::sync::Arc;
use solana_client::rpc_client::RpcClient;
use crate::wallet::Wallet;
use solana_actions_token::TokenActions; 
/// core struct for interacting with the Solana blockchain.
pub struct Agent {
    pub client: Arc<RpcClient>,
    pub wallet: Arc<dyn Wallet>,
}

impl Agent {
    pub fn new(wallet: Arc<dyn Wallet>, rpc_url: &str) -> Self {
        Self {
            wallet,
            client: Arc::new(RpcClient::new(rpc_url.to_string())),
        }
    }
}

// implement logic for the token actions.
#[async_trait::async_trait]
impl TokenActions for Agent {
    async fn transfer(
        &self,
        _to: solana_sdk::pubkey::Pubkey,
        _amount: f64,
        _mint: Option<solana_sdk::pubkey::Pubkey>,
    ) -> anyhow::Result<String> {
        unimplemented!("Transfer logic not yet implemented")
    }

    async fn get_balance(
        &self,
        _mint: Option<solana_sdk::pubkey::Pubkey>,
    ) -> anyhow::Result<f64> {
        unimplemented!("Get balance logic not yet implemented")
    }
}