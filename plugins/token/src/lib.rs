// plugin/token/src/lib.rs
use solana_sdk::pubkey::Pubkey;
use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait TokenActions {
    async fn transfer(
        &self,
        to: Pubkey,
        amount: f64,
        mint: Option<Pubkey>,
    ) -> Result<String>;

    async fn get_balance(&self, mint: Option<Pubkey>) -> Result<f64>;

    // add more functions here
}