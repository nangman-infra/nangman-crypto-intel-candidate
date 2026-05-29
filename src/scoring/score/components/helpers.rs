use super::*;

pub(super) fn push_component(
    components: &mut Vec<ScoreComponent>,
    name: impl Into<String>,
    value: i64,
    reason: impl Into<String>,
) {
    if value != 0 {
        components.push(ScoreComponent {
            name: name.into(),
            value,
            reason: reason.into(),
        });
    }
}
