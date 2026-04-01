use crate::domain::sync::SyncTrigger;

/// 判断同步 revision 是否发生变化
///
/// # Arguments
/// * `previous` - 上次同步的 revision 时间戳（毫秒）
/// * `current` - 当前获取的 revision 时间戳（毫秒）
///
/// # Returns
/// * 如果 previous 为 None 且 current 有值，返回 true（首次获取）
/// * 如果 current 为 None，返回 false（无法判断）
/// * 如果两者都有值且不同，返回 true
/// * 如果两者都有值且相同，返回 false
pub fn is_revision_changed(previous: Option<i64>, current: Option<i64>) -> bool {
    match (previous, current) {
        (_, None) => false,
        (Some(previous), Some(current)) => previous != current,
        (None, Some(_)) => true,
    }
}

/// 判断是否应跳过 payload 持久化
///
/// 业务规则：
/// 1. 手动同步（Manual trigger）永远不跳过，确保用户操作生效
/// 2. 首次同步（last_sync_at_ms 为 None）不跳过，确保数据初始化
/// 3. 非手动同步时，如果 revision 未变化，跳过持久化（避免重复写入）
///
/// # Arguments
/// * `trigger` - 同步触发方式
/// * `last_sync_at_ms` - 上次成功同步的时间戳
/// * `previous_revision_ms` - 上次同步时的 revision
/// * `current_revision_ms` - 当前获取的 revision
///
/// # Returns
/// true 表示可以跳过 payload 持久化，false 表示需要持久化
pub fn should_skip_payload_persist(
    trigger: SyncTrigger,
    last_sync_at_ms: Option<i64>,
    previous_revision_ms: Option<i64>,
    current_revision_ms: Option<i64>,
) -> bool {
    // 规则 1: 手动同步永远执行完整持久化
    if matches!(trigger, SyncTrigger::Manual) {
        return false;
    }

    // 规则 2: 首次同步必须持久化
    if last_sync_at_ms.is_none() {
        return false;
    }

    // 规则 3: revision 未变化时跳过
    match (previous_revision_ms, current_revision_ms) {
        (Some(previous), Some(current)) => previous == current,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revision_changed_when_previous_none_and_current_some() {
        assert!(is_revision_changed(None, Some(42)));
    }

    #[test]
    fn revision_not_changed_when_current_none() {
        assert!(!is_revision_changed(Some(42), None));
    }

    #[test]
    fn revision_changed_when_values_differ() {
        assert!(is_revision_changed(Some(41), Some(42)));
    }

    #[test]
    fn revision_not_changed_when_values_same() {
        assert!(!is_revision_changed(Some(42), Some(42)));
    }

    #[test]
    fn skip_persist_for_manual_sync_even_if_revision_same() {
        assert!(!should_skip_payload_persist(
            SyncTrigger::Manual,
            Some(1_700_000_000_000),
            Some(42),
            Some(42)
        ));
    }

    #[test]
    fn skip_persist_when_revision_unchanged_and_synced_before() {
        assert!(should_skip_payload_persist(
            SyncTrigger::Poll,
            Some(1_700_000_000_000),
            Some(42),
            Some(42)
        ));
    }

    #[test]
    fn no_skip_persist_on_first_sync() {
        assert!(!should_skip_payload_persist(
            SyncTrigger::Poll,
            None,
            Some(42),
            Some(42)
        ));
    }

    #[test]
    fn no_skip_persist_when_revision_unknown() {
        assert!(!should_skip_payload_persist(
            SyncTrigger::Poll,
            Some(1_700_000_000_000),
            Some(42),
            None
        ));
    }
}
