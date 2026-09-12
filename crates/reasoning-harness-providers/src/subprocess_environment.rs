use std::{env, ffi::OsString, process::Command};

const COMMON_BASELINE_ENV: &[&str] = &[
    "PATH",
    "TMPDIR",
    "TMP",
    "TEMP",
    "LANG",
    "LANGUAGE",
    "LC_ALL",
    "LC_CTYPE",
    "LC_MESSAGES",
    "TZ",
];

#[cfg(windows)]
const WINDOWS_BASELINE_ENV: &[&str] =
    &["SystemRoot", "WINDIR", "ComSpec", "PATHEXT", "SystemDrive"];

/// Replaces ambient environment inheritance with Reason's minimal subprocess baseline.
///
/// This is an OS-process secrecy boundary only. It does not change resolver/MCP/verifier
/// semantics or promote child output to authority. Integration-specific credentials must be
/// injected explicitly through `isolate_subprocess_environment_with`; they are never recovered
/// from project config or broad parent-environment inheritance here.
pub(crate) fn isolate_subprocess_environment(command: &mut Command) {
    isolate_subprocess_environment_with(command, std::iter::empty::<(OsString, OsString)>());
}

/// Applies the minimal baseline plus explicitly scoped environment values for one selected
/// integration. The caller must obtain scoped secrets from a secure product-owned source; this
/// function deliberately has no ambient-variable lookup by arbitrary name.
pub(crate) fn isolate_subprocess_environment_with<I>(command: &mut Command, scoped: I)
where
    I: IntoIterator<Item = (OsString, OsString)>,
{
    command.env_clear();
    preserve_parent_values(command, COMMON_BASELINE_ENV);
    #[cfg(windows)]
    preserve_parent_values(command, WINDOWS_BASELINE_ENV);
    command.envs(scoped);
}

fn preserve_parent_values(command: &mut Command, keys: &[&str]) {
    for key in keys {
        if let Some(value) = env::var_os(key) {
            command.env(key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENTINEL_KEY: &str = "REASON_SUBPROCESS_SENTINEL_SECRET";
    const SENTINEL_VALUE: &str = "must-not-leak-to-child";

    fn output_environment(command: &mut Command) -> String {
        let output = command.output().expect("environment probe child");
        assert!(output.status.success());
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    #[cfg(unix)]
    fn environment_probe() -> Command {
        Command::new("/usr/bin/env")
    }

    #[cfg(windows)]
    fn environment_probe() -> Command {
        let comspec = env::var_os("ComSpec").unwrap_or_else(|| OsString::from("cmd.exe"));
        let mut command = Command::new(comspec);
        command.args(["/D", "/C", "set"]);
        command
    }

    fn value_for<'a>(dump: &'a str, key: &str) -> Option<&'a str> {
        dump.lines().find_map(|line| {
            let (found, value) = line.split_once('=')?;
            if found.eq_ignore_ascii_case(key) {
                Some(value)
            } else {
                None
            }
        })
    }

    #[test]
    fn child_environment_strips_ambient_sentinel_and_preserves_launch_baseline() {
        let ambient_sentinel = env::var_os(SENTINEL_KEY).is_some();
        let expected_path = env::var_os("PATH").map(|value| value.to_string_lossy().into_owned());
        let mut command = environment_probe();
        if !ambient_sentinel {
            // Full local test suites need not mutate the process-global environment. The
            // cross-platform CI gate supplies the same key in the parent environment.
            command.env(SENTINEL_KEY, SENTINEL_VALUE);
        }
        isolate_subprocess_environment(&mut command);
        let dump = output_environment(&mut command);

        assert!(value_for(&dump, SENTINEL_KEY).is_none());
        if let Some(expected_path) = expected_path {
            assert_eq!(value_for(&dump, "PATH"), Some(expected_path.as_str()));
        }

        #[cfg(windows)]
        for key in ["SystemRoot", "WINDIR", "ComSpec", "PATHEXT", "SystemDrive"] {
            if let Some(expected) = env::var_os(key) {
                let expected = expected.to_string_lossy();
                assert_eq!(value_for(&dump, key), Some(expected.as_ref()), "{key}");
            }
        }
    }

    #[test]
    fn scoped_environment_is_explicit_and_does_not_restore_ambient_secrets() {
        let mut command = environment_probe();
        command.env(SENTINEL_KEY, SENTINEL_VALUE);
        isolate_subprocess_environment_with(
            &mut command,
            [(
                OsString::from("REASON_SCOPED_INTEGRATION_TOKEN"),
                OsString::from("scoped-only"),
            )],
        );
        let dump = output_environment(&mut command);

        assert!(value_for(&dump, SENTINEL_KEY).is_none());
        assert_eq!(
            value_for(&dump, "REASON_SCOPED_INTEGRATION_TOKEN"),
            Some("scoped-only")
        );
    }
}
