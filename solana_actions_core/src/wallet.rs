use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
};
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

/// trait for signing Solana transactions.
/// allows for flexible wallet implementations, from local keypairs to remote signers.
#[async_trait]
pub trait Wallet: Send + Sync + Debug {
    fn pubkey(&self) -> Pubkey;
    async fn sign_transaction(
        &self,
        tx: VersionedTransaction,
    ) -> anyhow::Result<VersionedTransaction>;
    async fn sign_all_transactions(
        &self,
        txs: Vec<VersionedTransaction>,
    ) -> anyhow::Result<Vec<VersionedTransaction>>;
}

/// wallet implementation using a local Solana Keypair.
#[derive(Debug)]
pub struct KeypairWallet {
    pub keypair: Arc<Keypair>,
}

impl KeypairWallet {
    pub fn new(keypair: Keypair) -> Self {
        Self {
            keypair: Arc::new(keypair),
        }
    }
}

#[async_trait]
impl Wallet for KeypairWallet {
    fn pubkey(&self) -> Pubkey {
        self.keypair.pubkey()
    }

    async fn sign_transaction(
        &self,
        mut tx: VersionedTransaction,
    ) -> anyhow::Result<VersionedTransaction> {
        tx.sign(&[self.keypair.as_ref()], self.keypair.pubkey().into());
        Ok(tx)
    }

    async fn sign_all_transactions(
        &self,
        mut txs: Vec<VersionedTransaction>,
    ) -> anyhow::Result<Vec<VersionedTransaction>> {
        for tx in &mut txs {
            tx.sign(&[self.keypair.as_ref()], self.keypair.pubkey().into());
        }
        Ok(txs)
    }
}