use std::{
    env,
    io::{self, IsTerminal},
};

use super::OutputFormat;

/// Product-layer terminal presentation policy.
///
/// This policy is intentionally presentation-only: it never changes Harness semantics,
/// provider/model selection, JSON contracts, or persisted session/evidence data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TerminalPresentation {
    human: bool,
    full_tty: bool,
    plain: bool,
}

impl TerminalPresentation {
    pub(super) fn detect(format: OutputFormat, explicit_plain: bool) -> Self {
        Self::from_inputs(
            format,
            explicit_plain,
            io::stdin().is_terminal(),
            io::stdout().is_terminal(),
            io::stderr().is_terminal(),
            env::var_os("NO_COLOR").is_some(),
            env::var("TERM").ok().as_deref(),
        )
    }

    pub(super) fn from_inputs(
        format: OutputFormat,
        explicit_plain: bool,
        stdin_is_terminal: bool,
        stdout_is_terminal: bool,
        stderr_is_terminal: bool,
        no_color: bool,
        term: Option<&str>,
    ) -> Self {
        let human = format == OutputFormat::Human;
        let full_tty = stdin_is_terminal && stdout_is_terminal && stderr_is_terminal;
        let dumb_terminal = term.is_some_and(|value| value.trim().eq_ignore_ascii_case("dumb"));
        let plain = explicit_plain || no_color || dumb_terminal || !human || !full_tty;
        Self {
            human,
            full_tty,
            plain,
        }
    }

    pub(super) fn is_plain(self) -> bool {
        self.plain
    }

    pub(super) fn progress_allowed(self) -> bool {
        self.human && self.full_tty && !self.plain
    }

    pub(super) fn decoration_allowed(self) -> bool {
        self.human && self.full_tty && !self.plain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn human_tty(explicit_plain: bool, no_color: bool, term: Option<&str>) -> TerminalPresentation {
        TerminalPresentation::from_inputs(
            OutputFormat::Human,
            explicit_plain,
            true,
            true,
            true,
            no_color,
            term,
        )
    }

    #[test]
    fn explicit_plain_no_color_and_dumb_terminal_select_plain_mode() {
        assert!(human_tty(true, false, Some("xterm-256color")).is_plain());
        assert!(human_tty(false, true, Some("xterm-256color")).is_plain());
        assert!(human_tty(false, false, Some("dumb")).is_plain());
        assert!(human_tty(false, false, Some(" DUMB ")).is_plain());
    }

    #[test]
    fn ordinary_human_full_tty_keeps_static_progress_available() {
        let policy = human_tty(false, false, Some("xterm-256color"));
        assert!(!policy.is_plain());
        assert!(policy.progress_allowed());
        assert!(policy.decoration_allowed());
    }

    #[test]
    fn json_and_redirected_streams_are_plain_and_decoration_free() {
        let json = TerminalPresentation::from_inputs(
            OutputFormat::Json,
            false,
            true,
            true,
            true,
            false,
            Some("xterm-256color"),
        );
        assert!(json.is_plain());
        assert!(!json.progress_allowed());
        assert!(!json.decoration_allowed());

        let redirected = TerminalPresentation::from_inputs(
            OutputFormat::Human,
            false,
            true,
            false,
            true,
            false,
            Some("xterm-256color"),
        );
        assert!(redirected.is_plain());
        assert!(!redirected.progress_allowed());
        assert!(!redirected.decoration_allowed());
    }
}
