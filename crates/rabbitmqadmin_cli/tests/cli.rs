use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::env;
use std::process::Command;
use std::time::Duration;

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

    // declare new exchange in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("exchange_vhost_1")
        .arg("declare")
        .arg("exchange")
        .arg("--name")
        .arg("new_exchange1");
    cmd.assert().success();

    // declare new exchange in vhost 2
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

#[test]
fn queues() -> Result<(), Box<dyn std::error::Error>> {
    // declare vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["declare", "vhost", "--name", "queue_vhost_1"]);
    cmd.assert().success();

    // declare vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["declare", "vhost", "--name", "queue_vhost_2"]);
    cmd.assert().success();

    // declare new queue in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("queue_vhost_1")
        .arg("declare")
        .arg("queue")
        .arg("--name")
        .arg("new_queue1")
        .arg("--type")
        .arg("classic");
    cmd.assert().success();

    // declare new queue in vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("queue_vhost_2")
        .arg("declare")
        .arg("queue")
        .arg("--name")
        .arg("new_queue2")
        .arg("--type")
        .arg("quorum");
    cmd.assert().success();

    await_queue_metric_emission();

    // list queues in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["-V", "queue_vhost_1", "list", "queues"]);
    cmd.assert().success().stdout(
        predicate::str::contains("new_queue1").and(predicate::str::contains("new_queue2").not()),
    );

    // delete the queues from vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("queue_vhost_1")
        .arg("delete")
        .arg("queue")
        .arg("--name")
        .arg("new_queue1");
    cmd.assert().success();

    // list queue in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V").arg("queue_vhost_1").arg("list").arg("queues");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("new_queue1").not());

    // delete vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "vhost", "--name", "queue_vhost_1"]);
    cmd.assert().success();

    // delete vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "vhost", "--name", "queue_vhost_2"]);
    cmd.assert().success();

    Ok(())
}

#[test]
fn test_users() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args([
        "declare",
        "user",
        "--name",
        "new_user",
        "--password",
        "pa$$w0rd",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "users"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("new_user"));

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "user", "--name", "new_user"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "users"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("new_user").not());

    Ok(())
}

#[test]
fn test_bindings() -> Result<(), Box<dyn std::error::Error>> {
    // declare vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["declare", "vhost", "--name", "bindings_vhost_1"]);
    cmd.assert().success();

    // declare vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["declare", "vhost", "--name", "bindings_vhost_2"]);
    cmd.assert().success();

    // declare new queue in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("bindings_vhost_1")
        .arg("declare")
        .arg("queue")
        .arg("--name")
        .arg("new_queue_1")
        .arg("--type")
        .arg("classic");
    cmd.assert().success();

    // declare new queue in vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("bindings_vhost_2")
        .arg("declare")
        .arg("queue")
        .arg("--name")
        .arg("new_queue_2")
        .arg("--type")
        .arg("quorum");
    cmd.assert().success();

    // declare exchange -> queue binding
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("bindings_vhost_1")
        .arg("declare")
        .arg("binding")
        .arg("--source")
        .arg("amq.direct")
        .arg("--destination_type")
        .arg("queue")
        .arg("--destination")
        .arg("new_queue_1")
        .arg("--routing_key")
        .arg("routing_key_queue");
    cmd.assert().success();

    // declare exchange -> exchange binding
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("bindings_vhost_1")
        .arg("declare")
        .arg("binding")
        .arg("--source")
        .arg("amq.direct")
        .arg("--destination_type")
        .arg("exchange")
        .arg("--destination")
        .arg("amq.topic")
        .arg("--routing_key")
        .arg("routing_key_exchange");
    cmd.assert().success();
    await_queue_metric_emission();

    // list bindings in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["-V", "bindings_vhost_1", "list", "bindings"]);
    cmd.assert().success().stdout(
        predicate::str::contains("new_queue_1")
            .and(predicate::str::contains("routing_key_queue"))
            .and(predicate::str::contains("routing_key_exchange")),
    );

    // delete the queues from vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg("bindings_vhost_1")
        .arg("delete")
        .arg("queue")
        .arg("--name")
        .arg("new_queue_1");
    cmd.assert().success();

    // this routing_key should not longer be present
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["-V", "bindings_vhost_1", "list", "bindings"]);
    cmd.assert().success().stdout(
        predicate::str::contains("new_queue_1")
            .not()
            .and(predicate::str::contains("routing_key_queue"))
            .not()
            .and(predicate::str::contains("routing_key_exchange")),
    );

    // delete vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "vhost", "--name", "bindings_vhost_1"]);
    cmd.assert().success();

    // delete vhost 2
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "vhost", "--name", "bindings_vhost_2"]);
    cmd.assert().success();

    Ok(())
}

#[allow(dead_code)]
pub fn await_metric_emission(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

#[allow(dead_code)]
pub fn await_queue_metric_emission() {
    let delay = env::var("TEST_STATS_DELAY").unwrap_or("500".to_owned());
    await_metric_emission(delay.parse::<u64>().unwrap());
}
