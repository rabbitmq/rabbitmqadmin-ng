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
use predicates::prelude::*;

mod test_helpers;
use crate::test_helpers::*;

#[test]
fn list_exchanges() -> Result<(), Box<dyn std::error::Error>> {
    let vh1 = "exchange_vhost_1";
    let vh2 = "exchange_vhost_2";

    let x1 = "new_exchange_1";
    let x2 = "new_exchange_2";

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    // declare vhost 1
    run_succeeds(["declare", "vhost", "--name", vh1]);

    // declare vhost 2
    run_succeeds(["declare", "vhost", "--name", vh2]);

    // declare a new exchange in vhost 1
    run_succeeds(["-V", vh1, "declare", "exchange", "--name", x1]);

    // declare a new exchange in vhost 2
    run_succeeds(["-V", vh2, "declare", "exchange", "--name", x2]);

    // list exchanges in vhost 1
    run_succeeds(["-V", vh1, "list", "exchanges"]).stdout(
        predicate::str::contains("amq.direct")
            .and(predicate::str::contains("amq.fanout"))
            .and(predicate::str::contains(x1))
            .and(predicate::str::contains(x2).not()),
    );

    // delete the exchanges from vhost 1
    run_succeeds(["-V", vh1, "delete", "exchange", "--name", x1]);

    // list exchange in vhost 1
    run_succeeds(["-V", vh1, "list", "exchanges"]).stdout(
        predicate::str::contains("amq.direct")
            .and(predicate::str::contains("amq.topic"))
            .and(predicate::str::contains(x1).not()),
    );

    // list exchange in vhost 2
    run_succeeds(["-V", vh2, "list", "exchanges"]).stdout(
        predicate::str::contains("amq.direct")
            .and(predicate::str::contains("amq.headers"))
            .and(predicate::str::contains(x2))
            .and(predicate::str::contains(x1).not()),
    );

    delete_vhost(vh1).expect("failed to delete a virtual host");
    delete_vhost(vh2).expect("failed to delete a virtual host");

    Ok(())
}

#[test]
fn delete_an_existing_exchange() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "delete_exchange_vhost_1";
    let x = "exchange_1_to_delete";

    // create a vhost
    create_vhost(vh)?;

    // declare an exchange
    run_succeeds(["-V", vh, "declare", "exchange", "--name", x]);

    // list exchanges in vhost 1
    run_succeeds(["-V", vh, "list", "exchanges"]).stdout(predicate::str::contains(x));

    // delete the exchange
    run_succeeds(["-V", vh, "delete", "exchange", "--name", x]);

    // list exchange in vhost 1
    run_succeeds(["-V", vh, "list", "exchanges"]).stdout(predicate::str::contains(x).not());

    // delete the vhost
    delete_vhost(vh)?;

    Ok(())
}

#[test]
fn delete_a_non_existing_exchange() -> Result<(), Box<dyn std::error::Error>> {
    let vh = "delete_exchange_vhost_2";

    // declare a vhost
    create_vhost(vh)?;

    // try deleting a non-existent exchange with --idempotently
    run_succeeds([
        "--vhost",
        vh,
        "delete",
        "exchange",
        "--name",
        "7s98df7s79df-non-existent",
        "--idempotently",
    ]);

    // try deleting it without
    run_fails([
        "--vhost",
        vh,
        "delete",
        "exchange",
        "--name",
        "7s98df7s79df-non-existent",
    ])
    .stderr(predicate::str::contains("Not Found"));

    // delete the vhost
    delete_vhost(vh)?;

    Ok(())
}
