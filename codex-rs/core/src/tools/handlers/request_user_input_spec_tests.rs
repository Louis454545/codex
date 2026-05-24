use super::*;
use codex_features::Features;
use codex_protocol::config_types::ModeKind;
use codex_tools::request_user_input_available_modes;
use pretty_assertions::assert_eq;

fn default_available_modes() -> Vec<ModeKind> {
    request_user_input_available_modes(&Features::with_defaults())
}

#[test]
fn wait_user_tool_has_empty_schema() {
    assert_eq!(
        create_request_user_input_tool("Wait for the user.".to_string()),
        ToolSpec::Function(ResponsesApiTool {
            name: "wait_user".to_string(),
            description: "Wait for the user.".to_string(),
            strict: false,
            defer_loading: None,
            parameters: JsonSchema::object(
                std::collections::BTreeMap::new(),
                Some(Vec::new()),
                Some(false.into()),
            ),
            output_schema: None,
        })
    );
}

#[test]
fn request_user_input_unavailable_messages_allow_default_mode() {
    assert_eq!(
        request_user_input_unavailable_message(ModeKind::Plan, &default_available_modes()),
        None
    );
    assert_eq!(
        request_user_input_unavailable_message(ModeKind::Default, &default_available_modes()),
        None
    );
    assert_eq!(
        request_user_input_unavailable_message(ModeKind::Execute, &default_available_modes()),
        Some("wait_user is unavailable in Execute mode".to_string())
    );
    assert_eq!(
        request_user_input_unavailable_message(
            ModeKind::PairProgramming,
            &default_available_modes()
        ),
        Some("wait_user is unavailable in Pair Programming mode".to_string())
    );
}

#[test]
fn request_user_input_tool_description_mentions_available_modes() {
    assert_eq!(
        request_user_input_tool_description(&default_available_modes()),
        "Wait indefinitely for the user's next prompt input and return it as this tool's result. Use this as the final tool call only after the current work is otherwise finished, so the agent does not stop without another user message. This tool is only available in Default or Plan mode.".to_string()
    );
}
