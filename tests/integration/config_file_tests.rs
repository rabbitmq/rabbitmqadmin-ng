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

use crate::test_helpers::{output_includes, run_fails, run_succeeds};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push("config_files");
    path.push(name);
    path
}

fn temp_config_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "rabbitmqadmin_test_config_{}.toml",
        std::process::id()
    ));
    path
}

#[test]
fn config_file_show_path_with_existing_file() -> Result<(), Box<dyn Error>> {
    let config_path = fixture_path("test_config.toml");
    let args = [
        "--config",
        config_path.to_str().unwrap(),
        "config_file",
        "show_path",
    ];
    run_succeeds(args).stdout(output_includes("Configuration file path"));
    Ok(())
}

#[test]
fn config_file_show_path_with_missing_file() -> Result<(), Box<dyn Error>> {
    let args = [
        "--config",
        "/nonexistent/path/to/config.toml",
        "config_file",
        "show_path",
    ];
    run_fails(args).stderr(output_includes("does not exist"));
    Ok(())
}

#[test]
fn config_file_show_with_existing_file() -> Result<(), Box<dyn Error>> {
    let config_path = fixture_path("test_config.toml");
    let args = [
        "--config",
        config_path.to_str().unwrap(),
        "config_file",
        "show",
    ];
    run_succeeds(args)
        .stdout(output_includes("local"))
        .stdout(output_includes("production"))
        .stdout(output_includes("staging"))
        .stdout(output_includes("********"));
    Ok(())
}

#[test]
fn config_file_show_with_reveal_passwords() -> Result<(), Box<dyn Error>> {
    let config_path = fixture_path("test_config.toml");
    let args = [
        "--config",
        config_path.to_str().unwrap(),
        "config_file",
        "show",
        "--reveal-passwords",
    ];
    run_succeeds(args)
        .stdout(output_includes("local"))
        .stdout(output_includes("guest"))
        .stdout(output_includes("secret123"))
        .stdout(output_includes("staging_pass"));
    Ok(())
}

#[test]
fn config_file_show_with_reveal_passwords_explicit_true() -> Result<(), Box<dyn Error>> {
    let config_path = fixture_path("test_config.toml");
    let args = [
        "--config",
        config_path.to_str().unwrap(),
        "config_file",
        "show",
        "--reveal-passwords",
        "true",
    ];
    run_succeeds(args)
        .stdout(output_includes("secret123"))
        .stdout(output_includes("staging_pass"));
    Ok(())
}

#[test]
fn config_file_show_with_missing_file() -> Result<(), Box<dyn Error>> {
    let args = [
        "--config",
        "/nonexistent/path/to/config.toml",
        "config_file",
        "show",
    ];
    run_fails(args).stderr(output_includes("does not exist"));
    Ok(())
}

#[test]
fn config_file_add_node_and_delete_node() -> Result<(), Box<dyn Error>> {
    let temp_path = temp_config_path();

    fs::write(&temp_path, "# Test config\n")?;

    let add_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "add_node",
        "--node",
        "test_node",
        "--host",
        "test.example.com",
        "--port",
        "15673",
        "--username",
        "test_user",
        "--password",
        "test_pass",
    ];
    run_succeeds(add_args);

    let show_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "show",
        "--reveal-passwords",
    ];
    run_succeeds(show_args)
        .stdout(output_includes("test_node"))
        .stdout(output_includes("test.example.com"))
        .stdout(output_includes("15673"))
        .stdout(output_includes("test_user"))
        .stdout(output_includes("test_pass"));

    let delete_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "delete_node",
        "--node",
        "test_node",
    ];
    run_succeeds(delete_args);

    let show_args_after = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "show",
    ];
    let output = run_succeeds(show_args_after);
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(!stdout.contains("test_node"));

    fs::remove_file(&temp_path)?;

    Ok(())
}

#[test]
fn config_file_add_node_with_defaults_hostname_as_name() -> Result<(), Box<dyn Error>> {
    let temp_path = temp_config_path();

    fs::write(&temp_path, "# Test config\n")?;

    let add_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "add_node",
        "--host",
        "my-rabbit-host",
    ];
    run_succeeds(add_args);

    let show_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "show",
    ];
    run_succeeds(show_args).stdout(output_includes("my-rabbit-host"));

    fs::remove_file(&temp_path)?;

    Ok(())
}

#[test]
fn config_file_add_node_with_missing_file_fails() -> Result<(), Box<dyn Error>> {
    let temp_path = temp_config_path();

    if temp_path.exists() {
        fs::remove_file(&temp_path)?;
    }

    let add_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "add_node",
        "--node",
        "new_node",
        "--host",
        "newhost.example.com",
    ];
    run_fails(add_args).stderr(output_includes("does not exist"));

    Ok(())
}

#[test]
fn config_file_add_node_creates_file_if_missing() -> Result<(), Box<dyn Error>> {
    let temp_path = temp_config_path();

    if temp_path.exists() {
        fs::remove_file(&temp_path)?;
    }

    let add_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "add_node",
        "--node",
        "new_node",
        "--host",
        "newhost.example.com",
        "--create-file-if-missing",
    ];
    run_succeeds(add_args);

    assert!(temp_path.exists());

    let show_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "show",
    ];
    run_succeeds(show_args)
        .stdout(output_includes("new_node"))
        .stdout(output_includes("newhost.example.com"));

    fs::remove_file(&temp_path)?;
    Ok(())
}

#[test]
fn config_file_delete_node_with_missing_file() -> Result<(), Box<dyn Error>> {
    let args = [
        "--config",
        "/nonexistent/path/to/config.toml",
        "config_file",
        "delete_node",
        "--node",
        "test_node",
    ];
    run_fails(args).stderr(output_includes("does not exist"));
    Ok(())
}

#[test]
fn config_file_delete_node_creates_file_if_missing() -> Result<(), Box<dyn Error>> {
    let temp_path = temp_config_path();

    if temp_path.exists() {
        fs::remove_file(&temp_path)?;
    }

    let delete_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "delete_node",
        "--node",
        "nonexistent_node",
        "--create-file-if-missing",
    ];
    run_succeeds(delete_args);

    assert!(temp_path.exists());

    fs::remove_file(&temp_path)?;
    Ok(())
}

#[test]
fn config_file_delete_node_nonexistent_succeeds() -> Result<(), Box<dyn Error>> {
    let temp_path = temp_config_path();

    fs::write(&temp_path, "[existing_node]\nhostname = \"localhost\"\n")?;

    let delete_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "delete_node",
        "--node",
        "nonexistent_node",
    ];
    run_succeeds(delete_args);

    let contents = fs::read_to_string(&temp_path)?;
    assert!(contents.contains("existing_node"));

    fs::remove_file(&temp_path)?;

    Ok(())
}

#[test]
fn config_file_show_help() -> Result<(), Box<dyn Error>> {
    let args = ["config_file", "--help"];
    run_succeeds(args)
        .stdout(output_includes("show_path"))
        .stdout(output_includes("show"))
        .stdout(output_includes("add_node"))
        .stdout(output_includes("update_node"))
        .stdout(output_includes("delete_node"));
    Ok(())
}

#[test]
fn config_file_show_displays_vhost() -> Result<(), Box<dyn Error>> {
    let config_path = fixture_path("test_config.toml");
    let args = [
        "--config",
        config_path.to_str().unwrap(),
        "config_file",
        "show",
    ];
    run_succeeds(args)
        .stdout(output_includes("/"))
        .stdout(output_includes("prod_vhost"))
        .stdout(output_includes("staging_vhost"));
    Ok(())
}

#[test]
fn config_file_show_displays_scheme() -> Result<(), Box<dyn Error>> {
    let config_path = fixture_path("test_config.toml");
    let args = [
        "--config",
        config_path.to_str().unwrap(),
        "config_file",
        "show",
    ];
    run_succeeds(args).stdout(output_includes("https"));
    Ok(())
}

#[test]
fn config_file_show_with_reveal_passwords_false() -> Result<(), Box<dyn Error>> {
    let config_path = fixture_path("test_config.toml");
    let args = [
        "--config",
        config_path.to_str().unwrap(),
        "config_file",
        "show",
        "--reveal-passwords",
        "false",
    ];
    run_succeeds(args).stdout(output_includes("********"));
    Ok(())
}

#[test]
fn config_file_show_empty_config() -> Result<(), Box<dyn Error>> {
    let temp_path = temp_config_path();
    fs::write(&temp_path, "# Empty config\n")?;

    let args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "show",
    ];
    run_succeeds(args);

    fs::remove_file(&temp_path)?;
    Ok(())
}

#[test]
fn config_file_add_node_fails_if_exists() -> Result<(), Box<dyn Error>> {
    let temp_path = temp_config_path();

    fs::write(
        &temp_path,
        "[test_node]\nhostname = \"old.example.com\"\nport = 1111\n",
    )?;

    let add_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "add_node",
        "--node",
        "test_node",
        "--host",
        "new.example.com",
        "--port",
        "2222",
    ];
    run_fails(add_args).stderr(output_includes("already exists"));

    let contents = fs::read_to_string(&temp_path)?;
    assert!(contents.contains("old.example.com"));
    assert!(contents.contains("1111"));

    fs::remove_file(&temp_path)?;
    Ok(())
}

#[test]
fn config_file_update_node_with_missing_file_fails() -> Result<(), Box<dyn Error>> {
    let temp_path = temp_config_path();

    if temp_path.exists() {
        fs::remove_file(&temp_path)?;
    }

    let update_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "update_node",
        "--node",
        "new_node",
        "--host",
        "newhost.example.com",
    ];
    run_fails(update_args).stderr(output_includes("does not exist"));

    Ok(())
}

#[test]
fn config_file_update_node_creates_new() -> Result<(), Box<dyn Error>> {
    let temp_path = temp_config_path();

    fs::write(&temp_path, "# Test config\n")?;

    let update_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "update_node",
        "--node",
        "new_node",
        "--host",
        "newhost.example.com",
        "--port",
        "15673",
    ];
    run_succeeds(update_args);

    let show_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "show",
    ];
    run_succeeds(show_args)
        .stdout(output_includes("new_node"))
        .stdout(output_includes("newhost.example.com"))
        .stdout(output_includes("15673"));

    fs::remove_file(&temp_path)?;
    Ok(())
}

#[test]
fn config_file_update_node_merges_with_existing() -> Result<(), Box<dyn Error>> {
    let temp_path = temp_config_path();

    fs::write(
        &temp_path,
        "[test_node]\nhostname = \"old.example.com\"\nport = 1111\nusername = \"admin\"\npassword = \"secret\"\n",
    )?;

    // Only update hostname and port, username and password should be preserved
    let update_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "update_node",
        "--node",
        "test_node",
        "--host",
        "new.example.com",
        "--port",
        "2222",
    ];
    run_succeeds(update_args);

    let show_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "show",
        "--reveal-passwords",
    ];
    run_succeeds(show_args)
        .stdout(output_includes("new.example.com"))
        .stdout(output_includes("2222"))
        .stdout(output_includes("admin"))
        .stdout(output_includes("secret"));

    let contents = fs::read_to_string(&temp_path)?;
    assert!(
        !contents.contains("old.example.com"),
        "old hostname should be replaced"
    );
    assert!(!contents.contains("1111"), "old port should be replaced");
    assert!(contents.contains("admin"), "username should be preserved");
    assert!(contents.contains("secret"), "password should be preserved");

    fs::remove_file(&temp_path)?;
    Ok(())
}

#[test]
fn config_file_update_node_creates_file_if_missing() -> Result<(), Box<dyn Error>> {
    let temp_path = temp_config_path();

    if temp_path.exists() {
        fs::remove_file(&temp_path)?;
    }

    let update_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "update_node",
        "--node",
        "new_node",
        "--host",
        "newhost.example.com",
        "--create-file-if-missing",
    ];
    run_succeeds(update_args);

    assert!(temp_path.exists());

    let show_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "show",
    ];
    run_succeeds(show_args)
        .stdout(output_includes("new_node"))
        .stdout(output_includes("newhost.example.com"));

    fs::remove_file(&temp_path)?;
    Ok(())
}

#[test]
fn config_file_add_node_with_vhost() -> Result<(), Box<dyn Error>> {
    let temp_path = temp_config_path();

    fs::write(&temp_path, "# Test config\n")?;

    let add_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "add_node",
        "--node",
        "vhost_test",
        "--host",
        "localhost",
        "--vhost",
        "my_vhost",
    ];
    run_succeeds(add_args);

    let show_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "show",
    ];
    run_succeeds(show_args).stdout(output_includes("my_vhost"));

    fs::remove_file(&temp_path)?;
    Ok(())
}

#[test]
fn config_file_add_node_with_base_uri() -> Result<(), Box<dyn Error>> {
    let temp_path = temp_config_path();

    fs::write(&temp_path, "# Test config\n")?;

    let add_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "add_node",
        "--node",
        "uri_test",
        "--base-uri",
        "https://rabbit.example.com:15673/api",
    ];
    run_succeeds(add_args);

    let contents = fs::read_to_string(&temp_path)?;
    assert!(contents.contains("base_uri"));
    assert!(contents.contains("https://rabbit.example.com:15673/api"));

    fs::remove_file(&temp_path)?;
    Ok(())
}

#[test]
fn config_file_add_node_with_scheme() -> Result<(), Box<dyn Error>> {
    let temp_path = temp_config_path();

    fs::write(&temp_path, "# Test config\n")?;

    let add_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "add_node",
        "--node",
        "tls_node",
        "--host",
        "rabbitmq.eng.megacorp.local",
        "--port",
        "15671",
        "--scheme",
        "https",
    ];
    run_succeeds(add_args);

    let contents = fs::read_to_string(&temp_path)?;
    assert!(contents.contains("scheme"));
    assert!(contents.contains("https"));
    assert!(contents.contains("rabbitmq.eng.megacorp.local"));

    fs::remove_file(&temp_path)?;
    Ok(())
}

#[test]
fn config_file_add_node_with_path_prefix() -> Result<(), Box<dyn Error>> {
    let temp_path = temp_config_path();

    fs::write(&temp_path, "# Test config\n")?;

    let add_args = [
        "--config",
        temp_path.to_str().unwrap(),
        "config_file",
        "add_node",
        "--node",
        "prefixed_node",
        "--host",
        "localhost",
        "--path-prefix",
        "/rabbitmq/api",
    ];
    run_succeeds(add_args);

    let contents = fs::read_to_string(&temp_path)?;
    assert!(contents.contains("path_prefix"));
    assert!(contents.contains("/rabbitmq/api"));

    fs::remove_file(&temp_path)?;
    Ok(())
}
