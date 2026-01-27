// Copyright (C) 2023-2026 RabbitMQ Core Team (teamrabbitmq@gmail.com)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod test_helpers;

use bel7_cli::CommandShellExt;
use std::error::Error;
use std::ffi::OsStr;
use std::process::Command;
use test_helpers::{output_includes, run_succeeds};

fn run_with_shell_env<I, S>(args: I, shell_path: &str) -> assert_cmd::assert::Assert
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("rabbitmqadmin"));
    cmd.clear_shell_detection_env();
    cmd.env("SHELL", shell_path);
    cmd.args(args);
    assert_cmd::assert::Assert::new(cmd.output().unwrap())
}

#[test]
fn shell_completions_bash() -> Result<(), Box<dyn Error>> {
    run_succeeds(["shell", "completions", "--shell", "bash"])
        .stdout(output_includes("_rabbitmqadmin"));
    Ok(())
}

#[test]
fn shell_completions_zsh() -> Result<(), Box<dyn Error>> {
    run_succeeds(["shell", "completions", "--shell", "zsh"])
        .stdout(output_includes("#compdef rabbitmqadmin"));
    Ok(())
}

#[test]
fn shell_completions_fish() -> Result<(), Box<dyn Error>> {
    run_succeeds(["shell", "completions", "--shell", "fish"])
        .stdout(output_includes("complete -c rabbitmqadmin"));
    Ok(())
}

#[test]
fn shell_completions_elvish() -> Result<(), Box<dyn Error>> {
    run_succeeds(["shell", "completions", "--shell", "elvish"]).stdout(output_includes(
        "set edit:completion:arg-completer[rabbitmqadmin]",
    ));
    Ok(())
}

#[test]
fn shell_completions_nushell() -> Result<(), Box<dyn Error>> {
    run_succeeds(["shell", "completions", "--shell", "nushell"])
        .stdout(output_includes("module completions"));
    Ok(())
}

#[test]
fn shell_completions_nu_alias() -> Result<(), Box<dyn Error>> {
    run_succeeds(["shell", "completions", "--shell", "nu"])
        .stdout(output_includes("module completions"));
    Ok(())
}

#[test]
fn shell_completions_detects_bash() -> Result<(), Box<dyn Error>> {
    run_with_shell_env(["shell", "completions"], "/bin/bash")
        .success()
        .stdout(output_includes("_rabbitmqadmin"));
    Ok(())
}

#[test]
fn shell_completions_detects_zsh() -> Result<(), Box<dyn Error>> {
    run_with_shell_env(["shell", "completions"], "/bin/zsh")
        .success()
        .stdout(output_includes("#compdef rabbitmqadmin"));
    Ok(())
}

#[test]
fn shell_completions_detects_fish() -> Result<(), Box<dyn Error>> {
    run_with_shell_env(["shell", "completions"], "/usr/bin/fish")
        .success()
        .stdout(output_includes("complete -c rabbitmqadmin"));
    Ok(())
}

#[test]
fn shell_completions_detects_elvish() -> Result<(), Box<dyn Error>> {
    run_with_shell_env(["shell", "completions"], "/usr/local/bin/elvish")
        .success()
        .stdout(output_includes(
            "set edit:completion:arg-completer[rabbitmqadmin]",
        ));
    Ok(())
}

#[test]
fn shell_completions_detects_nushell() -> Result<(), Box<dyn Error>> {
    run_with_shell_env(["shell", "completions"], "/opt/homebrew/bin/nu")
        .success()
        .stdout(output_includes("module completions"));
    Ok(())
}

#[test]
fn shell_completions_defaults_to_bash_for_unknown_shell() -> Result<(), Box<dyn Error>> {
    run_with_shell_env(["shell", "completions"], "/unknown/shell")
        .success()
        .stdout(output_includes("_rabbitmqadmin"));
    Ok(())
}

#[test]
fn shell_completions_defaults_to_bash_when_shell_env_unset() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("rabbitmqadmin"));
    cmd.clear_shell_detection_env();
    cmd.args(["shell", "completions"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("_rabbitmqadmin"));
    Ok(())
}

#[test]
fn shell_completions_includes_subcommands() -> Result<(), Box<dyn Error>> {
    let output = run_succeeds(["shell", "completions", "--shell", "bash"]);
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("declare"));
    assert!(stdout.contains("delete"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("show"));
    assert!(stdout.contains("queues"));
    assert!(stdout.contains("exchanges"));
    assert!(stdout.contains("vhosts"));
    Ok(())
}

#[test]
fn shell_help_shows_completions_subcommand() -> Result<(), Box<dyn Error>> {
    run_succeeds(["shell", "--help"]).stdout(output_includes("completions"));
    Ok(())
}

#[test]
fn shell_completions_help() -> Result<(), Box<dyn Error>> {
    run_succeeds(["shell", "completions", "--help"])
        .stdout(output_includes("shell completion scripts"));
    Ok(())
}

#[test]
fn shell_completions_rejects_invalid_shell() {
    test_helpers::run_fails(["shell", "completions", "--shell", "invalid"])
        .stderr(output_includes("invalid value 'invalid'"));
}

#[test]
fn shell_completions_detects_shell_name_without_path() -> Result<(), Box<dyn Error>> {
    run_with_shell_env(["shell", "completions"], "zsh")
        .success()
        .stdout(output_includes("#compdef rabbitmqadmin"));
    Ok(())
}

mod property_tests {
    use bel7_cli::CommandShellExt;
    use proptest::prelude::*;
    use std::ffi::OsStr;
    use std::process::Command;

    fn run_with_shell_env<I, S>(args: I, shell_path: &str) -> assert_cmd::assert::Assert
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("rabbitmqadmin"));
        cmd.clear_shell_detection_env();
        cmd.env("SHELL", shell_path);
        cmd.args(args);
        assert_cmd::assert::Assert::new(cmd.output().unwrap())
    }

    proptest! {
        #[test]
        fn unknown_shells_default_to_bash(random_path in "[a-z0-9/]{1,50}") {
            let path = format!("/some/path/{}", random_path);
            let result = run_with_shell_env(["shell", "completions"], &path);
            let output = result.get_output();
            prop_assert!(output.status.success());
            let stdout = String::from_utf8_lossy(&output.stdout);
            prop_assert!(stdout.contains("_rabbitmqadmin") || stdout.contains("#compdef") ||
                         stdout.contains("complete -c") || stdout.contains("module completions") ||
                         stdout.contains("edit:completion:arg-completer"));
        }

        #[test]
        fn bash_detected_with_any_path_prefix(prefix in "[a-z0-9/]{0,30}") {
            let path = format!("{}/bash", prefix);
            let result = run_with_shell_env(["shell", "completions"], &path);
            let output = result.get_output();
            prop_assert!(output.status.success());
            let stdout = String::from_utf8_lossy(&output.stdout);
            prop_assert!(stdout.contains("_rabbitmqadmin"));
        }

        #[test]
        fn zsh_detected_with_any_path_prefix(prefix in "[a-z0-9/]{0,30}") {
            let path = format!("{}/zsh", prefix);
            let result = run_with_shell_env(["shell", "completions"], &path);
            let output = result.get_output();
            prop_assert!(output.status.success());
            let stdout = String::from_utf8_lossy(&output.stdout);
            prop_assert!(stdout.contains("#compdef rabbitmqadmin"));
        }

        #[test]
        fn fish_detected_with_any_path_prefix(prefix in "[a-z0-9/]{0,30}") {
            let path = format!("{}/fish", prefix);
            let result = run_with_shell_env(["shell", "completions"], &path);
            let output = result.get_output();
            prop_assert!(output.status.success());
            let stdout = String::from_utf8_lossy(&output.stdout);
            prop_assert!(stdout.contains("complete -c rabbitmqadmin"));
        }
    }
}
