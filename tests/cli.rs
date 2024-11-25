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

#[test]
fn test_permissions() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.args([
        "declare",
        "user",
        "--name",
        "user_with_permissions",
        "--password",
        "pa$$w0rd",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args([
        "declare",
        "permissions",
        "--user",
        "user_with_permissions",
        "--configure",
        "foo",
        "--read",
        "bar",
        "--write",
        "baz",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "permissions"]);
    cmd.assert().success().stdout(
        predicate::str::contains("foo")
            .and(predicate::str::contains("bar"))
            .and(predicate::str::contains("baz")),
    );

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "permissions", "--user", "user_with_permissions"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "permissions"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("user_with_permissions").not());

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "user", "--name", "user_with_permissions"]);
    cmd.assert().success();
    Ok(())
}

#[test]
fn test_policies() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.args([
        "declare",
        "policy",
        "--name",
        "test_policy",
        "--pattern",
        "foo-.*",
        "--apply-to",
        "queues",
        "--priority",
        "123",
        "--definition",
        "{\"max-length\": 12345}",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "policies"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test_policy").and(predicate::str::contains("12345")));

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "policy", "--name", "test_policy"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "policies"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test_policy").not());

    Ok(())
}

#[test]
fn test_operator_policies() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.args([
        "declare",
        "operator_policy",
        "--name",
        "test_operator_policy",
        "--pattern",
        "op-foo.*",
        "--apply-to",
        "queues",
        "--priority",
        "123",
        "--definition",
        "{\"max-length\": 12345}",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "operator_policies"]);
    cmd.assert().success().stdout(
        predicate::str::contains("test_operator_policy").and(predicate::str::contains("op-foo")),
    );

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args([
        "delete",
        "operator_policy",
        "--name",
        "test_operator_policy",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "operator_policies"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test_operator_policy").not());

    Ok(())
}

#[test]
fn test_vhost_limits() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.args([
        "declare",
        "vhost_limit",
        "--name",
        "max-connections",
        "--value",
        "1234",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "vhost_limits"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("max-connections").and(predicate::str::contains("1234")));

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "vhost_limit", "--name", "max-connections"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "vhost_limits"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("max-connections").not());

    Ok(())
}

#[test]
fn test_user_limits() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.args([
        "declare",
        "user_limit",
        "--user",
        "guest",
        "--name",
        "max-connections",
        "--value",
        "1234",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "user_limits"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("max-connections").and(predicate::str::contains("1234")));

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args([
        "delete",
        "user_limit",
        "--user",
        "guest",
        "--name",
        "max-connections",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["list", "user_limits"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("max-connections").not());

    Ok(())
}

#[test]
fn test_runtime_parameters() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["declare", "vhost", "--name", "parameters_vhost_1"]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.args([
        "-V",
        "parameters_vhost_1",
        "declare",
        "parameter",
        "--component",
        "federation-upstream",
        "--name",
        "my-upstream",
        "--value",
        "{\"uri\":\"amqp://target.hostname\",\"expires\":3600000}",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args([
        "-V",
        "parameters_vhost_1",
        "list",
        "parameters",
        "--component",
        "federation-upstream",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("my-upstream").and(predicate::str::contains("3600000")));

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args([
        "-V",
        "parameters_vhost_1",
        "delete",
        "parameter",
        "--component",
        "federation-upstream",
        "--name",
        "my-upstream",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args([
        "-V",
        "parameters_vhost_1",
        "list",
        "parameters",
        "--component",
        "federation-upstream",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("my-upstream").not());

    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["delete", "vhost", "--name", "parameters_vhost_1"]);
    cmd.assert().success();

    Ok(())
}

#[test]
fn test_export_definitions() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.arg("definitions").arg("export");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("guest"));

    Ok(())
}

#[test]
fn test_import_definitions() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;

    cmd.args([
        "definitions",
        "import",
        "--file",
        "tests/fixtures/definitions1.json",
    ]);
    cmd.assert().success();

    Ok(())
}

#[test]
fn test_messages() -> Result<(), Box<dyn std::error::Error>> {
    // declare a new queue
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("declare")
        .arg("queue")
        .arg("--name")
        .arg("publish_consume")
        .arg("--type")
        .arg("classic");
    cmd.assert().success();

    // publish a message
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("publish")
        .arg("message")
        .arg("--routing-key")
        .arg("publish_consume")
        .arg("--payload")
        .arg("test_messages_1")
        .arg("--properties")
        .arg("{\"timestamp\": 1234, \"message_id\": \"foo\"}");
    cmd.assert().success();

    // consume a message
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("get")
        .arg("messages")
        .arg("--queue")
        .arg("publish_consume");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test_messages_1"));

    // delete the test queue
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("delete")
        .arg("queue")
        .arg("--name")
        .arg("publish_consume");
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
