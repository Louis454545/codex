use crate::function_tool::FunctionCallError;
use crate::session::TurnInput;
use crate::session::turn::build_skills_and_plugins;
use crate::tools::context::FunctionToolOutput;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
use crate::tools::handlers::request_user_message_spec::REQUEST_USER_MESSAGE_TOOL_NAME;
use crate::tools::handlers::request_user_message_spec::create_request_user_message_tool;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use codex_protocol::models::FunctionCallOutputContentItem;
use codex_protocol::request_user_message::user_input_to_function_output_content;
use codex_tools::ToolName;
use codex_tools::ToolSpec;

pub struct RequestUserMessageHandler;

impl ToolExecutor<ToolInvocation> for RequestUserMessageHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(REQUEST_USER_MESSAGE_TOOL_NAME)
    }

    fn spec(&self) -> ToolSpec {
        create_request_user_message_tool()
    }

    fn handle(&self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'_> {
        Box::pin(self.handle_call(invocation))
    }
}

impl RequestUserMessageHandler {
    async fn handle_call(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn crate::tools::context::ToolOutput>, FunctionCallError> {
        let ToolInvocation {
            session,
            turn,
            call_id,
            payload,
            cancellation_token,
            ..
        } = invocation;

        match payload {
            ToolPayload::Function { .. } => {}
            _ => {
                return Err(FunctionCallError::RespondToModel(format!(
                    "{REQUEST_USER_MESSAGE_TOOL_NAME} handler received unsupported payload"
                )));
            }
        }

        if turn.session_source.is_non_root_agent() {
            return Err(FunctionCallError::RespondToModel(
                "request_user_message can only be used by the root thread".to_string(),
            ));
        }

        if session
            .services
            .extensions
            .suppress_request_user_message(
                &session.services.session_extension_data,
                &session.services.thread_extension_data,
            )
            .await
        {
            return Ok(boxed_tool_output(FunctionToolOutput::from_content(
                vec![FunctionCallOutputContentItem::InputText {
                    text: "An active goal owns automatic continuation; continue working on it."
                        .to_string(),
                }],
                Some(true),
            )));
        }

        let response = session
            .request_user_message(turn.as_ref(), call_id)
            .await
            .ok_or_else(|| {
                FunctionCallError::RespondToModel(format!(
                    "{REQUEST_USER_MESSAGE_TOOL_NAME} was cancelled before receiving a response"
                ))
            })?;
        let turn_state = session
            .input_queue
            .turn_state_for_sub_id(&session.active_turn, &turn.sub_id)
            .await;
        if let Some(turn_state) = turn_state {
            turn_state
                .lock()
                .await
                .set_request_user_message_context_action(response.context_action);
        }
        let input = TurnInput::UserInput {
            content: response.items.clone(),
            client_id: None,
        };
        if let Some((injection_items, explicitly_enabled_connectors)) = build_skills_and_plugins(
            &session,
            turn.as_ref(),
            std::slice::from_ref(&input),
            &cancellation_token,
        )
        .await
        {
            session
                .merge_connector_selection(explicitly_enabled_connectors)
                .await;
            for item in injection_items {
                session
                    .record_conversation_items(&turn, std::slice::from_ref(&item))
                    .await;
            }
        }
        let content = user_input_to_function_output_content(response.items);

        Ok(boxed_tool_output(FunctionToolOutput::from_content(
            content,
            Some(true),
        )))
    }
}

impl CoreToolRuntime for RequestUserMessageHandler {}
