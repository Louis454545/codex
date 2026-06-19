use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;

use crate::models::ContentItem;
use crate::models::FunctionCallOutputContentItem;
use crate::models::LocalImagePreparation;
use crate::models::ResponseInputItem;
use crate::user_input::UserInput;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema, TS)]
pub struct RequestUserMessageResponse {
    pub items: Vec<UserInput>,
    #[serde(default)]
    pub context_action: RequestUserMessageContextAction,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
pub enum RequestUserMessageContextAction {
    #[default]
    Continue,
    Compact,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
pub struct RequestUserMessageEvent {
    /// Responses API call id for the associated tool call, if available.
    pub call_id: String,
    /// Turn ID that this request belongs to.
    pub turn_id: String,
}

pub fn user_input_to_function_output_content(
    items: Vec<UserInput>,
) -> Vec<FunctionCallOutputContentItem> {
    let response_item = ResponseInputItem::from_user_input(items, LocalImagePreparation::Process);
    match response_item {
        ResponseInputItem::Message { content, .. } => content
            .into_iter()
            .filter_map(|item| match item {
                ContentItem::InputText { text } => {
                    Some(FunctionCallOutputContentItem::InputText { text })
                }
                ContentItem::InputImage { image_url, detail } => {
                    Some(FunctionCallOutputContentItem::InputImage { image_url, detail })
                }
                ContentItem::OutputText { .. } => None,
            })
            .collect(),
        ResponseInputItem::FunctionCallOutput { .. }
        | ResponseInputItem::CustomToolCallOutput { .. }
        | ResponseInputItem::ToolSearchOutput { .. }
        | ResponseInputItem::McpToolCallOutput { .. } => Vec::new(),
    }
}
