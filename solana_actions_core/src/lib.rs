pub mod agent;
pub mod wallet;
pub mod actions;
pub mod token_actions;
pub mod defi_actions;
pub mod nft_actions;
pub mod misc_actions;

pub use actions::{Action, ActionExample, ActionMetadata, ActionRegistry};
pub use token_actions::register_token_actions;
pub use defi_actions::register_defi_actions;
pub use nft_actions::register_nft_actions;
pub use misc_actions::register_misc_actions;

/// Convenience helper to register all available actions for an agent.
/// As more domains are added (NFT, DeFi, misc, blinks, etc.), extend this
/// function to register their actions as well.
pub fn register_all_actions(registry: &mut ActionRegistry) {
    register_token_actions(registry);
    register_defi_actions(registry);
    register_nft_actions(registry);
    register_misc_actions(registry);
}