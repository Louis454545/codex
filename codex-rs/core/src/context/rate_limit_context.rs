//! Hidden user-context fragment for runtime-owned rate-limit steering prompts.

use super::ContextualUserFragment;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseInputItem;

/// Hidden runtime-owned rate-limit steering context injected into model input.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RateLimitContext {
    prompt: String,
}

impl RateLimitContext {
    pub(crate) fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
        }
    }

    pub(crate) fn into_response_input_item(self) -> ResponseInputItem {
        ResponseInputItem::Message {
            role: <Self as ContextualUserFragment>::role().to_string(),
            content: vec![ContentItem::InputText {
                text: self.render(),
            }],
            phase: None,
        }
    }
}

impl ContextualUserFragment for RateLimitContext {
    fn role() -> &'static str {
        "user"
    }

    fn markers(&self) -> (&'static str, &'static str) {
        Self::type_markers()
    }

    fn type_markers() -> (&'static str, &'static str) {
        ("<rate_limit_context>", "</rate_limit_context>")
    }

    fn body(&self) -> String {
        format!("\n{}\n", self.prompt)
    }
}
