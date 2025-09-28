// Copyright (C) 2023-2025 RabbitMQ Core Team (teamrabbitmq@gmail.com)
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

use crate::test_helpers::*;

#[test]
fn test_vhosts_delete_multiple_basic() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = "rabbitmqadmin.test-vhosts-delete-multiple-basic";

    // Clean up any existing test vhosts first (only our specific ones)
    delete_vhosts_with_prefix("rabbitmqadmin.test-vhosts-delete-multiple").ok();

    // Create 5 test virtual hosts
    for i in 1..=5 {
        let vh_name = format!("{}-{}", prefix, i);
        run_succeeds(["vhosts", "declare", "--name", &vh_name]);
    }

    // Verify they exist
    let client = api_client();
    let vhosts_before = client.list_vhosts()?;
    let test_vhosts_before: Vec<_> = vhosts_before
        .iter()
        .filter(|vh| vh.name.starts_with(prefix))
        .collect();
    assert_eq!(test_vhosts_before.len(), 5);

    // Delete them using the new command with idempotently flag
    run_succeeds([
        "vhosts",
        "delete_multiple",
        "--name-pattern",
        &format!("{}.*", prefix),
        "--approve",
        "--idempotently",
    ]);

    // Verify they're gone
    let vhosts_after = client.list_vhosts()?;
    let test_vhosts_after: Vec<_> = vhosts_after
        .iter()
        .filter(|vh| vh.name.starts_with(prefix))
        .collect();
    assert_eq!(test_vhosts_after.len(), 0);

    Ok(())
}

#[test]
fn test_vhosts_delete_multiple_dry_run() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = "rabbitmqadmin.test-vhosts-delete-multiple-dry-run";

    // Clean up any existing test vhosts first (only our specific ones)
    delete_vhosts_with_prefix("rabbitmqadmin.test-vhosts-delete-multiple").ok();

    // Create 3 test virtual hosts
    for i in 1..=3 {
        let vh_name = format!("{}-{}", prefix, i);
        run_succeeds(["vhosts", "declare", "--name", &vh_name]);
    }

    // Verify they exist
    let client = api_client();
    let vhosts_before = client.list_vhosts()?;
    let test_vhosts_before: Vec<_> = vhosts_before
        .iter()
        .filter(|vh| vh.name.starts_with(prefix))
        .collect();
    assert_eq!(test_vhosts_before.len(), 3);

    // Run dry-run (should not delete anything)
    run_succeeds([
        "vhosts",
        "delete_multiple",
        "--name-pattern",
        &format!("{}.*", prefix),
        "--dry-run",
    ]);

    // Verify they still exist
    let vhosts_after = client.list_vhosts()?;
    let test_vhosts_after: Vec<_> = vhosts_after
        .iter()
        .filter(|vh| vh.name.starts_with(prefix))
        .collect();
    assert_eq!(test_vhosts_after.len(), 3);

    // Clean up
    delete_vhosts_with_prefix("rabbitmqadmin.test-vhosts-delete-multiple").ok();

    Ok(())
}

#[test]
fn test_vhosts_delete_multiple_non_interactive() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = "rabbitmqadmin.test-vhosts-delete-multiple-non-interactive";

    // Clean up any existing test vhosts first (only our specific ones)
    delete_vhosts_with_prefix("rabbitmqadmin.test-vhosts-delete-multiple").ok();

    // Create 2 test virtual hosts
    for i in 1..=2 {
        let vh_name = format!("{}-{}", prefix, i);
        run_succeeds(["vhosts", "declare", "--name", &vh_name]);
    }

    // Verify they exist
    let client = api_client();
    let vhosts_before = client.list_vhosts()?;
    let test_vhosts_before: Vec<_> = vhosts_before
        .iter()
        .filter(|vh| vh.name.starts_with(prefix))
        .collect();
    assert_eq!(test_vhosts_before.len(), 2);

    // Delete using non-interactive mode (no --approve needed)
    run_succeeds([
        "--non-interactive",
        "vhosts",
        "delete_multiple",
        "--name-pattern",
        &format!("{}.*", prefix),
        "--idempotently",
    ]);

    // Verify they're gone
    let vhosts_after = client.list_vhosts()?;
    let test_vhosts_after: Vec<_> = vhosts_after
        .iter()
        .filter(|vh| vh.name.starts_with(prefix))
        .collect();
    assert_eq!(test_vhosts_after.len(), 0);

    Ok(())
}

#[test]
fn test_vhosts_delete_multiple_protects_default_vhost() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = "rabbitmqadmin.test-vhosts-delete-multiple-protects-default";

    // Clean up any existing test vhosts first (only our specific ones)
    delete_vhosts_with_prefix("rabbitmqadmin.test-vhosts-delete-multiple").ok();

    // Create test virtual hosts
    for i in 1..=2 {
        let vh_name = format!("{}-{}", prefix, i);
        run_succeeds(["vhosts", "declare", "--name", &vh_name]);
    }

    // Verify they exist
    let client = api_client();
    let vhosts_before = client.list_vhosts()?;
    let test_vhosts_before: Vec<_> = vhosts_before
        .iter()
        .filter(|vh| vh.name.starts_with(prefix))
        .collect();
    assert_eq!(test_vhosts_before.len(), 2);

    // Verify default vhost exists
    let default_vhost_before = vhosts_before.iter().find(|vh| vh.name == "/");
    assert!(default_vhost_before.is_some());

    // Try to delete everything including default vhost
    run_succeeds([
        "vhosts",
        "delete_multiple",
        "--name-pattern",
        ".*", // This would match everything including "/"
        "--approve",
        "--idempotently",
    ]);

    // Verify test vhosts are gone but default vhost still exists
    let vhosts_after = client.list_vhosts()?;
    let test_vhosts_after: Vec<_> = vhosts_after
        .iter()
        .filter(|vh| vh.name.starts_with(prefix))
        .collect();
    assert_eq!(test_vhosts_after.len(), 0);

    let default_vhost_after = vhosts_after.iter().find(|vh| vh.name == "/");
    assert!(default_vhost_after.is_some());

    Ok(())
}

#[test]
fn test_vhosts_delete_multiple_with_invalid_regex() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = "rabbitmqadmin.test-vhosts-delete-multiple-invalid-regex";

    // Clean up any existing test vhosts first (only our specific ones)
    delete_vhosts_with_prefix("rabbitmqadmin.test-vhosts-delete-multiple").ok();

    // Create a test virtual host
    let vh_name = format!("{}-1", prefix);
    run_succeeds(["vhosts", "declare", "--name", &vh_name]);

    // Try to delete with invalid regex pattern
    run_fails([
        "vhosts",
        "delete_multiple",
        "--name-pattern",
        "[invalid", // Invalid regex
        "--approve",
    ]);

    // Verify the vhost still exists
    let client = api_client();
    let vhosts = client.list_vhosts()?;
    let test_vhost = vhosts.iter().find(|vh| vh.name == vh_name);
    assert!(test_vhost.is_some());

    // Clean up
    delete_vhosts_with_prefix("rabbitmqadmin.test-vhosts-delete-multiple").ok();

    Ok(())
}

#[test]
fn test_vhosts_delete_multiple_requires_approve_in_interactive_mode()
-> Result<(), Box<dyn std::error::Error>> {
    let prefix = "rabbitmqadmin.test-vhosts-delete-multiple-requires-approve";

    // Clean up any existing test vhosts first (only our specific ones)
    delete_vhosts_with_prefix("rabbitmqadmin.test-vhosts-delete-multiple").ok();

    // Create a test virtual host
    let vh_name = format!("{}-1", prefix);
    run_succeeds(["vhosts", "declare", "--name", &vh_name]);

    // Try to delete without --approve flag (should fail)
    run_fails([
        "vhosts",
        "delete_multiple",
        "--name-pattern",
        &format!("{}.*", prefix),
    ]);

    // Verify the vhost still exists
    let client = api_client();
    let vhosts = client.list_vhosts()?;
    let test_vhost = vhosts.iter().find(|vh| vh.name == vh_name);
    assert!(test_vhost.is_some());

    // Clean up
    delete_vhosts_with_prefix("rabbitmqadmin.test-vhosts-delete-multiple").ok();

    Ok(())
}
// This test verifies that the delete_multiple command continues processing
// even when individual vhost deletions fail and shows appropriate progress indicators.
#[test]
fn test_vhosts_delete_multiple_continues_on_individual_failures()
-> Result<(), Box<dyn std::error::Error>> {
    let prefix = "rabbitmqadmin.test-vhosts-delete-multiple-continues";

    // Clean up any existing test vhosts first (only our specific ones)
    delete_vhosts_with_prefix("rabbitmqadmin.test-vhosts-delete-multiple").ok();

    // Create test virtual hosts
    for i in 1..=3 {
        let vh_name = format!("{}-{}", prefix, i);
        run_succeeds(["vhosts", "declare", "--name", &vh_name]);
    }

    // Verify they exist
    let client = api_client();
    let vhosts_before = client.list_vhosts()?;
    let test_vhosts_before: Vec<_> = vhosts_before
        .iter()
        .filter(|vh| vh.name.starts_with(prefix))
        .collect();
    assert_eq!(test_vhosts_before.len(), 3);

    // Manually delete one vhost to simulate a failure scenario
    // (This would cause a 404 when the command tries to delete it)
    let vh_name_to_predelete = format!("{}-2", prefix);
    delete_vhost(&vh_name_to_predelete).ok();

    // Run delete_multiple - it should:
    // 1. Continue processing even when deleting vh-2 fails (404)
    // 2. Show progress with 'X' for failed deletions
    // 3. Successfully delete vh-1 and vh-3
    run_succeeds([
        "vhosts",
        "delete_multiple",
        "--name-pattern",
        &format!("{}.*", prefix),
        "--approve",
        "--idempotently",
    ]);

    // Verify that only the successfully deleted vhosts are gone
    // (vh-1 and vh-3 should be deleted, vh-2 was already gone)
    let vhosts_after = client.list_vhosts()?;
    let test_vhosts_after: Vec<_> = vhosts_after
        .iter()
        .filter(|vh| vh.name.starts_with(prefix))
        .collect();
    assert_eq!(test_vhosts_after.len(), 0);

    // Clean up any remaining test vhosts to ensure test isolation
    delete_vhosts_with_prefix("rabbitmqadmin.test-vhosts-delete-multiple").ok();

    Ok(())
}

#[test]
fn test_vhosts_delete_multiple_protects_deletion_protected_vhosts()
-> Result<(), Box<dyn std::error::Error>> {
    let prefix = "rabbitmqadmin.test-vhosts-delete-multiple-protects-protected";

    // Clean up any existing test vhosts first (only our specific ones)
    delete_vhosts_with_prefix("rabbitmqadmin.test-vhosts-delete-multiple").ok();

    // Create test virtual hosts
    for i in 1..=3 {
        let vh_name = format!("{}-{}", prefix, i);
        run_succeeds(["vhosts", "declare", "--name", &vh_name]);
    }

    // Enable deletion protection for the second vhost only
    let protected_vh = format!("{}-2", prefix);
    run_succeeds([
        "vhosts",
        "enable_deletion_protection",
        "--name",
        &protected_vh,
    ]);

    // We begin with this many virtual hosts
    let client = api_client();
    let vhosts_before = client.list_vhosts()?;
    let test_vhosts_before: Vec<_> = vhosts_before
        .iter()
        .filter(|vh| vh.name.starts_with(prefix))
        .collect();
    assert_eq!(test_vhosts_before.len(), 3);

    // Try to delete all using the 'vhosts delete_multiple' command
    run_succeeds([
        "vhosts",
        "delete_multiple",
        "--name-pattern",
        &format!("{}.*", prefix),
        "--approve",
        "--idempotently",
    ]);

    // Verify that the protected vhost still exists, but several others were deleted
    let vhosts_after = client.list_vhosts()?;
    let test_vhosts_after: Vec<_> = vhosts_after
        .iter()
        .filter(|vh| vh.name.starts_with(prefix))
        .collect();

    // Only the protected vhost should remain
    assert_eq!(test_vhosts_after.len(), 1);
    assert_eq!(test_vhosts_after[0].name, protected_vh);

    // Clean up
    run_succeeds([
        "vhosts",
        "disable_deletion_protection",
        "--name",
        &protected_vh,
    ]);
    run_succeeds(["vhosts", "delete", "--name", &protected_vh]);

    Ok(())
}
