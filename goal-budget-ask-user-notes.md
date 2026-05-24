# Goal Budget and Low Usage Ask-User Behavior

## Intent

This change is meant to keep long-running `/goal` sessions alive even after the
configured goal token budget has been reached.

Previously, reaching the goal budget could stop the goal flow by marking the
goal as budget-limited and steering the model to wrap up. The desired behavior is
different: the budget should remain visible as guidance, but it should not force
the active goal to stop. The model can continue working past the budget unless it
explicitly marks the goal complete or blocked for a real reason.

## Requested behavior

- Reaching the `/goal` token budget must not automatically stop the goal.
- Goal usage accounting should continue after the token budget is reached.
- When the goal budget is reached, the model should be instructed to use
  `request_user_input` before it would otherwise finish the turn, so the user can
  continue the same turn from the prompt input.
- When the five-hour usage window has less than 5% remaining, the runtime should
  inject steering that tells the model to call `request_user_input` instead of
  ending with a final answer.
- The TUI should show an indication when the five-hour limit has entered this
  ask-user mode.

## Why

The goal is to avoid losing continuity at the exact moment usage is nearly
exhausted. Instead of ending the model turn and requiring the user to send a new
message, Codex should request user input through the ask-user tool and continue
from that tool response inside the same turn.
