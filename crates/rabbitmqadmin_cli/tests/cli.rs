use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn show_help_with_no_arguments() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    // cmd.arg("foobar").arg("test/file/doesnt/exist");
    cmd.assert().failure().stderr(predicate::str::contains(
        "equires a subcommand but one was not provided",
    ));

    Ok(())
}

#[test]
fn show_subcommands_with_no_arguments() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.assert().failure().stderr(predicate::str::contains(
        "requires a subcommand but one was not provided",
    ));

    Ok(())
}

#[test]
fn nice_error_cant_resolve_host() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.arg("-H").arg("nosuchhost").arg("list").arg("queues");
    cmd.assert().failure().stderr(
        predicate::str::contains("failed to lookup address information")
            .and(predicate::str::contains("nosuchhost")),
    );

    Ok(())
}

#[test]
fn nice_error_when_connection_refused() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.arg("-P").arg("27651").arg("list").arg("queues");
    cmd.assert().failure().stderr(
        predicate::str::contains("Connection refused").and(predicate::str::contains("27651")),
    );

    Ok(())
}
