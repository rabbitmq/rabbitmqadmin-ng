use rabbitmq_http_client::responses::Overview;
use tabled::{Table, Tabled};
use tabled::settings::Panel;

#[derive(Tabled)]
struct OverviewRow<'a> {
    key: &'a str,
    value: String,
}

pub fn overview_table(ov: Overview) -> Table {
    let data = vec![
        OverviewRow { key: "Product name", value: ov.product_name },
        OverviewRow { key: "Product version", value: ov.product_version },
        OverviewRow { key: "RabbitMQ version", value: ov.rabbitmq_version },
        OverviewRow { key: "Erlang version", value: ov.erlang_version },
        OverviewRow { key: "Erlang details", value: ov.erlang_full_version },
    ];
    // TODO: if any tags are non-empty, add them to the table
    let tb = Table::builder(data);
    let mut t= tb.build();
    t.with(Panel::header("Overview"));
    t
}
