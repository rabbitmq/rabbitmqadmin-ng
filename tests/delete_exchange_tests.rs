// Copyright (C) 2023-2024 RabbitMQ Core Team (teamrabbitmq@gmail.com)
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
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

mod test_helpers;

#[test]
fn delete_an_existing_exchange() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "delete_exchange_vhost_1";
    let x = "exchange_1_to_delete";

    // create a vhost
    test_helpers::create_vhost(vh).unwrap();

    // declare an exchange
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg(vh)
        .arg("declare")
        .arg("exchange")
        .arg("--name")
        .arg(x);
    cmd.assert().success();

    // list exchanges in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.args(["-V", vh, "list", "exchanges"]);
    cmd.assert().success().stdout(predicate::str::contains(x));

    // delete the exchange
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg(vh)
        .arg("delete")
        .arg("exchange")
        .arg("--name")
        .arg(x);
    cmd.assert().success();

    // list exchange in vhost 1
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V").arg(vh).arg("list").arg("exchanges");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(x).not());

    // delete the vhost
    test_helpers::delete_vhost(vh).unwrap();

    Ok(())
}

#[test]
fn delete_a_non_existing_exchange() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "delete_exchange_vhost_2";

    // declare a vhost
    test_helpers::create_vhost(vh).unwrap();

    // try deleting a non-existent exchange with --idempotently
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg(vh)
        .arg("delete")
        .arg("exchange")
        .arg("--name")
        .arg("7s98df7s79df-non-existent")
        .arg("--idempotently");
    cmd.assert().success();

    // try deleting it without
    let mut cmd = Command::cargo_bin("rabbitmqadmin")?;
    cmd.arg("-V")
        .arg(vh)
        .arg("delete")
        .arg("exchange")
        .arg("--name")
        .arg("7s98df7s79df-non-existent");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Not Found"));

    // delete the vhost
    test_helpers::delete_vhost(vh).unwrap();

    Ok(())
}
