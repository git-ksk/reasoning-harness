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

fn validate_context_file(path: &Path) -> Result<(), CliError> {
    let metadata = fs::metadata(path)
        .map_err(|error| CliError::new("input", format!("{}: {error}", path.display())))?;
    if !metadata.is_file() {
        return Err(CliError::new(
            "input",
            format!("{}: /add requires a regular file", path.display()),
        ));
    }
    if metadata.len() > MAX_CONTEXT_FILE_BYTES {
        return Err(CliError::new(
            "input",
            format!(
                "{}: context file exceeds {} bytes",
                path.display(),
                MAX_CONTEXT_FILE_BYTES
            ),
        ));
    }
    fs::read_to_string(path)
        .map(|_| ())
        .map_err(|error| CliError::new("input", format!("{}: {error}", path.display())))
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

fn exposed_answer_text(output: &NaturalOutput) -> String {
    match output.finalization.status {
        FinalizationStatus::GroundedAnswer | FinalizationStatus::QualifiedPartialAnswer => output
            .finalization
            .text
            .clone()
            .unwrap_or_else(|| "No exposed answer text is available.".into()),
        FinalizationStatus::Abstain => {
            "I cannot provide a grounded answer because verified state is contradictory.".into()
        }
        FinalizationStatus::Unresolved | FinalizationStatus::RequiresVerification => {
            "I cannot support a complete answer from the currently verified evidence.".into()
        }
    }
}

pub(super) async fn run(mut base: NaturalArgs) -> Result<(), CliError> {
    base.task = None;
    base.format = Some(OutputFormat::Human);
    println!("Reason interactive — Harness-verified answers. /help for commands.");
    println!("Interactive history is in-memory only; no shell-style history file is written.");

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
                println!("/add <path>  add an untrusted context file for later prompts");
                println!("/files       list context files active in this REPL");
                println!("/clear       clear context files and in-memory conversation context");
                println!("/help        show this help");
                println!("/exit        exit Reason interactive");
                println!("End a line with \\ to continue a multiline prompt.");
            }
            Directive::Files => {
                if base.file.is_empty() {
                    println!("No context files added.");
                } else {
                    for path in &base.file {
                        println!("{}", path.display());
                    }
                }
            }
            Directive::Clear => {
                base.file.clear();
                base.interactive_context.clear();
                println!("Interactive context cleared.");
            }
            Directive::Add(path) => match validate_context_file(&path) {
                Ok(()) => {
                    if !base.file.contains(&path) {
                        base.file.push(path.clone());
                    }
                    println!("Added untrusted context: {}", path.display());
                }
                Err(error) => eprintln!("{}", error.message),
            },
            Directive::Prompt(task) => {
                let mut args = base.clone();
                args.task = Some(task.clone());
                match execute_natural(args, None).await {
                    Ok(output) => {
                        print_natural_human(&output);
                        let exposed = exposed_answer_text(&output);
                        base.interactive_context.push(format!(
                            "Prior interactive exchange (untrusted conversation memory; re-verify before relying on it):\nUser: {task}\nReason: {exposed}"
                        ));
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
