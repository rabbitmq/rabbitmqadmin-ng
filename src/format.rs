use rabbitmq_http_client::responses::Overview;
use tabled::settings::Panel;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct OverviewRow<'a> {
    key: &'a str,
    value: String,
}

pub fn overview_table(ov: Overview) -> Table {
    let data = vec![
        OverviewRow {
            key: "Product name",
            value: ov.product_name,
        },
        OverviewRow {
            key: "Product version",
            value: ov.product_version,
        },
        OverviewRow {
            key: "RabbitMQ version",
            value: ov.rabbitmq_version,
        },
        OverviewRow {
            key: "Erlang version",
            value: ov.erlang_version,
        },
        OverviewRow {
            key: "Erlang details",
            value: ov.erlang_full_version,
        },
    ];
    // TODO: if any tags are non-empty, add them to the table
    let tb = Table::builder(data);
    let mut t = tb.build();
    t.with(Panel::header("Overview"));
    t
}

pub fn churn_overview_table(ov: Overview) -> Table {
    let data = vec![
        OverviewRow {
            key: "Client connections opened",
            value: ov.churn_rates.connection_created.to_string(),
        },
        OverviewRow {
            key: "Client connections closed",
            value: ov.churn_rates.connection_closed.to_string(),
        },
        OverviewRow {
            key: "Client channels opened",
            value: ov.churn_rates.channel_created.to_string(),
        },
        OverviewRow {
            key: "Client channels closed",
            value: ov.churn_rates.channel_closed.to_string(),
        },
        OverviewRow {
            key: "Queues and streams (re)declarations",
            value: ov.churn_rates.queue_declared.to_string(),
        },
        OverviewRow {
            key: "Queues and streams created",
            value: ov.churn_rates.queue_created.to_string(),
        },
        OverviewRow {
            key: "Queues and streams deleted",
            value: ov.churn_rates.queue_deleted.to_string(),
        },
    ];
    let tb = Table::builder(data);
    let mut t = tb.build();
    t.with(Panel::header(
        "Entity (connections, queues, etc) churn over the most recent sample period",
    ));
    t
}
