//! The names TrenchDot went by as GodotTrench, so old installs, settings and projects keep working.

use std::ffi::OsString;

pub const ENV_PREFIX: &str = "GODOTTRENCH_";

/// The environment variable `name` (a `TRENCHDOT_` one), or the same variable under its old prefix. A blank value counts
/// as unset.
pub fn env_var_os(name: &str) -> Option<OsString> {
    let set = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
    set(name).or_else(|| set(&format!("{ENV_PREFIX}{}", name.strip_prefix("TRENCHDOT_")?)))
}

pub fn env_var(name: &str) -> Option<String> {
    env_var_os(name)?.into_string().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Each test has a variable of its own, they run in parallel.
    #[test]
    fn the_old_variable_answers_when_the_new_one_is_not_set() {
        let (new, old) = ("TRENCHDOT_TEST_LEGACY_A", format!("{ENV_PREFIX}TEST_LEGACY_A"));
        assert_eq!(env_var(new), None);
        unsafe { std::env::set_var(&old, "from the old name") };
        assert_eq!(env_var(new).as_deref(), Some("from the old name"));
        unsafe { std::env::set_var(new, "from the new name") };
        assert_eq!(env_var(new).as_deref(), Some("from the new name"), "the new name wins");
        unsafe { std::env::set_var(new, "") };
        assert_eq!(env_var(new).as_deref(), Some("from the old name"), "a blank one counts as unset");
        unsafe { std::env::remove_var(&old) };
        assert_eq!(env_var_os(new), None);
    }

    #[test]
    fn only_trenchdot_variables_have_an_old_name() {
        unsafe { std::env::set_var("PATH_LEGACY_TEST", "x") };
        assert_eq!(env_var("PATH_LEGACY_TEST").as_deref(), Some("x"));
        assert_eq!(env_var("OTHER_LEGACY_TEST_B"), None);
    }

    #[test]
    fn an_old_kill_switch_still_turns_mcp_off() {
        let old = format!("{ENV_PREFIX}DISABLE_MCP");
        unsafe { std::env::set_var(&old, "1") };
        let value = env_var_os(crate::mcp::switch::DISABLE_ENV);
        unsafe { std::env::remove_var(&old) };
        assert!(crate::mcp::switch::env_disables(value.as_deref()));
    }
}
