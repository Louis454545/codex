use crate::function_tool::FunctionCallError;
use crate::tools::context::FunctionToolOutput;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
use crate::tools::handlers::request_user_message_spec::REQUEST_USER_MESSAGE_TOOL_NAME;
use crate::tools::handlers::request_user_message_spec::create_request_user_message_tool;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
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

        let response = session
            .request_user_message(turn.as_ref(), call_id)
            .await
            .ok_or_else(|| {
                FunctionCallError::RespondToModel(format!(
                    "{REQUEST_USER_MESSAGE_TOOL_NAME} was cancelled before receiving a response"
                ))
            })?;
        let content = user_input_to_function_output_content(response.items);

        Ok(boxed_tool_output(FunctionToolOutput::from_content(
            content,
            Some(true),
        )))
    }
}

impl CoreToolRuntime for RequestUserMessageHandler {}
