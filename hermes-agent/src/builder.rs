use crate::{AgentConfig, AIAgent, IterationBudget};

pub struct AIAgentBuilder {
    config: AgentConfig,
    budget: Option<IterationBudget>,
}

impl AIAgentBuilder {
    pub fn new() -> Self {
        Self {
            config: AgentConfig::default(),
            budget: None,
        }
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.config.base_url = Some(base_url.into());
        self
    }

    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.config.api_key = Some(key.into());
        self
    }

    pub fn max_iterations(mut self, iters: usize) -> Self {
        self.config.max_iterations = iters;
        self
    }

    pub fn budget(mut self, budget: IterationBudget) -> Self {
        self.budget = Some(budget);
        self
    }

    pub fn build(self) -> AIAgent {
        let budget = self.budget.unwrap_or_else(|| IterationBudget::new(self.config.max_iterations));
        AIAgent {
            config: self.config,
            budget,
        }
    }
}

impl Default for AIAgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}
