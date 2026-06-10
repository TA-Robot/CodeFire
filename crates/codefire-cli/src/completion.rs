pub(crate) const COMPLETION_COMMANDS: &[&str] = &[
    "init",
    "open",
    "branch",
    "status",
    "scan",
    "fire",
    "extinguish",
    "verify",
    "commit",
    "clone",
    "merge",
    "upload",
    "list",
    "request-merge",
    "request-list",
    "request-review",
    "request-apply",
    "show",
    "diff",
    "review-pack",
    "patch",
    "doctor",
    "storage",
    "evidence",
    "explain",
    "context",
    "migrate",
    "serve",
    "completion",
    "version",
];

pub(crate) fn help_text() -> String {
    format!(
        "\
codefire {version}

usage: codefire <command> [args]

commands:
  {commands}

common:
  codefire init [path]
  codefire open <branch> <path> [--dry-run] [--json]
  codefire scan [path] [--json] [--metrics]
  codefire verify [path] [--details] [--blocking-only] [--json] [--metrics]
  codefire doctor [path] [--quick] [--json]
  codefire commit -m <message> [--signer <name>] [--key-id <id>]
  codefire completion <bash|zsh>
",
        version = codefire_core::VERSION,
        commands = COMPLETION_COMMANDS.join(" "),
    )
}

pub(crate) fn completion_script(shell: &str) -> Option<String> {
    match shell {
        "bash" => Some(bash_completion_script()),
        "zsh" => Some(zsh_completion_script()),
        _ => None,
    }
}

pub(crate) fn bash_completion_script() -> String {
    let commands = COMPLETION_COMMANDS.join(" ");
    format!(
        r#"# bash completion for codefire
_codefire_complete()
{{
  local cur cmd
  COMPREPLY=()
  cur="${{COMP_WORDS[COMP_CWORD]}}"
  cmd="${{COMP_WORDS[1]}}"
  local commands="{commands}"

  if [[ $COMP_CWORD -eq 1 ]]; then
    COMPREPLY=( $(compgen -W "$commands" -- "$cur") )
    return 0
  fi

  case "$cmd" in
    branch)
      if [[ $COMP_CWORD -eq 2 ]]; then COMPREPLY=( $(compgen -W "list" -- "$cur") ); fi
      ;;
    open|clone|merge|commit|extinguish|patch)
      COMPREPLY=( $(compgen -W "--dry-run --json --idempotency-key --wait-lock --lock-timeout --into --message -m --signer --key-id --resolution --rationale --evidence --evidence-ref --interactive --all-matching --batch --batch-template --full --output" -- "$cur") )
      ;;
    upload|request-merge|request-review|request-apply)
      COMPREPLY=( $(compgen -W "--dry-run --json --idempotency-key --wait-lock --lock-timeout --actor --token --request-key-id --reviewer --decision --comment" -- "$cur") )
      ;;
    scan|status)
      COMPREPLY=( $(compgen -W "--json --metrics" -- "$cur") )
      ;;
    verify)
      COMPREPLY=( $(compgen -W "--details --blocking-only --json --metrics" -- "$cur") )
      ;;
    doctor)
      COMPREPLY=( $(compgen -W "--quick --full --json" -- "$cur") )
      ;;
    diff)
      COMPREPLY=( $(compgen -W "--algorithm --context --rename-detection --atoms --trace --impact --json" -- "$cur") )
      ;;
    review-pack)
      COMPREPLY=( $(compgen -W "--base --output --algorithm --no-rename-detection" -- "$cur") )
      ;;
    storage)
      if [[ $COMP_CWORD -eq 2 ]]; then
        COMPREPLY=( $(compgen -W "report" -- "$cur") )
      else
        COMPREPLY=( $(compgen -W "--quick --full --json --large-threshold --remote" -- "$cur") )
      fi
      ;;
    evidence)
      if [[ $COMP_CWORD -eq 2 ]]; then
        COMPREPLY=( $(compgen -W "add" -- "$cur") )
      else
        COMPREPLY=( $(compgen -W "--path --artifact --from-command --from-argv --argv --batch --label --dry-run --json" -- "$cur") )
      fi
      ;;
    explain)
      if [[ $COMP_CWORD -eq 2 ]]; then COMPREPLY=( $(compgen -W "fire atom verify-failure storage-warning" -- "$cur") ); fi
      ;;
    context)
      COMPREPLY=( $(compgen -W "--path --branch --changed --atom --fire --json" -- "$cur") )
      ;;
    migrate)
      if [[ $COMP_CWORD -eq 2 ]]; then
        COMPREPLY=( $(compgen -W "check dry-run" -- "$cur") )
      else
        COMPREPLY=( $(compgen -W "--json --target-format" -- "$cur") )
      fi
      ;;
    serve)
      COMPREPLY=( $(compgen -W "--host --port --tls-cert --tls-key --tls-client-ca" -- "$cur") )
      ;;
    completion)
      COMPREPLY=( $(compgen -W "bash zsh" -- "$cur") )
      ;;
  esac
  return 0
}}
complete -F _codefire_complete codefire
"#,
    )
}

pub(crate) fn zsh_completion_script() -> String {
    let command_entries = COMPLETION_COMMANDS
        .iter()
        .map(|command| format!("{command}\\:{command}"))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        r#"#compdef codefire
# zsh completion for codefire
_codefire()
{{
  local -a commands
  commands=({command_entries})

  if (( CURRENT == 2 )); then
    _describe 'command' commands
    return
  fi

  case $words[2] in
    branch)
      _arguments '1:branch command:(list)'
      ;;
    open|clone|merge|commit|extinguish|patch)
      _arguments '--dry-run[dry run]' '--json[emit JSON]' '--idempotency-key[idempotency key]' '--wait-lock[wait for locks]' '--lock-timeout[lock timeout]' '--into[target branch]' '--message[commit message]' '-m[commit message]' '--signer[commit signer]' '--key-id[commit signing key id]' '--resolution[resolution]' '--rationale[rationale]' '--evidence[evidence text]' '--evidence-ref[evidence object id]' '--interactive[interactive resolution]' '--all-matching[extinguish matching source-target fires]' '--batch[batch file]' '--batch-template[generate extinguish batch template]' '--full[include full output]' '--output[output file]'
      ;;
    upload|request-merge|request-review|request-apply)
      _arguments '--dry-run[dry run]' '--json[emit JSON]' '--idempotency-key[idempotency key]' '--wait-lock[wait for locks]' '--lock-timeout[lock timeout]' '--actor[remote actor]' '--token[remote token]' '--request-key-id[remote request signing key id]' '--reviewer[reviewer]' '--decision[decision]:decision:(approve reject)' '--comment[comment]'
      ;;
    scan|status)
      _arguments '--json[emit JSON]' '--metrics[show metrics]'
      ;;
    verify)
      _arguments '--details[show failed verification diagnostic details]' '--blocking-only[show only blocking diagnostics]' '--json[emit JSON]' '--metrics[show metrics]'
      ;;
    doctor)
      _arguments '--quick[skip full object store integrity scan]' '--full[run full object store integrity scan]' '--json[emit JSON]'
      ;;
    diff)
      _arguments '--algorithm[diff algorithm]:algorithm:(myers patience histogram)' '--context[context lines]:lines:' '--rename-detection[detect renames and copies]' '--atoms[include Atom diff]' '--trace[include TraceGraph diff]' '--impact[include policy and fire impact]' '--json[emit JSON]'
      ;;
    review-pack)
      _arguments '--base[base commitish]' '--output[output file]' '--algorithm[diff algorithm]:algorithm:(myers patience histogram)' '--no-rename-detection[disable rename detection]'
      ;;
    storage)
      _arguments '1:storage command:(report)' '--quick[skip object JSON parsing]' '--full[include object type and artifact analysis]' '--json[emit JSON]' '--large-threshold[large object threshold]' '--remote[remote project URL]'
      ;;
    evidence)
      _arguments '1:evidence command:(add)' '--path[repo or open path]' '--artifact[artifact path]' '--from-command[shell command]' '--from-argv[program]' '--argv[argv argument]' '--batch[batch file]' '--label[label]' '--dry-run[dry run]' '--json[emit JSON]'
      ;;
    explain)
      _arguments '1:explain subject:(fire atom verify-failure storage-warning)'
      ;;
    context)
      _arguments '--path[repo or open path]' '--branch[branch]' '--changed[changed context]' '--atom[Atom id]' '--fire[fire id]' '--json[emit JSON]'
      ;;
    migrate)
      _arguments '1:migrate command:(check dry-run)' '--json[emit JSON]' '--target-format[target format]'
      ;;
    serve)
      _arguments '--host[host]' '--port[port]' '--tls-cert[TLS certificate]' '--tls-key[TLS private key]' '--tls-client-ca[required client CA PEM]'
      ;;
    completion)
      _arguments '1:shell:(bash zsh)'
      ;;
  esac
}}
_codefire "$@"
"#,
    )
}
