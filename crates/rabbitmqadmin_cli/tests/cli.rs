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

#[test]
fn list_vhosts() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.arg("list").arg("vhosts");
    cmd.assert().success().stdout(predicate::str::contains("/"));

    Ok(())
}

#[test]
fn exchanges() -> Result<(), Box<dyn std::error::Error>> {
    // declare vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["declare", "vhost", "--name", "exchange_vhost_1"]);
    cmd.assert().success();

    // declare vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["declare", "vhost", "--name", "exchange_vhost_2"]);
    cmd.assert().success();

    // declare new change in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("exchange_vhost_1")
        .arg("declare")
        .arg("exchange")
        .arg("--name")
        .arg("new_exchange1");
    cmd.assert().success();

    // declare new change in vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("exchange_vhost_2")
        .arg("declare")
        .arg("exchange")
        .arg("--name")
        .arg("new_exchange2");
    cmd.assert().success();

    // list exchanges in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["-V", "exchange_vhost_1", "list", "exchanges"]);
    cmd.assert().success().stdout(
        predicate::str::contains("amq.direct")
            .and(predicate::str::contains("amq.fanout"))
            .and(predicate::str::contains("amq.headers"))
            .and(predicate::str::contains("amq.topic"))
            .and(predicate::str::contains("new_exchange1"))
            .and(predicate::str::contains("new_exchange2").not()),
    );

    // delete the exchanges from vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("exchange_vhost_1")
        .arg("delete")
        .arg("exchange")
        .arg("--name")
        .arg("new_exchange1");
    cmd.assert().success();

    // list exchange in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("exchange_vhost_1")
        .arg("list")
        .arg("exchanges");
    cmd.assert().success().stdout(
        predicate::str::contains("amq.direct")
            .and(predicate::str::contains("amq.fanout"))
            .and(predicate::str::contains("amq.headers"))
            .and(predicate::str::contains("amq.topic"))
            .and(predicate::str::contains("new_exchange1").not()),
    );

    // delete vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "vhost", "--name", "exchange_vhost_1"]);
    cmd.assert().success();

    // delete vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "vhost", "--name", "exchange_vhost_2"]);
    cmd.assert().success();

    Ok(())
}
