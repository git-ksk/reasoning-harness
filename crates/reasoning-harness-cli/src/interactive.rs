use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Directive {
    Prompt(String),
    Add(PathBuf),
    Files,
    Clear,
    Help,
    Exit,
}

fn parse_directive(input: &str) -> Result<Option<Directive>, CliError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if !trimmed.starts_with('/') {
        return Ok(Some(Directive::Prompt(trimmed.to_string())));
    }

    let (command, argument) = trimmed
        .split_once(char::is_whitespace)
        .map(|(command, argument)| (command, argument.trim()))
        .unwrap_or((trimmed, ""));
    match command {
        "/exit" | "/quit" if argument.is_empty() => Ok(Some(Directive::Exit)),
        "/help" if argument.is_empty() => Ok(Some(Directive::Help)),
        "/files" if argument.is_empty() => Ok(Some(Directive::Files)),
        "/clear" if argument.is_empty() => Ok(Some(Directive::Clear)),
        "/add" => {
            if argument.is_empty() {
                return Err(CliError::new("input", "/add requires a file path"));
            }
            let path = if argument.len() >= 2
                && ((argument.starts_with('"') && argument.ends_with('"'))
                    || (argument.starts_with('\'') && argument.ends_with('\'')))
            {
                &argument[1..argument.len() - 1]
            } else {
                argument
            };
            if path.trim().is_empty() {
                return Err(CliError::new(
                    "input",
                    "/add requires a non-empty file path",
                ));
            }
            Ok(Some(Directive::Add(PathBuf::from(path))))
        }
        _ => Err(CliError::new(
            "input",
            format!("unknown interactive command {command:?}; use /help"),
        )),
    }
}

fn effective_output_format(args: &NaturalArgs) -> Result<OutputFormat, CliError> {
    if let Some(format) = args.format {
        return Ok(format);
    }
    if args.no_config {
        return Ok(OutputFormat::Human);
    }
    if args.config.as_ref().is_some_and(|path| is_stdin(path)) {
        return Err(CliError::new(
            "configuration",
            "--config must be a file path; stdin config is not supported",
        ));
    }
    let loaded = load_cli_config(args.config.as_ref())
        .map_err(|error| CliError::new("configuration", error))?;
    Ok(loaded.config.run.format.unwrap_or_default())
}

pub(super) fn should_start(args: &NaturalArgs, stdin_is_terminal: bool) -> Result<bool, CliError> {
    if args
        .task
        .as_deref()
        .is_some_and(|task| !task.trim().is_empty())
        || !stdin_is_terminal
    {
        return Ok(false);
    }
    Ok(effective_output_format(args)? == OutputFormat::Human)
}

fn read_entry() -> Result<Option<String>, CliError> {
    let mut entry = String::new();
    let mut continuation = false;
    loop {
        if continuation {
            print!("... ");
        } else {
            print!("reason> ");
        }
        io::stdout()
            .flush()
            .map_err(|error| CliError::new("interactive_io", format!("flush prompt: {error}")))?;

        let mut line = String::new();
        let read = io::stdin()
            .read_line(&mut line)
            .map_err(|error| CliError::new("interactive_io", format!("read prompt: {error}")))?;
        if read == 0 {
            return if entry.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some(entry))
            };
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if let Some(prefix) = line.strip_suffix('\\') {
            entry.push_str(prefix);
            entry.push('\n');
            continuation = true;
            continue;
        }
        entry.push_str(line);
        return Ok(Some(entry));
    }
}

fn pick_session() -> Result<managed_session::ManagedSession, CliError> {
    let list = managed_session::list()?;
    if !list.problems.is_empty() {
        println!("Unavailable managed sessions:");
        for problem in &list.problems {
            println!("  ! {}  {}", problem.file_name, problem.state);
        }
    }
    if list.sessions.is_empty() {
        return Err(CliError::new(
            "session_not_found",
            "no compatible managed sessions exist; start `reason` or inspect `reason session list`",
        ));
    }
    println!("Choose a managed session:");
    for (index, session) in list.sessions.iter().enumerate() {
        println!(
            "  {}) {}  {}  ({} turn{})",
            index + 1,
            session.short_id,
            session.title,
            session.turns,
            if session.turns == 1 { "" } else { "s" }
        );
    }
    print!("session> ");
    io::stdout()
        .flush()
        .map_err(|error| CliError::new("interactive_io", format!("flush picker: {error}")))?;
    let mut line = String::new();
    let read = io::stdin()
        .read_line(&mut line)
        .map_err(|error| CliError::new("interactive_io", format!("read picker: {error}")))?;
    if read == 0 {
        return Err(CliError::new("cancelled", "session picker closed at EOF"));
    }
    let index = line.trim().parse::<usize>().map_err(|_| {
        CliError::new(
            "input",
            "session picker expects the displayed numeric choice",
        )
    })?;
    if index == 0 {
        return Err(CliError::new("input", "session picker choices start at 1"));
    }
    let summary = list.sessions.get(index - 1).ok_or_else(|| {
        CliError::new(
            "input",
            "session picker choice is outside the displayed range",
        )
    })?;
    managed_session::load_by_selector(&summary.id)
}

fn should_persist_managed(ephemeral: bool, turns_len: usize) -> bool {
    !ephemeral && turns_len > 0
}

fn pin_runtime_from_session(
    base: &mut NaturalArgs,
    session: &managed_session::ManagedSession,
    reject_explicit_drift: bool,
) -> Result<(), CliError> {
    let Some(runtime) = session.last_runtime() else {
        return Ok(());
    };
    if reject_explicit_drift {
        if base
            .provider
            .is_some_and(|provider| provider != runtime.provider)
        {
            return Err(CliError::new(
                "session_incompatible",
                "requested provider differs from the persisted managed-session provider; start a new session instead",
            ));
        }
        if base
            .model
            .as_deref()
            .is_some_and(|model| model != runtime.model)
        {
            return Err(CliError::new(
                "session_incompatible",
                "requested model differs from the persisted managed-session model; start a new session instead",
            ));
        }
        if base
            .max_tokens
            .is_some_and(|max_tokens| max_tokens != runtime.max_tokens)
        {
            return Err(CliError::new(
                "session_incompatible",
                "requested max-token setting differs from the persisted managed session; start a new session instead",
            ));
        }
    }
    base.provider = Some(runtime.provider);
    base.model = Some(runtime.model.clone());
    base.max_tokens = Some(runtime.max_tokens);
    base.safety_profile = runtime.safety_profile;
    Ok(())
}

pub(super) async fn run(
    mut base: NaturalArgs,
    continue_session: bool,
    resume: Option<String>,
) -> Result<(), CliError> {
    let project = managed_session::current_project_key()?;
    let ephemeral = base.ephemeral;
    let resuming = continue_session || resume.is_some();
    let mut session = if continue_session {
        managed_session::load_latest_for_project(&project)?
    } else if let Some(selector) = resume {
        if selector.is_empty() {
            pick_session()?
        } else {
            managed_session::load_by_selector(&selector)?
        }
    } else {
        managed_session::ManagedSession::new(project)
    };

    if resuming {
        pin_runtime_from_session(&mut base, &session, true)?;
    }
    for path in std::mem::take(&mut base.file) {
        session.add_context_file(&path)?;
    }
    if resuming && should_persist_managed(ephemeral, session.turns_len()) {
        managed_session::save(&mut session)?;
    }
    base.interactive_context = session.conversation_context();
    base.task = None;
    base.format = Some(OutputFormat::Human);

    if resuming {
        println!(
            "Reason interactive — resumed {} ({}, {} turn{}).",
            session.short_id(),
            session.title(),
            session.turns_len(),
            if session.turns_len() == 1 { "" } else { "s" }
        );
    } else {
        println!("Reason interactive — Harness-verified answers. /help for commands.");
        if ephemeral {
            println!(
                "Ephemeral mode: prompts, context snapshots, and turns are kept in memory only and are not persisted by Reason."
            );
        } else {
            println!("A managed session is saved after the first successful verified turn.");
        }
    }

    loop {
        let Some(entry) = read_entry()? else {
            println!();
            return Ok(());
        };
        let directive = match parse_directive(&entry) {
            Ok(Some(directive)) => directive,
            Ok(None) => continue,
            Err(error) => {
                eprintln!("{}", error.message);
                continue;
            }
        };

        match directive {
            Directive::Exit => return Ok(()),
            Directive::Help => {
                println!(
                    "/add <path>  add an untrusted context snapshot ({})",
                    if ephemeral {
                        "memory-only"
                    } else {
                        "persisted"
                    }
                );
                println!("/files       list context snapshots active in this session");
                println!(
                    "/clear       stop carrying prior conversation/context into later prompts"
                );
                println!("/help        show this help");
                println!("/exit        exit Reason interactive");
                println!("End a line with \\ to continue a multiline prompt.");
            }
            Directive::Files => {
                let sources = session.context_sources().collect::<Vec<_>>();
                if sources.is_empty() {
                    println!("No context files added.");
                } else {
                    for source in sources {
                        println!("{source}");
                    }
                }
            }
            Directive::Clear => {
                session.clear_context();
                base.interactive_context = session.conversation_context();
                if should_persist_managed(ephemeral, session.turns_len()) {
                    managed_session::save(&mut session)?;
                }
                println!(
                    "Interactive carry-over context cleared; typed prior turns remain in history."
                );
            }
            Directive::Add(path) => match session.add_context_file(&path) {
                Ok(()) => {
                    base.interactive_context = session.conversation_context();
                    if should_persist_managed(ephemeral, session.turns_len()) {
                        managed_session::save(&mut session)?;
                    }
                    println!("Added untrusted context snapshot: {}", path.display());
                }
                Err(error) => eprintln!("{}", error.message),
            },
            Directive::Prompt(task) => {
                let mut args = base.clone();
                args.task = Some(task);
                let safety_profile = args.safety_profile;
                match execute_natural(args, None).await {
                    Ok(output) => {
                        print_natural_human(&output);
                        session.append_turn(&output, safety_profile)?;
                        if should_persist_managed(ephemeral, session.turns_len()) {
                            managed_session::save(&mut session)?;
                        }
                        pin_runtime_from_session(&mut base, &session, false)?;
                        base.interactive_context = session.conversation_context();
                        if !ephemeral {
                            println!("session: {}", session.short_id());
                        }
                    }
                    Err(error) => eprintln!("{}", error.message),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_human_tty_enters_interactive_mode() {
        let args = NaturalArgs {
            no_config: true,
            ..Default::default()
        };
        assert!(should_start(&args, true).unwrap());
    }

    #[test]
    fn task_pipe_and_json_never_auto_enter_interactive_mode() {
        let one_shot = NaturalArgs {
            task: Some("check this".into()),
            no_config: true,
            ..Default::default()
        };
        assert!(!should_start(&one_shot, true).unwrap());

        let piped = NaturalArgs {
            no_config: true,
            ..Default::default()
        };
        assert!(!should_start(&piped, false).unwrap());

        let json = NaturalArgs {
            format: Some(OutputFormat::Json),
            no_config: true,
            ..Default::default()
        };
        assert!(!should_start(&json, true).unwrap());
    }

    #[test]
    fn conversation_memory_is_untrusted_context_without_fact_authority() {
        let args = NaturalArgs {
            no_config: true,
            interactive_context: vec![
                "Prior interactive exchange: User: where? Reason: us-east-1".into(),
            ],
            ..Default::default()
        };
        let built = build_natural_input(&args, "verify it again").unwrap();
        let memory = built
            .input
            .evidence
            .iter()
            .find(|evidence| evidence.source == "interactive-session")
            .expect("interactive memory evidence");
        assert!(memory.facts.is_empty());
        assert_eq!(
            memory.metadata.provenance_class.as_deref(),
            Some("untrusted_context")
        );
    }

    #[test]
    fn ephemeral_mode_never_persists_managed_state_even_after_turns_exist() {
        assert!(!should_persist_managed(true, 1));
        assert!(!should_persist_managed(true, 99));
        assert!(!should_persist_managed(false, 0));
        assert!(should_persist_managed(false, 1));
    }

    #[test]
    fn parses_interactive_commands_without_shell_interpretation() {
        assert_eq!(
            parse_directive("/add '/tmp/context with spaces.txt'").unwrap(),
            Some(Directive::Add(PathBuf::from(
                "/tmp/context with spaces.txt"
            )))
        );
        assert_eq!(
            parse_directive("follow up on that").unwrap(),
            Some(Directive::Prompt("follow up on that".into()))
        );
        assert_eq!(parse_directive("/exit").unwrap(), Some(Directive::Exit));
        assert!(parse_directive("/unknown").is_err());
    }
}
