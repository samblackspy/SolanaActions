use std::sync::Arc;
use solana_client::rpc_client::RpcClient;
use crate::wallet::Wallet;
use solana_actions_token::TokenActions;
use anyhow::{Result, anyhow};
use solana_sdk::{
    pubkey::Pubkey,
    system_instruction,
    instruction::Instruction,
    transaction::VersionedTransaction,
    message::{self, VersionedMessage},
    native_token::LAMPORTS_PER_SOL,
    program_pack::Pack,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Mint;

///  core struct for interacting with the Solana blockchain.
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

/// Impl TokenActions trait for the core Agent.
#[async_trait::async_trait]
impl TokenActions for Agent {
    async fn get_balance(&self, mint: Option<Pubkey>) -> Result<f64> {
        let owner = self.wallet.pubkey();

        match mint {
            None => {
                let lamports = self.client.get_balance(&owner)?;
                Ok(lamports as f64 / LAMPORTS_PER_SOL as f64)
            }
            Some(mint_pubkey) => {
                let ata = get_associated_token_address(&owner, &mint_pubkey);
                let balance = match self.client.get_token_account_balance(&ata) {
                    Ok(ui_token_amount) => ui_token_amount.ui_amount.unwrap_or(0.0),
                    Err(_) => 0.0,
                };
                Ok(balance)
            }
        }
    }

    async fn transfer(
        &self,
        to: Pubkey,
        amount: f64,
        mint: Option<Pubkey>,
    ) -> Result<String> {
        let from_pubkey = self.wallet.pubkey();
        let mut instructions: Vec<Instruction> = Vec::new();

        match mint {
            None => {
                let lamports = (amount * LAMPORTS_PER_SOL as f64) as u64;
                if lamports == 0 { return Err(anyhow!("Transfer amount is too small")); }
                let transfer_ix = system_instruction::transfer(&from_pubkey, &to, lamports);
                instructions.push(transfer_ix);
            }
            Some(mint_pubkey) => {
                let mint_info = self.client.get_account(&mint_pubkey)?;
                let token_program_id = mint_info.owner;
                let token_mint_account = Mint::unpack(&mint_info.data)?;
                let decimals = token_mint_account.decimals;
                let amount_in_base_units = (amount * 10f64.powi(decimals as i32)) as u64;

                if amount_in_base_units == 0 { return Err(anyhow!("Transfer amount is too small")); }

                let source_ata = get_associated_token_address(&from_pubkey, &mint_pubkey);
                let dest_ata = get_associated_token_address(&to, &mint_pubkey);

                if self.client.get_account(&dest_ata).is_err() {
                    let create_ata_ix =
                        spl_associated_token_account::instruction::create_associated_token_account(
                            &from_pubkey,
                            &to,
                            &mint_pubkey,
                            &token_program_id,
                        );
                    instructions.push(create_ata_ix);
                }

                let transfer_ix = spl_token::instruction::transfer_checked(
                    &token_program_id,
                    &source_ata,
                    &mint_pubkey,
                    &dest_ata,
                    &from_pubkey,
                    &[],
                    amount_in_base_units,
                    decimals,
                )?;
                instructions.push(transfer_ix);
            }
        }

        let latest_blockhash = self.client.get_latest_blockhash()?;
        
        let message = VersionedMessage::V0(
            message::v0::Message::try_compile(
                &from_pubkey,
                &instructions,
                &[],
                latest_blockhash,
            )?
        );

        let tx = VersionedTransaction {
            signatures: vec![],
            message,
        };
        
        let signed_tx = self.wallet.sign_transaction(tx).await?;
        
        let signature = self.client.send_and_confirm_transaction(&signed_tx)?;

        Ok(signature.to_string())
    }
}