use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::agent::Agent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionExample {
    pub input: Value,
    pub output: Value,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionMetadata {
    pub name: String,
    pub similes: Vec<String>,
    pub description: String,
    pub examples: Vec<ActionExample>,
    pub input_schema: Value,
}

#[async_trait]
pub trait Action: Send + Sync {
    fn metadata(&self) -> &ActionMetadata;

    async fn call(&self, agent: &Agent, input: Value) -> Result<Value>;
}

#[derive(Default)]
pub struct ActionRegistry {
    actions: HashMap<String, Arc<dyn Action>>, 
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    pub fn register<A>(&mut self, action: A)
    where
        A: Action + 'static,
    {
        let action = Arc::new(action) as Arc<dyn Action>;
        let name = action.metadata().name.clone();
        self.actions.insert(name, action);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Action>> {
        self.actions.get(name).cloned()
    }

    pub fn all(&self) -> Vec<Arc<dyn Action>> {
        self.actions.values().cloned().collect()
    }

    /// Execute an action by name with the given JSON input.
    pub async fn execute(
        &self,
        name: &str,
        agent: &Agent,
        input: Value,
    ) -> Result<Value> {
        let action = self
            .get(name)
            .ok_or_else(|| anyhow!("Unknown action: {name}"))?;
        action.call(agent, input).await
    }

    /// Return metadata for all registered actions (useful for AI tool schemas).
    pub fn metadata(&self) -> Vec<ActionMetadata> {
        self.actions
            .values()
            .map(|a| a.metadata().clone())
            .collect()
    }
}
