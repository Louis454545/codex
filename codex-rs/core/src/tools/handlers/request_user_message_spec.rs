use codex_tools::JsonSchema;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolSpec;
use std::collections::BTreeMap;

pub const REQUEST_USER_MESSAGE_TOOL_NAME: &str = "request_user_message";

pub fn create_request_user_message_tool() -> ToolSpec {
    ToolSpec::Function(ResponsesApiTool {
        name: REQUEST_USER_MESSAGE_TOOL_NAME.to_string(),
        description: "Request the user's next normal Codex composer message and wait for it. This tool takes no arguments. The user can respond with text and supported attachments such as images.".to_string(),
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(BTreeMap::new(), Some(Vec::new()), Some(false.into())),
        output_schema: None,
    })
}
