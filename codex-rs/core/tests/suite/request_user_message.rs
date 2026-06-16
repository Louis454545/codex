#![allow(clippy::unwrap_used)]

use codex_protocol::models::PermissionProfile;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::Op;
use codex_protocol::request_user_message::RequestUserMessageResponse;
use codex_protocol::user_input::UserInput;
use core_test_support::TempDirExt;
use core_test_support::responses;
use core_test_support::responses::ResponsesRequest;
use core_test_support::responses::ev_assistant_message;
use core_test_support::responses::ev_completed;
use core_test_support::responses::ev_function_call;
use core_test_support::responses::ev_response_created;
use core_test_support::responses::sse;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::local_selections;
use core_test_support::test_codex::test_codex;
use core_test_support::test_codex::turn_permission_fields;
use core_test_support::wait_for_event;
use core_test_support::wait_for_event_match;
use pretty_assertions::assert_eq;
use serde_json::Value;
use serde_json::json;

const REQUEST_USER_MESSAGE: &str = "request_user_message";

fn contains_function_tool(request: &ResponsesRequest, name: &str) -> bool {
    request
        .body_json()
        .get("tools")
        .and_then(Value::as_array)
        .is_some_and(|tools| {
            tools.iter().any(|tool| {
                tool.get("type").and_then(Value::as_str) == Some("function")
                    && tool.get("name").and_then(Value::as_str) == Some(name)
            })
        })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn low_5h_quota_forces_request_user_message_recovery() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;
    let TestCodex {
        codex,
        cwd,
        session_configured,
        ..
    } = test_codex().build(&server).await?;

    codex
        .set_app_server_client_info(
            Some("codex-tui".to_string()),
            Some("test".to_string()),
            /*mcp_elicitations_auto_deny*/ false,
        )
        .await?;

    let call_id = "request-user-message-call";
    let response_mock = responses::mount_sse_sequence_with_headers(
        &server,
        vec![
            (
                sse(vec![
                    ev_response_created("resp-1"),
                    ev_assistant_message("msg-1", "done"),
                    ev_completed("resp-1"),
                ]),
                vec![
                    ("x-codex-primary-used-percent".to_string(), "96".to_string()),
                    (
                        "x-codex-primary-window-minutes".to_string(),
                        "300".to_string(),
                    ),
                    (
                        "x-codex-primary-reset-at".to_string(),
                        "1700000000".to_string(),
                    ),
                ],
            ),
            (
                sse(vec![
                    ev_response_created("resp-2"),
                    ev_function_call(call_id, REQUEST_USER_MESSAGE, "{}"),
                    ev_completed("resp-2"),
                ]),
                Vec::new(),
            ),
            (
                sse(vec![ev_response_created("resp-3"), ev_completed("resp-3")]),
                Vec::new(),
            ),
        ],
    )
    .await;

    let (sandbox_policy, permission_profile) =
        turn_permission_fields(PermissionProfile::Disabled, cwd.path());
    codex
        .submit(Op::UserInput {
            items: vec![UserInput::Text {
                text: "finish this".into(),
                text_elements: Vec::new(),
            }],
            final_output_json_schema: None,
            responsesapi_client_metadata: None,
            additional_context: Default::default(),
            thread_settings: codex_protocol::protocol::ThreadSettingsOverrides {
                environments: Some(local_selections(cwd.abs())),
                approval_policy: Some(AskForApproval::Never),
                sandbox_policy: Some(sandbox_policy),
                permission_profile,
                collaboration_mode: Some(codex_protocol::config_types::CollaborationMode {
                    mode: codex_protocol::config_types::ModeKind::Default,
                    settings: codex_protocol::config_types::Settings {
                        model: session_configured.model.clone(),
                        reasoning_effort: None,
                        developer_instructions: None,
                    },
                }),
                ..Default::default()
            },
        })
        .await?;

    let request = wait_for_event_match(&codex, |event| match event {
        EventMsg::RequestUserMessage(request) => Some(request.clone()),
        _ => None,
    })
    .await;
    assert_eq!(request.call_id, call_id);

    codex
        .submit(Op::UserMessageToolResponse {
            id: request.turn_id,
            response: RequestUserMessageResponse {
                items: vec![UserInput::Text {
                    text: "continue please".to_string(),
                    text_elements: Vec::new(),
                }],
            },
        })
        .await?;

    wait_for_event(&codex, |event| matches!(event, EventMsg::TurnComplete(_))).await;

    let requests = response_mock.requests();
    assert_eq!(requests.len(), 3);
    assert!(!contains_function_tool(&requests[0], REQUEST_USER_MESSAGE));
    assert!(contains_function_tool(&requests[1], REQUEST_USER_MESSAGE));
    assert_eq!(
        requests[1].body_json().get("tool_choice"),
        Some(&json!({
            "type": "function",
            "name": REQUEST_USER_MESSAGE,
        }))
    );
    assert!(
        requests[1]
            .instructions_text()
            .contains("You already produced the assistant response for this turn")
    );

    let (content, success) = requests[2]
        .function_call_output_content_and_success(call_id)
        .expect("request_user_message output should be present");
    assert_eq!(content, Some("continue please".to_string()));
    assert_eq!(success, None);

    Ok(())
}
