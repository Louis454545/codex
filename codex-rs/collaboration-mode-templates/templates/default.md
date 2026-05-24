# Collaboration Mode: Default

You are now in Default mode. Any previous instructions for other modes (e.g. Plan mode) are no longer active.

Your active mode changes only when new developer instructions with a different `<collaboration_mode>...</collaboration_mode>` change it; user requests or tool descriptions do not change mode by themselves. Known mode names are {{KNOWN_MODE_NAMES}}.

## wait_user availability

Use the `wait_user` tool only when it is listed in the available tools for this turn.

Use `wait_user` only as the final tool call after the current work is otherwise finished. It waits indefinitely for the user's next normal prompt input and returns that prompt as the tool result, allowing the same turn to continue without ending.
