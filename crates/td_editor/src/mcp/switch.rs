//! Whether MCP may run at all. The preferences turn it off until a person turns it back on, [`DISABLE_ENV`] for one
//! run.

use std::ffi::OsStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::state::Prefs;

pub const DISABLE_ENV: &str = "TRENCHDOT_DISABLE_MCP";

/// The [`super::Saved`] setting that is there while MCP is off in Preferences.
pub const OFF_FILE: &str = "mcp-off";

/// What turned MCP off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OffBy {
    Env,
    Prefs,
}

impl OffBy {
    /// Finishes "MCP is turned off ...".
    pub fn place(self) -> &'static str {
        match self {
            OffBy::Env => "by TRENCHDOT_DISABLE_MCP",
            OffBy::Prefs => "in File > Preferences",
        }
    }

    /// The error MCP clients get while MCP is off.
    pub fn refusal(self) -> String {
        match self {
            OffBy::Env => format!("MCP is turned off in TrenchDot by {DISABLE_ENV}"),
            OffBy::Prefs => "MCP is turned off in TrenchDot, File > Preferences turns it back on".into(),
        }
    }
}

/// Set to anything but empty or 0.
pub fn env_disables(value: Option<&OsStr>) -> bool {
    value.map(|v| v.to_string_lossy()).is_some_and(|v| !matches!(v.trim(), "" | "0"))
}

/// The variable wins over the preferences.
pub fn off_by(enabled: bool, env_off: bool) -> Option<OffBy> {
    if env_off {
        Some(OffBy::Env)
    } else if enabled {
        None
    } else {
        Some(OffBy::Prefs)
    }
}

/// What MCP a start brings up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Startup {
    /// stdin and stdout carry MCP. While MCP is off they answer every request with an error, so a client fails at once
    /// instead of waiting.
    pub stdio: bool,
    /// The HTTP server's port. It is kept while MCP is off, so turning MCP on starts the server there.
    pub http: Option<u16>,
    /// No server runs while this is set, whatever the flags ask for.
    pub off: Option<OffBy>,
}

impl Startup {
    /// The HTTP server to start now.
    pub fn serve_http(&self) -> Option<u16> {
        self.http.filter(|_| self.off.is_none())
    }

    /// Why the MCP that was asked for does not run.
    pub fn note(&self) -> Option<String> {
        let by = self.off?;
        (self.stdio || self.http.is_some()).then(|| format!("MCP is turned off {}, so no MCP server starts", by.place()))
    }
}

pub fn startup(stdio: bool, http_flag: Option<u16>, prefs: &Prefs, env: Option<&OsStr>) -> Startup {
    Startup { stdio, http: http_flag.or(prefs.mcp_http.then_some(prefs.mcp_port)), off: off_by(prefs.mcp_enabled, env_disables(env)) }
}

/// Whether MCP is on, shared by the transports and the editor.
#[derive(Clone, Debug, Default)]
pub struct Gate(Arc<AtomicU8>);

impl Gate {
    pub fn new(off: Option<OffBy>) -> Self {
        let gate = Self::default();
        gate.set(off);
        gate
    }

    pub fn off(&self) -> Option<OffBy> {
        match self.0.load(Ordering::SeqCst) {
            0 => None,
            1 => Some(OffBy::Env),
            _ => Some(OffBy::Prefs),
        }
    }

    pub fn set(&self, off: Option<OffBy>) {
        let value = match off {
            None => 0,
            Some(OffBy::Env) => 1,
            Some(OffBy::Prefs) => 2,
        };
        self.0.store(value, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prefs(enabled: bool, http: bool) -> Prefs {
        Prefs { mcp_enabled: enabled, mcp_http: http, mcp_port: 7900, ..Prefs::default() }
    }

    fn env(v: &str) -> Option<&OsStr> {
        Some(OsStr::new(v))
    }

    #[test]
    fn the_variable_counts_unless_empty_or_zero() {
        assert!(!env_disables(None));
        for off in ["", " ", "0", " 0 "] {
            assert!(!env_disables(env(off)), "{off:?}");
        }

        for on in ["1", "yes", "true", "false", "00"] {
            assert!(env_disables(env(on)), "{on:?}");
        }
    }

    #[test]
    fn mcp_is_on_unless_something_turns_it_off() {
        assert!(Prefs::default().mcp_enabled);
        let plain = startup(false, None, &Prefs::default(), None);
        assert_eq!(plain, Startup { stdio: false, http: None, off: None });
        assert_eq!(plain.note(), None);

        let both = startup(true, Some(9000), &prefs(true, true), env("0"));
        assert_eq!((both.stdio, both.http, both.off, both.serve_http()), (true, Some(9000), None, Some(9000)));
        assert_eq!(startup(false, None, &prefs(true, true), None).serve_http(), Some(7900));
        assert_eq!(startup(false, None, &prefs(true, false), None).serve_http(), None);
    }

    #[test]
    fn off_wins_over_the_flags() {
        let by_env = startup(true, Some(9000), &prefs(true, false), env("1"));
        assert_eq!((by_env.off, by_env.serve_http(), by_env.http), (Some(OffBy::Env), None, Some(9000)));
        assert!(by_env.note().unwrap().contains(DISABLE_ENV));

        let by_prefs = startup(false, None, &prefs(false, true), None);
        assert_eq!((by_prefs.off, by_prefs.serve_http(), by_prefs.http), (Some(OffBy::Prefs), None, Some(7900)));
        assert!(by_prefs.note().unwrap().contains("File > Preferences"));

        assert_eq!(startup(true, None, &prefs(false, false), env("1")).off, Some(OffBy::Env), "the variable names the reason");
        assert_eq!(startup(false, None, &prefs(false, false), None).note(), None, "nothing asked for MCP");
    }

    #[test]
    fn the_gate_is_shared() {
        let gate = Gate::new(Some(OffBy::Prefs));
        let copy = gate.clone();
        assert_eq!(copy.off(), Some(OffBy::Prefs));
        gate.set(None);
        assert_eq!(copy.off(), None);
        gate.set(Some(OffBy::Env));
        assert_eq!(copy.off(), Some(OffBy::Env));
        assert!(OffBy::Env.refusal().contains("turned off in TrenchDot"));
    }
}
