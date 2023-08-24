use super::{ChatRequest, ChatResponse, History};
use crate::{chat::errors::ModelError, secret_manager::Secrets};
use async_trait::async_trait;

#[async_trait]
pub trait ChatLlm {
    fn name(&self) -> &str;
    async fn chat(
        &self,
        secrets: Secrets,
        prompt: String,
        system: Option<String>,
        history: Vec<History>,
    ) -> Result<String, ModelError>;
    fn system_limit(&self) -> usize {
        0
    }
    fn prompt_limit(&self) -> usize {
        0
    }
    fn response_limit(&self) -> usize {
        0
    }
    fn context_size(&self) -> usize;

    /// This should use the model tokenizer, but for now we'll just use a rough count of the number of words and multiply by 1.4
    fn count_tokens(&self, s: &str) -> usize {
        (s.split_whitespace().count() as f32 * 1.35).ceil() as usize
    }
}

impl ChatRequest {
    /// Age out old history and trim the prompt and system to fit the model limits
    pub fn trim(&mut self, llm: &dyn ChatLlm) -> Result<(), ModelError> {
        tracing::trace!("Trimming request");
        let mut total_tokens = 0;
        let system_tokens = self
            .system
            .as_ref()
            .map(|s| llm.count_tokens(s))
            .unwrap_or(0);
        let prompt_tokens = llm.count_tokens(&self.prompt);
        total_tokens += system_tokens + prompt_tokens;
        if 0 < llm.system_limit() && llm.system_limit() < system_tokens {
            tracing::debug!(
                "System too long: {} > {}",
                system_tokens,
                llm.system_limit()
            );
            return Err(ModelError::SystemTooLong);
        }
        if 0 < llm.prompt_limit() && llm.prompt_limit() < prompt_tokens {
            tracing::debug!(
                "Prompt too long: {} > {}",
                prompt_tokens,
                llm.prompt_limit()
            );
            return Err(ModelError::PromptTooLong);
        }
        let old_history_len = self.history.len() as u64;
        let mut new_history = Vec::new();
        while let Some(history) = self.history.pop() {
            let gen_tokens = llm.count_tokens(&history.generation);
            let historical_prompt_tokens = llm.count_tokens(&history.prompt);
            if 0 < llm.prompt_limit() && llm.prompt_limit() < historical_prompt_tokens {
                tracing::debug!(
                    "Historical prompt too long: {} > {}",
                    historical_prompt_tokens,
                    llm.prompt_limit()
                );
                return Err(ModelError::HistoryPromptTooLong(
                    old_history_len - new_history.len() as u64,
                ));
            }
            total_tokens += gen_tokens + historical_prompt_tokens;
            if llm.context_size() < total_tokens {
                break;
            }
            new_history.push(history);
        }
        new_history.reverse();
        tracing::debug!(
            "Trimmed history to {} items from {}",
            new_history.len(),
            old_history_len
        );
        self.history = new_history;
        Ok(())
    }
}

impl ChatResponse {
    pub fn to_redis_string(&self) -> String {
        format!("OK:{}", self.generation)
    }

    pub fn from_redis_string(s: &str, uuid: &str) -> Option<Self> {
        let (status, generation) = s.split_once(':')?;

        if status == "OK" {
            Some(Self {
                generation: generation.to_string(),
                uuid: uuid.to_string(),
            })
        } else {
            None
        }
    }
}
