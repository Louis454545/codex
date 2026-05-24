use crate::context::RateLimitContext;

use codex_protocol::models::ResponseInputItem;
use codex_protocol::protocol::RateLimitSnapshot;
use codex_protocol::protocol::RateLimitWindow;

const WAIT_USER_REMAINING_THRESHOLD_PERCENT: f64 = 5.0;
const FIVE_HOURS_MINUTES: i64 = 5 * 60;

pub(crate) fn five_hour_wait_user_steering_item(
    snapshot: &RateLimitSnapshot,
) -> Option<ResponseInputItem> {
    let window = five_hour_window(snapshot)?;
    let remaining_percent = remaining_percent(window);
    if remaining_percent >= WAIT_USER_REMAINING_THRESHOLD_PERCENT {
        return None;
    }

    Some(
        RateLimitContext::new(five_hour_wait_user_prompt(remaining_percent))
            .into_response_input_item(),
    )
}

fn five_hour_window(snapshot: &RateLimitSnapshot) -> Option<&RateLimitWindow> {
    snapshot
        .primary
        .as_ref()
        .filter(|window| is_five_hour_window(window))
        .or_else(|| {
            snapshot
                .secondary
                .as_ref()
                .filter(|window| is_five_hour_window(window))
        })
        .or_else(|| {
            snapshot
                .primary
                .as_ref()
                .filter(|window| !is_weekly_window(window))
        })
        .or_else(|| {
            snapshot
                .secondary
                .as_ref()
                .filter(|window| !is_weekly_window(window))
        })
}

fn is_five_hour_window(window: &RateLimitWindow) -> bool {
    window
        .window_minutes
        .is_some_and(|minutes| is_approximate_window(minutes, FIVE_HOURS_MINUTES))
}

fn is_weekly_window(window: &RateLimitWindow) -> bool {
    const WEEKLY_MINUTES: i64 = 7 * 24 * 60;

    window
        .window_minutes
        .is_some_and(|minutes| is_approximate_window(minutes, WEEKLY_MINUTES))
}

fn is_approximate_window(minutes: i64, expected_minutes: i64) -> bool {
    let minutes = minutes.max(0) as f64;
    let expected_minutes = expected_minutes as f64;
    minutes >= expected_minutes * 0.95 && minutes <= expected_minutes * 1.05
}

fn remaining_percent(window: &RateLimitWindow) -> f64 {
    (100.0 - window.used_percent).clamp(0.0, 100.0)
}

fn five_hour_wait_user_prompt(remaining_percent: f64) -> String {
    format!(
        "The 5-hour usage window has {remaining_percent:.1}% remaining, which is below the 5% threshold.\n\n\
Do not finish the turn with a final answer while this threshold is active. After your current tool-call sequence and work for this step are otherwise finished, call wait_user as the final tool call. The user's next prompt input will be returned as the wait_user tool result; continue from that tool result in the same turn."
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::models::ContentItem;
    use codex_protocol::protocol::RateLimitWindow;
    use pretty_assertions::assert_eq;

    #[test]
    fn creates_steering_item_when_five_hour_remaining_is_below_threshold() {
        let snapshot = RateLimitSnapshot {
            limit_id: Some("codex".to_string()),
            limit_name: Some("codex".to_string()),
            primary: Some(window(
                /*used_percent*/ 96.0,
                /*window_minutes*/ Some(300),
            )),
            secondary: None,
            credits: None,
            plan_type: None,
            rate_limit_reached_type: None,
        };

        let Some(ResponseInputItem::Message { content, .. }) =
            five_hour_wait_user_steering_item(&snapshot)
        else {
            panic!("expected steering item");
        };
        let [ContentItem::InputText { text }] = content.as_slice() else {
            panic!("expected text content");
        };

        assert!(text.contains("4.0% remaining"));
        assert!(text.contains("wait_user"));
        assert!(text.contains("Do not finish the turn"));
    }

    #[test]
    fn skips_steering_item_at_threshold() {
        let snapshot = RateLimitSnapshot {
            limit_id: Some("codex".to_string()),
            limit_name: Some("codex".to_string()),
            primary: Some(window(
                /*used_percent*/ 95.0,
                /*window_minutes*/ Some(300),
            )),
            secondary: None,
            credits: None,
            plan_type: None,
            rate_limit_reached_type: None,
        };

        assert_eq!(None, five_hour_wait_user_steering_item(&snapshot));
    }

    fn window(used_percent: f64, window_minutes: Option<i64>) -> RateLimitWindow {
        RateLimitWindow {
            used_percent,
            window_minutes,
            resets_at: None,
        }
    }
}
