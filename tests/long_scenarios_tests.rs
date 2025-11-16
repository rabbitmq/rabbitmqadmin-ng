use predicates::prelude::*;
use std::error::Error;

mod test_helpers;
use test_helpers::*;

#[test]
fn test_multi_vhost_with_users_and_permissions() -> Result<(), Box<dyn Error>> {
    let vhost1 = "long.scenario.vhost1";
    let vhost2 = "long.scenario.vhost2";
    let vhost3 = "long.scenario.vhost3";
    let user1 = "long.scenario.user1";
    let user2 = "long.scenario.user2";
    let user3 = "long.scenario.user3";

    delete_vhost(vhost1).ok();
    delete_vhost(vhost2).ok();
    delete_vhost(vhost3).ok();
    delete_user(user1).ok();
    delete_user(user2).ok();
    delete_user(user3).ok();

    run_succeeds(["vhosts", "declare", "--name", vhost1]);
    run_succeeds([
        "vhosts",
        "declare",
        "--name",
        vhost2,
        "--default-queue-type",
        "quorum",
    ]);
    run_succeeds([
        "vhosts",
        "declare",
        "--name",
        vhost3,
        "--default-queue-type",
        "stream",
    ]);

    run_succeeds(["users", "declare", "--name", user1, "--password", "pass1"]);
    run_succeeds([
        "users",
        "declare",
        "--name",
        user2,
        "--password",
        "pass2",
        "--tags",
        "monitoring",
    ]);
    run_succeeds([
        "users",
        "declare",
        "--name",
        user3,
        "--password",
        "pass3",
        "--tags",
        "policymaker,management",
    ]);

    run_succeeds([
        "--vhost",
        vhost1,
        "permissions",
        "declare",
        "--user",
        user1,
        "--configure",
        ".*",
        "--read",
        ".*",
        "--write",
        ".*",
    ]);
    run_succeeds([
        "--vhost",
        vhost2,
        "permissions",
        "declare",
        "--user",
        user2,
        "--configure",
        "",
        "--read",
        ".*",
        "--write",
        "",
    ]);
    run_succeeds([
        "--vhost",
        vhost3,
        "permissions",
        "declare",
        "--user",
        user3,
        "--configure",
        "^test\\..*",
        "--read",
        "^test\\..*",
        "--write",
        "^test\\..*",
    ]);

    run_succeeds(["permissions", "list"])
        .stdout(output_includes(user1).and(output_includes(vhost1)));
    run_succeeds(["permissions", "list"])
        .stdout(output_includes(user2).and(output_includes(vhost2)));
    run_succeeds(["permissions", "list"])
        .stdout(output_includes(user3).and(output_includes(vhost3)));

    run_succeeds(["--vhost", vhost1, "permissions", "delete", "--user", user1]);
    run_succeeds(["--vhost", vhost2, "permissions", "delete", "--user", user2]);
    run_succeeds(["--vhost", vhost3, "permissions", "delete", "--user", user3]);

    delete_user(user1)?;
    delete_user(user2)?;
    delete_user(user3)?;
    delete_vhost(vhost1)?;
    delete_vhost(vhost2)?;
    delete_vhost(vhost3)?;

    Ok(())
}

#[test]
fn test_queue_types_with_bindings_and_policies() -> Result<(), Box<dyn Error>> {
    let vhost = "long.scenario.queues";
    let classic_queue = "long.classic.queue";
    let quorum_queue = "long.quorum.queue";
    let exchange_topic = "long.topic.exchange";
    let exchange_fanout = "long.fanout.exchange";
    let policy_ttl = "long.ttl.policy";

    delete_vhost(vhost).ok();
    create_vhost(vhost)?;

    run_succeeds([
        "--vhost",
        vhost,
        "queues",
        "declare",
        "--name",
        classic_queue,
        "--type",
        "classic",
        "--durable",
        "true",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "queues",
        "declare",
        "--name",
        quorum_queue,
        "--type",
        "quorum",
        "--durable",
        "true",
    ]);

    run_succeeds([
        "--vhost",
        vhost,
        "exchanges",
        "declare",
        "--name",
        exchange_topic,
        "--type",
        "topic",
        "--durable",
        "true",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "exchanges",
        "declare",
        "--name",
        exchange_fanout,
        "--type",
        "fanout",
        "--durable",
        "true",
    ]);

    run_succeeds([
        "--vhost",
        vhost,
        "bindings",
        "declare",
        "--source",
        exchange_topic,
        "--destination-type",
        "queue",
        "--destination",
        classic_queue,
        "--routing-key",
        "events.#",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "bindings",
        "declare",
        "--source",
        exchange_topic,
        "--destination-type",
        "queue",
        "--destination",
        quorum_queue,
        "--routing-key",
        "events.critical.*",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "bindings",
        "declare",
        "--source",
        exchange_fanout,
        "--destination-type",
        "queue",
        "--destination",
        classic_queue,
    ]);

    run_succeeds([
        "--vhost",
        vhost,
        "policies",
        "declare",
        "--name",
        policy_ttl,
        "--pattern",
        "^long\\..*",
        "--definition",
        r#"{"message-ttl":60000,"expires":120000}"#,
        "--priority",
        "1",
    ]);

    await_queue_metric_emission();

    run_succeeds(["--vhost", vhost, "queues", "list"])
        .stdout(output_includes(classic_queue).and(output_includes(quorum_queue)));
    run_succeeds(["--vhost", vhost, "exchanges", "list"])
        .stdout(output_includes(exchange_topic).and(output_includes(exchange_fanout)));
    run_succeeds(["--vhost", vhost, "bindings", "list"]).stdout(output_includes("events.#"));
    run_succeeds(["--vhost", vhost, "policies", "list"]).stdout(output_includes(policy_ttl));

    run_succeeds([
        "--vhost",
        vhost,
        "bindings",
        "delete",
        "--source",
        exchange_topic,
        "--destination-type",
        "queue",
        "--destination",
        classic_queue,
        "--routing-key",
        "events.#",
    ]);
    run_succeeds(["--vhost", vhost, "policies", "delete", "--name", policy_ttl]);
    run_succeeds([
        "--vhost",
        vhost,
        "queues",
        "delete",
        "--name",
        classic_queue,
    ]);
    run_succeeds(["--vhost", vhost, "queues", "delete", "--name", quorum_queue]);
    run_succeeds([
        "--vhost",
        vhost,
        "exchanges",
        "delete",
        "--name",
        exchange_topic,
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "exchanges",
        "delete",
        "--name",
        exchange_fanout,
    ]);

    delete_vhost(vhost)?;

    Ok(())
}

#[test]
fn test_user_limits_and_vhost_limits() -> Result<(), Box<dyn Error>> {
    let vhost = "long.scenario.limits";
    let user = "long.limited.user";

    delete_vhost(vhost).ok();
    delete_user(user).ok();

    create_vhost(vhost)?;
    run_succeeds(["users", "declare", "--name", user, "--password", "pass"]);

    run_succeeds([
        "--vhost",
        vhost,
        "permissions",
        "declare",
        "--user",
        user,
        "--configure",
        ".*",
        "--read",
        ".*",
        "--write",
        ".*",
    ]);

    run_succeeds([
        "user_limits",
        "declare",
        "--user",
        user,
        "--limit-type",
        "max-connections",
        "--value",
        "10",
    ]);
    run_succeeds([
        "user_limits",
        "declare",
        "--user",
        user,
        "--limit-type",
        "max-channels",
        "--value",
        "50",
    ]);

    run_succeeds([
        "--vhost",
        vhost,
        "vhost_limits",
        "declare",
        "--limit-type",
        "max-connections",
        "--value",
        "100",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "vhost_limits",
        "declare",
        "--limit-type",
        "max-queues",
        "--value",
        "500",
    ]);

    run_succeeds(["user_limits", "list"])
        .stdout(output_includes(user).and(output_includes("max-connections")));
    run_succeeds(["--vhost", vhost, "vhost_limits", "list"])
        .stdout(output_includes("max-connections").and(output_includes("max-queues")));

    run_succeeds([
        "user_limits",
        "delete",
        "--user",
        user,
        "--limit-type",
        "max-connections",
    ]);
    run_succeeds([
        "user_limits",
        "delete",
        "--user",
        user,
        "--limit-type",
        "max-channels",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "vhost_limits",
        "delete",
        "--limit-type",
        "max-connections",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "vhost_limits",
        "delete",
        "--limit-type",
        "max-queues",
    ]);

    run_succeeds(["--vhost", vhost, "permissions", "delete", "--user", user]);
    delete_user(user)?;
    delete_vhost(vhost)?;

    Ok(())
}

#[test]
fn test_policies_and_operator_policies() -> Result<(), Box<dyn Error>> {
    let vhost = "long.scenario.policies";
    let policy1 = "long.policy.ha";
    let policy2 = "long.policy.ttl";
    let op_policy = "long.op.policy.maxlength";

    delete_vhost(vhost).ok();
    create_vhost(vhost)?;

    run_succeeds([
        "--vhost",
        vhost,
        "policies",
        "declare",
        "--name",
        policy1,
        "--pattern",
        "^ha\\..*",
        "--definition",
        r#"{"ha-mode":"all"}"#,
        "--priority",
        "1",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "policies",
        "declare",
        "--name",
        policy2,
        "--pattern",
        "^temp\\..*",
        "--definition",
        r#"{"message-ttl":300000}"#,
        "--priority",
        "2",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "operator_policies",
        "declare",
        "--name",
        op_policy,
        "--pattern",
        ".*",
        "--definition",
        r#"{"max-length":10000}"#,
        "--priority",
        "10",
    ]);

    run_succeeds(["--vhost", vhost, "policies", "list"])
        .stdout(output_includes(policy1).and(output_includes(policy2)));
    run_succeeds(["--vhost", vhost, "operator_policies", "list"])
        .stdout(output_includes(op_policy));

    run_succeeds([
        "--vhost",
        vhost,
        "policies",
        "update_definition",
        "--name",
        policy1,
        "--definition",
        r#"{"ha-mode":"exactly","ha-params":2}"#,
    ]);

    run_succeeds([
        "--vhost",
        vhost,
        "policies",
        "patch_definition",
        "--name",
        policy2,
        "--definition",
        r#"{"expires":600000}"#,
    ]);

    run_succeeds(["--vhost", vhost, "policies", "delete", "--name", policy1]);
    run_succeeds(["--vhost", vhost, "policies", "delete", "--name", policy2]);
    run_succeeds([
        "--vhost",
        vhost,
        "operator_policies",
        "delete",
        "--name",
        op_policy,
    ]);

    delete_vhost(vhost)?;

    Ok(())
}

#[test]
fn test_multiple_queue_types_and_stream() -> Result<(), Box<dyn Error>> {
    let vhost = "long.scenario.qtypes";
    let classic = "long.classic.q";
    let quorum = "long.quorum.q";
    let stream = "long.stream.s";

    delete_vhost(vhost).ok();
    create_vhost(vhost)?;

    run_succeeds([
        "--vhost",
        vhost,
        "queues",
        "declare",
        "--name",
        classic,
        "--type",
        "classic",
        "--durable",
        "true",
        "--auto-delete",
        "false",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "queues",
        "declare",
        "--name",
        quorum,
        "--type",
        "quorum",
        "--durable",
        "true",
    ]);
    run_succeeds(["--vhost", vhost, "streams", "declare", "--name", stream]);

    await_queue_metric_emission();

    run_succeeds(["--vhost", vhost, "queues", "list"])
        .stdout(output_includes(classic).and(output_includes(quorum)));
    run_succeeds(["--vhost", vhost, "streams", "list"]).stdout(output_includes(stream));

    run_succeeds(["--vhost", vhost, "queues", "purge", "--name", classic]);

    run_succeeds(["--vhost", vhost, "queues", "delete", "--name", classic]);
    run_succeeds(["--vhost", vhost, "queues", "delete", "--name", quorum]);
    run_succeeds(["--vhost", vhost, "streams", "delete", "--name", stream]);

    delete_vhost(vhost)?;

    Ok(())
}

#[test]
fn test_exchange_types_and_bindings() -> Result<(), Box<dyn Error>> {
    let vhost = "long.scenario.exchanges";
    let direct_ex = "long.direct.ex";
    let topic_ex = "long.topic.ex";
    let fanout_ex = "long.fanout.ex";
    let headers_ex = "long.headers.ex";
    let queue1 = "long.q1";
    let queue2 = "long.q2";

    delete_vhost(vhost).ok();
    create_vhost(vhost)?;

    run_succeeds([
        "--vhost",
        vhost,
        "exchanges",
        "declare",
        "--name",
        direct_ex,
        "--type",
        "direct",
        "--durable",
        "true",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "exchanges",
        "declare",
        "--name",
        topic_ex,
        "--type",
        "topic",
        "--durable",
        "true",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "exchanges",
        "declare",
        "--name",
        fanout_ex,
        "--type",
        "fanout",
        "--durable",
        "true",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "exchanges",
        "declare",
        "--name",
        headers_ex,
        "--type",
        "headers",
        "--durable",
        "true",
    ]);

    run_succeeds([
        "--vhost", vhost, "queues", "declare", "--name", queue1, "--type", "classic",
    ]);
    run_succeeds([
        "--vhost", vhost, "queues", "declare", "--name", queue2, "--type", "quorum",
    ]);

    run_succeeds([
        "--vhost",
        vhost,
        "bindings",
        "declare",
        "--source",
        direct_ex,
        "--destination-type",
        "queue",
        "--destination",
        queue1,
        "--routing-key",
        "direct.key",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "bindings",
        "declare",
        "--source",
        topic_ex,
        "--destination-type",
        "queue",
        "--destination",
        queue1,
        "--routing-key",
        "topic.*.key",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "bindings",
        "declare",
        "--source",
        fanout_ex,
        "--destination-type",
        "queue",
        "--destination",
        queue2,
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "bindings",
        "declare",
        "--source",
        topic_ex,
        "--destination-type",
        "exchange",
        "--destination",
        fanout_ex,
        "--routing-key",
        "forward.#",
    ]);

    await_queue_metric_emission();

    run_succeeds(["--vhost", vhost, "exchanges", "list"]).stdout(
        output_includes(direct_ex)
            .and(output_includes(topic_ex))
            .and(output_includes(fanout_ex))
            .and(output_includes(headers_ex)),
    );
    run_succeeds(["--vhost", vhost, "bindings", "list"])
        .stdout(output_includes("direct.key").and(output_includes("topic.*.key")));

    run_succeeds([
        "--vhost",
        vhost,
        "bindings",
        "delete",
        "--source",
        direct_ex,
        "--destination-type",
        "queue",
        "--destination",
        queue1,
        "--routing-key",
        "direct.key",
    ]);
    run_succeeds(["--vhost", vhost, "queues", "delete", "--name", queue1]);
    run_succeeds(["--vhost", vhost, "queues", "delete", "--name", queue2]);
    run_succeeds(["--vhost", vhost, "exchanges", "delete", "--name", direct_ex]);
    run_succeeds(["--vhost", vhost, "exchanges", "delete", "--name", topic_ex]);
    run_succeeds(["--vhost", vhost, "exchanges", "delete", "--name", fanout_ex]);
    run_succeeds([
        "--vhost",
        vhost,
        "exchanges",
        "delete",
        "--name",
        headers_ex,
    ]);

    delete_vhost(vhost)?;

    Ok(())
}

#[test]
fn test_runtime_parameters() -> Result<(), Box<dyn Error>> {
    let vhost = "long.scenario.params";
    let param1 = "long.param1";
    let param2 = "long.param2";
    let component = "test-component";

    delete_vhost(vhost).ok();
    create_vhost(vhost)?;

    run_succeeds([
        "--vhost",
        vhost,
        "parameters",
        "set",
        "--component",
        component,
        "--name",
        param1,
        "--value",
        r#"{"key1":"value1"}"#,
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "parameters",
        "set",
        "--component",
        component,
        "--name",
        param2,
        "--value",
        r#"{"key2":"value2","nested":{"inner":"data"}}"#,
    ]);

    run_succeeds(["--vhost", vhost, "parameters", "list"])
        .stdout(output_includes(param1).and(output_includes(param2)));

    run_succeeds([
        "--vhost",
        vhost,
        "parameters",
        "delete",
        "--component",
        component,
        "--name",
        param1,
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "parameters",
        "delete",
        "--component",
        component,
        "--name",
        param2,
        "--idempotently",
    ]);

    delete_vhost(vhost)?;

    Ok(())
}

#[test]
fn test_federation_upstreams() -> Result<(), Box<dyn Error>> {
    let vhost = "long.scenario.federation";
    let upstream1 = "long.upstream.ex";
    let upstream2 = "long.upstream.q";

    delete_vhost(vhost).ok();
    create_vhost(vhost)?;

    let uri = amqp_endpoint_with_vhost(vhost);

    run_succeeds([
        "--vhost",
        vhost,
        "federation",
        "declare_upstream_for_exchanges",
        "--name",
        upstream1,
        "--uri",
        &uri,
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "federation",
        "declare_upstream_for_queues",
        "--name",
        upstream2,
        "--uri",
        &uri,
    ]);

    run_succeeds(["--vhost", vhost, "federation", "list_upstreams"])
        .stdout(output_includes(upstream1).and(output_includes(upstream2)));

    run_succeeds([
        "--vhost",
        vhost,
        "federation",
        "delete_upstream",
        "--name",
        upstream1,
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "federation",
        "delete_upstream",
        "--name",
        upstream2,
    ]);

    delete_vhost(vhost)?;

    Ok(())
}

#[test]
fn test_shovels() -> Result<(), Box<dyn Error>> {
    let vhost = "long.scenario.shovels";
    let shovel1 = "long.shovel.amqp091";
    let source_queue = "long.source.queue";
    let dest_queue = "long.dest.queue";

    delete_vhost(vhost).ok();
    create_vhost(vhost)?;

    run_succeeds([
        "--vhost",
        vhost,
        "queues",
        "declare",
        "--name",
        source_queue,
        "--type",
        "classic",
    ]);
    run_succeeds([
        "--vhost", vhost, "queues", "declare", "--name", dest_queue, "--type", "classic",
    ]);

    let uri = amqp_endpoint_with_vhost(vhost);

    run_succeeds([
        "--vhost",
        vhost,
        "shovels",
        "declare_amqp091",
        "--name",
        shovel1,
        "--source-uri",
        &uri,
        "--destination-uri",
        &uri,
        "--source-queue",
        source_queue,
        "--destination-queue",
        dest_queue,
    ]);

    await_ms(1000);

    run_succeeds(["--vhost", vhost, "shovels", "list"]).stdout(output_includes(shovel1));

    run_succeeds(["--vhost", vhost, "shovels", "delete", "--name", shovel1]);
    run_succeeds(["--vhost", vhost, "queues", "delete", "--name", source_queue]);
    run_succeeds(["--vhost", vhost, "queues", "delete", "--name", dest_queue]);

    delete_vhost(vhost)?;

    Ok(())
}

#[test]
fn test_vhost_deletion_protection() -> Result<(), Box<dyn Error>> {
    let vhost = "long.scenario.protected";

    delete_vhost(vhost).ok();
    create_vhost(vhost)?;

    run_succeeds(["vhosts", "enable_deletion_protection", "--name", vhost]);

    run_fails(["vhosts", "delete", "--name", vhost]);

    run_succeeds(["vhosts", "disable_deletion_protection", "--name", vhost]);

    delete_vhost(vhost)?;

    Ok(())
}

#[test]
fn test_complex_permission_patterns() -> Result<(), Box<dyn Error>> {
    let vhost = "long.scenario.perms";
    let user_admin = "long.admin.user";
    let user_readonly = "long.readonly.user";
    let user_limited = "long.limited.user";

    delete_vhost(vhost).ok();
    delete_user(user_admin).ok();
    delete_user(user_readonly).ok();
    delete_user(user_limited).ok();

    create_vhost(vhost)?;
    run_succeeds([
        "users",
        "declare",
        "--name",
        user_admin,
        "--password",
        "admin",
    ]);
    run_succeeds([
        "users",
        "declare",
        "--name",
        user_readonly,
        "--password",
        "readonly",
    ]);
    run_succeeds([
        "users",
        "declare",
        "--name",
        user_limited,
        "--password",
        "limited",
    ]);

    run_succeeds([
        "--vhost",
        vhost,
        "permissions",
        "declare",
        "--user",
        user_admin,
        "--configure",
        ".*",
        "--read",
        ".*",
        "--write",
        ".*",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "permissions",
        "declare",
        "--user",
        user_readonly,
        "--configure",
        "",
        "--read",
        ".*",
        "--write",
        "",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "permissions",
        "declare",
        "--user",
        user_limited,
        "--configure",
        "^limited\\..*",
        "--read",
        "^limited\\..*",
        "--write",
        "^limited\\..*",
    ]);

    run_succeeds(["permissions", "list"]).stdout(
        output_includes(user_admin)
            .and(output_includes(user_readonly))
            .and(output_includes(user_limited)),
    );

    run_succeeds([
        "--vhost",
        vhost,
        "permissions",
        "delete",
        "--user",
        user_admin,
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "permissions",
        "delete",
        "--user",
        user_readonly,
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "permissions",
        "delete",
        "--user",
        user_limited,
    ]);

    delete_user(user_admin)?;
    delete_user(user_readonly)?;
    delete_user(user_limited)?;
    delete_vhost(vhost)?;

    Ok(())
}

#[test]
fn test_multiple_policies_with_priorities() -> Result<(), Box<dyn Error>> {
    let vhost = "long.scenario.priority";
    let policy_low = "long.policy.low";
    let policy_medium = "long.policy.medium";
    let policy_high = "long.policy.high";

    delete_vhost(vhost).ok();
    create_vhost(vhost)?;

    run_succeeds([
        "--vhost",
        vhost,
        "policies",
        "declare",
        "--name",
        policy_low,
        "--pattern",
        ".*",
        "--definition",
        r#"{"max-length":1000}"#,
        "--priority",
        "1",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "policies",
        "declare",
        "--name",
        policy_medium,
        "--pattern",
        "^important\\..*",
        "--definition",
        r#"{"max-length":5000}"#,
        "--priority",
        "5",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "policies",
        "declare",
        "--name",
        policy_high,
        "--pattern",
        "^critical\\..*",
        "--definition",
        r#"{"max-length":10000}"#,
        "--priority",
        "10",
    ]);

    run_succeeds(["--vhost", vhost, "policies", "list"]).stdout(
        output_includes(policy_low)
            .and(output_includes(policy_medium))
            .and(output_includes(policy_high)),
    );

    run_succeeds(["--vhost", vhost, "policies", "delete", "--name", policy_low]);
    run_succeeds([
        "--vhost",
        vhost,
        "policies",
        "delete",
        "--name",
        policy_medium,
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "policies",
        "delete",
        "--name",
        policy_high,
    ]);

    delete_vhost(vhost)?;

    Ok(())
}

#[test]
fn test_idempotent_operations() -> Result<(), Box<dyn Error>> {
    let vhost = "long.scenario.idempotent";
    let user = "long.idempotent.user";
    let queue = "long.idempotent.queue";

    delete_vhost(vhost).ok();
    delete_user(user).ok();

    create_vhost(vhost)?;
    run_succeeds(["users", "declare", "--name", user, "--password", "pass"]);

    run_succeeds([
        "--vhost", vhost, "queues", "declare", "--name", queue, "--type", "classic",
    ]);

    run_succeeds([
        "--vhost",
        vhost,
        "queues",
        "delete",
        "--name",
        queue,
        "--idempotently",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "queues",
        "delete",
        "--name",
        queue,
        "--idempotently",
    ]);

    run_succeeds(["delete", "user", "--name", user, "--idempotently"]);
    run_succeeds(["delete", "user", "--name", user, "--idempotently"]);

    run_succeeds(["vhosts", "delete", "--name", vhost, "--idempotently"]);
    run_succeeds(["vhosts", "delete", "--name", vhost, "--idempotently"]);

    Ok(())
}

#[test]
fn test_combined_workflow_realistic_scenario() -> Result<(), Box<dyn Error>> {
    let vhost = "long.scenario.production";
    let admin_user = "long.prod.admin";
    let app_user = "long.prod.app";
    let monitoring_user = "long.prod.monitor";
    let orders_queue = "long.orders.queue";
    let events_queue = "long.events.queue";
    let orders_exchange = "long.orders.exchange";
    let events_exchange = "long.events.exchange";
    let ha_policy = "long.ha.policy";

    delete_vhost(vhost).ok();
    delete_user(admin_user).ok();
    delete_user(app_user).ok();
    delete_user(monitoring_user).ok();

    create_vhost(vhost)?;

    run_succeeds([
        "users",
        "declare",
        "--name",
        admin_user,
        "--password",
        "adminpass",
        "--tags",
        "administrator",
    ]);
    run_succeeds([
        "users",
        "declare",
        "--name",
        app_user,
        "--password",
        "apppass",
        "--tags",
        "management",
    ]);
    run_succeeds([
        "users",
        "declare",
        "--name",
        monitoring_user,
        "--password",
        "monitorpass",
        "--tags",
        "monitoring",
    ]);

    run_succeeds([
        "--vhost",
        vhost,
        "permissions",
        "declare",
        "--user",
        admin_user,
        "--configure",
        ".*",
        "--read",
        ".*",
        "--write",
        ".*",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "permissions",
        "declare",
        "--user",
        app_user,
        "--configure",
        "^long\\..*",
        "--read",
        "^long\\..*",
        "--write",
        "^long\\..*",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "permissions",
        "declare",
        "--user",
        monitoring_user,
        "--configure",
        "",
        "--read",
        ".*",
        "--write",
        "",
    ]);

    run_succeeds([
        "--vhost",
        vhost,
        "vhost_limits",
        "declare",
        "--limit-type",
        "max-connections",
        "--value",
        "1000",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "vhost_limits",
        "declare",
        "--limit-type",
        "max-queues",
        "--value",
        "500",
    ]);

    run_succeeds([
        "user_limits",
        "declare",
        "--user",
        app_user,
        "--limit-type",
        "max-connections",
        "--value",
        "50",
    ]);

    run_succeeds([
        "--vhost",
        vhost,
        "queues",
        "declare",
        "--name",
        orders_queue,
        "--type",
        "quorum",
        "--durable",
        "true",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "queues",
        "declare",
        "--name",
        events_queue,
        "--type",
        "classic",
        "--durable",
        "true",
    ]);

    run_succeeds([
        "--vhost",
        vhost,
        "exchanges",
        "declare",
        "--name",
        orders_exchange,
        "--type",
        "topic",
        "--durable",
        "true",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "exchanges",
        "declare",
        "--name",
        events_exchange,
        "--type",
        "fanout",
        "--durable",
        "true",
    ]);

    run_succeeds([
        "--vhost",
        vhost,
        "bindings",
        "declare",
        "--source",
        orders_exchange,
        "--destination-type",
        "queue",
        "--destination",
        orders_queue,
        "--routing-key",
        "order.#",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "bindings",
        "declare",
        "--source",
        events_exchange,
        "--destination-type",
        "queue",
        "--destination",
        events_queue,
    ]);

    run_succeeds([
        "--vhost",
        vhost,
        "policies",
        "declare",
        "--name",
        ha_policy,
        "--pattern",
        "^long\\..*",
        "--definition",
        r#"{"max-length":100000}"#,
        "--priority",
        "1",
    ]);

    await_queue_metric_emission();

    run_succeeds(["--vhost", vhost, "queues", "list"])
        .stdout(output_includes(orders_queue).and(output_includes(events_queue)));
    run_succeeds(["--vhost", vhost, "exchanges", "list"])
        .stdout(output_includes(orders_exchange).and(output_includes(events_exchange)));
    run_succeeds(["users", "list"]).stdout(
        output_includes(admin_user)
            .and(output_includes(app_user))
            .and(output_includes(monitoring_user)),
    );

    run_succeeds(["--vhost", vhost, "policies", "delete", "--name", ha_policy]);
    run_succeeds(["--vhost", vhost, "queues", "delete", "--name", orders_queue]);
    run_succeeds(["--vhost", vhost, "queues", "delete", "--name", events_queue]);
    run_succeeds([
        "--vhost",
        vhost,
        "exchanges",
        "delete",
        "--name",
        orders_exchange,
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "exchanges",
        "delete",
        "--name",
        events_exchange,
    ]);
    run_succeeds([
        "user_limits",
        "delete",
        "--user",
        app_user,
        "--limit-type",
        "max-connections",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "vhost_limits",
        "delete",
        "--limit-type",
        "max-connections",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "vhost_limits",
        "delete",
        "--limit-type",
        "max-queues",
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "permissions",
        "delete",
        "--user",
        admin_user,
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "permissions",
        "delete",
        "--user",
        app_user,
    ]);
    run_succeeds([
        "--vhost",
        vhost,
        "permissions",
        "delete",
        "--user",
        monitoring_user,
    ]);
    delete_user(admin_user)?;
    delete_user(app_user)?;
    delete_user(monitoring_user)?;
    delete_vhost(vhost)?;

    Ok(())
}
