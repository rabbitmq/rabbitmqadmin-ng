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
#![allow(clippy::result_large_err)]

use crate::APIClient;
use crate::arg_helpers::ArgMatchesExt;
use crate::commands;
use crate::errors::CommandRunError;
use crate::output::ResultHandler;
use clap::ArgMatches;
use rabbitmq_http_client::commons::PolicyTarget;
use sysexits::ExitCode;

pub fn dispatch_command_group(
    first_level: &str,
    second_level: &str,
    args: &ArgMatches,
    client: APIClient,
    endpoint: String,
    vhost: String,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match first_level {
        "auth_attempts" => dispatch_auth_attempts(second_level, args, client, res_handler),
        "bindings" => dispatch_bindings(second_level, args, client, &vhost, res_handler),
        "channels" => dispatch_channels(second_level, args, client, res_handler),
        "close" => dispatch_close(second_level, args, client, res_handler),
        "connections" => dispatch_connections(second_level, args, client, res_handler),
        "declare" => dispatch_declare(second_level, args, client, &vhost, res_handler),
        "definitions" => dispatch_definitions(second_level, args, client, &vhost, res_handler),
        "delete" => dispatch_delete(second_level, args, client, &vhost, res_handler),
        "deprecated_features" => dispatch_deprecated_features(second_level, client, res_handler),
        "exchanges" => dispatch_exchanges(second_level, args, client, &vhost, res_handler),
        "export" => dispatch_export(second_level, args, client, res_handler),
        "feature_flags" => dispatch_feature_flags(second_level, args, client, res_handler),
        "federation" => dispatch_federation(second_level, args, client, &vhost, res_handler),
        "get" => dispatch_get(second_level, args, client, &vhost, res_handler),
        "global_parameters" => dispatch_global_parameters(second_level, args, client, res_handler),
        "health_check" => dispatch_health_check(second_level, args, client, res_handler),
        "import" => dispatch_import(second_level, args, client, res_handler),
        "list" => dispatch_list(second_level, args, client, &vhost, res_handler),
        "nodes" => dispatch_nodes(second_level, args, client, res_handler),
        "operator_policies" => {
            dispatch_operator_policies(second_level, args, client, &vhost, res_handler)
        }
        "parameters" => dispatch_parameters(second_level, args, client, &vhost, res_handler),
        "passwords" => dispatch_passwords(second_level, args, res_handler),
        "permissions" => dispatch_permissions(second_level, args, client, &vhost, res_handler),
        "plugins" => dispatch_plugins(second_level, args, client, res_handler),
        "policies" => dispatch_policies(second_level, args, client, &vhost, res_handler),
        "publish" => dispatch_publish(second_level, args, client, &vhost, res_handler),
        "purge" => dispatch_purge(second_level, args, client, &vhost, res_handler),
        "queues" => dispatch_queues(second_level, args, client, &vhost, res_handler),
        "rebalance" => dispatch_rebalance(second_level, client, res_handler),
        "show" => dispatch_show(second_level, args, client, &endpoint, res_handler),
        "shovels" => dispatch_shovels(second_level, args, client, &vhost, res_handler),
        "streams" => dispatch_streams(second_level, args, client, &vhost, res_handler),
        "users" => dispatch_users(second_level, args, client, res_handler),
        "user_limits" => dispatch_user_limits(second_level, args, client, res_handler),
        "vhosts" => dispatch_vhosts(second_level, args, client, res_handler),
        "vhost_limits" => dispatch_vhost_limits(second_level, args, client, &vhost, res_handler),
        _ => unknown_subcommand(first_level, second_level, res_handler),
    }
}

fn unknown_subcommand(
    command: &str,
    subcommand: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    let error = CommandRunError::UnknownCommandTarget {
        command: command.into(),
        subcommand: subcommand.into(),
    };
    res_handler.report_pre_command_run_error(&error);
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_auth_attempts(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "stats" => {
            let result = commands::list_auth_attempts(client, args);
            res_handler.tabular_result(result);
        }
        _ => return unknown_subcommand("auth_attempts", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_bindings(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "declare" => {
            let result = commands::declare_binding(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "delete" => {
            let result = commands::delete_binding(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "list" => {
            let result = commands::list_bindings(client);
            res_handler.tabular_result(result);
        }
        _ => return unknown_subcommand("bindings", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_channels(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "list" => {
            let result = commands::list_channels(client, args);
            res_handler.tabular_result(result);
        }
        _ => return unknown_subcommand("channels", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_close(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "connection" => {
            let result = commands::close_connection(client, args);
            res_handler.no_output_on_success(result);
        }
        "user_connections" => {
            let result = commands::close_user_connections(client, args);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("close", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_connections(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "close" => {
            let result = commands::close_connection(client, args);
            res_handler.no_output_on_success(result);
        }
        "close_of_user" => {
            let result = commands::close_user_connections(client, args);
            res_handler.no_output_on_success(result);
        }
        "list" => {
            let result = commands::list_connections(client, args);
            res_handler.tabular_result(result);
        }
        "list_of_user" => {
            let result = commands::list_user_connections(client, args);
            res_handler.tabular_result(result);
        }
        _ => return unknown_subcommand("connections", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_declare(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "binding" => {
            let result = commands::declare_binding(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "exchange" => {
            let result = commands::declare_exchange(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "operator_policy" => {
            let result = commands::declare_operator_policy(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "parameter" => {
            let result = commands::declare_parameter(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "permissions" => {
            let result = commands::declare_permissions(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "policy" => {
            let result = commands::declare_policy(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "queue" => {
            let result = commands::declare_queue(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "stream" => {
            let result = commands::declare_stream(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "user" => {
            let result = commands::declare_user(client, args);
            res_handler.no_output_on_success(result);
        }
        "user_limit" => {
            let result = commands::declare_user_limit(client, args);
            res_handler.no_output_on_success(result);
        }
        "vhost" => {
            let result = commands::declare_vhost(client, args);
            res_handler.no_output_on_success(result);
        }
        "vhost_limit" => {
            let result = commands::declare_vhost_limit(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("declare", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_definitions(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "export" => {
            let result = commands::export_cluster_wide_definitions(client, args);
            res_handler.no_output_on_success(result);
        }
        "export_from_vhost" => {
            let result = commands::export_vhost_definitions(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "import" => {
            let result = commands::import_definitions(client, args);
            res_handler.no_output_on_success(result);
        }
        "import_into_vhost" => {
            let result = commands::import_vhost_definitions(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("definitions", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_delete(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "binding" => {
            let result = commands::delete_binding(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "exchange" => {
            let result = commands::delete_exchange(client, vhost, args);
            res_handler.delete_operation_result(result);
        }
        "operator_policy" => {
            let result = commands::delete_operator_policy(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "parameter" => {
            let result = commands::delete_parameter(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "permissions" => {
            let result = commands::delete_permissions(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "policy" => {
            let result = commands::delete_policy(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "queue" => {
            let result = commands::delete_queue(client, vhost, args);
            res_handler.delete_operation_result(result);
        }
        "shovel" => {
            let result = commands::delete_shovel(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "stream" => {
            let result = commands::delete_stream(client, vhost, args);
            res_handler.delete_operation_result(result);
        }
        "user" => {
            let result = commands::delete_user(client, args);
            res_handler.delete_operation_result(result);
        }
        "user_limit" => {
            let result = commands::delete_user_limit(client, args);
            res_handler.no_output_on_success(result);
        }
        "vhost" => {
            let result = commands::delete_vhost(client, args);
            res_handler.delete_operation_result(result);
        }
        "vhost_limit" => {
            let result = commands::delete_vhost_limit(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("delete", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_deprecated_features(
    subcommand: &str,
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "list" => {
            let result = commands::list_deprecated_features(client);
            res_handler.tabular_result(result.map(|val| val.0));
        }
        "list_used" => {
            let result = commands::list_deprecated_features_in_use(client);
            res_handler.tabular_result(result.map(|val| val.0));
        }
        _ => return unknown_subcommand("deprecated_features", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_exchanges(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "bind" => {
            let result = commands::declare_binding(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "declare" => {
            let result = commands::declare_exchange(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "delete" => {
            let result = commands::delete_exchange(client, vhost, args);
            res_handler.delete_operation_result(result);
        }
        "list" => {
            let result = commands::list_exchanges(client, vhost, args);
            res_handler.tabular_result(result);
        }
        "unbind" => {
            let result = commands::delete_binding(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("exchanges", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_export(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "definitions" => {
            let result = commands::export_cluster_wide_definitions(client, args);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("export", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_feature_flags(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "enable" => {
            let result = commands::enable_feature_flag(client, args);
            res_handler.no_output_on_success(result);
        }
        "enable_all" => {
            let result = commands::enable_all_stable_feature_flags(client);
            res_handler.no_output_on_success(result);
        }
        "list" => {
            let result = commands::list_feature_flags(client);
            res_handler.tabular_result(result.map(|val| val.0));
        }
        _ => return unknown_subcommand("feature_flags", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_federation(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "declare_upstream" => {
            let result = commands::declare_federation_upstream(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "declare_upstream_for_exchanges" => {
            let result =
                commands::declare_federation_upstream_for_exchange_federation(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "declare_upstream_for_queues" => {
            let result =
                commands::declare_federation_upstream_for_queue_federation(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "delete_upstream" => {
            let result = commands::delete_federation_upstream(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "list_all_links" => {
            let result = commands::list_federation_links(client);
            res_handler.tabular_result(result);
        }
        "list_all_upstreams" => {
            let result = commands::list_federation_upstreams(client);
            res_handler.tabular_result(result);
        }
        "disable_tls_peer_verification_for_all_upstreams" => {
            let mut prog_rep = res_handler.instantiate_progress_reporter();
            let result = commands::disable_tls_peer_verification_for_all_federation_upstreams(
                client,
                prog_rep.as_mut(),
            );
            res_handler.no_output_on_success(result);
        }
        "enable_tls_peer_verification_for_all_upstreams" => {
            let mut prog_rep = res_handler.instantiate_progress_reporter();
            let result = commands::enable_tls_peer_verification_for_all_federation_upstreams(
                client,
                args,
                prog_rep.as_mut(),
            );
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("federation", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_get(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "messages" => {
            let result = commands::get_messages(client, vhost, args);
            res_handler.tabular_result(result);
        }
        _ => return unknown_subcommand("get", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_global_parameters(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "clear" => {
            let result = commands::delete_global_parameter(client, args);
            res_handler.no_output_on_success(result);
        }
        "list" => {
            let result = commands::list_global_parameters(client);
            res_handler.tabular_result(result);
        }
        "set" => {
            let result = commands::declare_global_parameter(client, args);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("global_parameters", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_health_check(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "cluster_wide_alarms" => {
            let result = commands::health_check_cluster_wide_alarms(client);
            res_handler.health_check_result(result);
        }
        "local_alarms" => {
            let result = commands::health_check_local_alarms(client);
            res_handler.health_check_result(result);
        }
        "node_is_quorum_critical" => {
            let result = commands::health_check_node_is_quorum_critical(client);
            res_handler.health_check_result(result);
        }
        "port_listener" => {
            let result = commands::health_check_port_listener(client, args);
            res_handler.health_check_result(result);
        }
        "protocol_listener" => {
            let result = commands::health_check_protocol_listener(client, args);
            res_handler.health_check_result(result);
        }
        _ => return unknown_subcommand("health_check", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_import(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "definitions" => {
            let result = commands::import_definitions(client, args);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("import", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_list(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "bindings" => {
            let result = commands::list_bindings(client);
            res_handler.tabular_result(result);
        }
        "channels" => {
            let result = commands::list_channels(client, args);
            res_handler.tabular_result(result);
        }
        "connections" => {
            let result = commands::list_connections(client, args);
            res_handler.tabular_result(result);
        }
        "consumers" => {
            let result = commands::list_consumers(client);
            res_handler.tabular_result(result);
        }
        "deprecated_features" => {
            let result = commands::list_deprecated_features(client);
            res_handler.tabular_result(result.map(|val| val.0));
        }
        "deprecated_features_in_use" => {
            let result = commands::list_deprecated_features_in_use(client);
            res_handler.tabular_result(result.map(|val| val.0));
        }
        "exchanges" => {
            let result = commands::list_exchanges(client, vhost, args);
            res_handler.tabular_result(result);
        }
        "feature_flags" => {
            let result = commands::list_feature_flags(client);
            res_handler.tabular_result(result.map(|val| val.0));
        }
        "nodes" => {
            let result = commands::list_nodes(client);
            res_handler.tabular_result(result);
        }
        "operator_policies" => {
            let result = commands::list_operator_policies(client);
            res_handler.tabular_result(result);
        }
        "parameters" => {
            let result = commands::list_parameters(client, vhost, args);
            res_handler.tabular_result(result);
        }
        "permissions" => {
            let result = commands::list_permissions(client);
            res_handler.tabular_result(result);
        }
        "policies" => {
            let result = commands::list_policies(client);
            res_handler.tabular_result(result);
        }
        "queues" => {
            let result = commands::list_queues(client, vhost, args);
            res_handler.tabular_result(result);
        }
        "user_connections" => {
            let result = commands::list_user_connections(client, args);
            res_handler.tabular_result(result);
        }
        "user_limits" => {
            let result = commands::list_user_limits(client, args);
            res_handler.tabular_result(result);
        }
        "users" => {
            let result = commands::list_users(client, args);
            res_handler.tabular_result(result);
        }
        "vhost_limits" => {
            let result = commands::list_vhost_limits(client, vhost);
            res_handler.tabular_result(result);
        }
        "vhosts" => {
            let result = commands::list_vhosts(client);
            res_handler.tabular_result(result);
        }
        _ => return unknown_subcommand("list", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_nodes(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "list" => {
            let result = commands::list_nodes(client);
            res_handler.tabular_result(result);
        }
        "memory_breakdown_in_bytes" => {
            let result = commands::show_memory_breakdown(client, args);
            res_handler.memory_breakdown_in_bytes_result(result);
        }
        "memory_breakdown_in_percent" => {
            let result = commands::show_memory_breakdown(client, args);
            res_handler.memory_breakdown_in_percent_result(result);
        }
        _ => return unknown_subcommand("nodes", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_operator_policies(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "declare" => {
            let result = commands::declare_operator_policy(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "delete" => {
            let result = commands::delete_operator_policy(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "delete_definition_keys" => {
            let result = commands::delete_operator_policy_definition_keys(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "delete_definition_keys_from_all_in" => {
            let result = commands::delete_operator_policy_definition_keys_in(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "list" => {
            let result = commands::list_operator_policies(client);
            res_handler.tabular_result(result);
        }
        "list_in" => {
            let typ_opt = args.get_one::<PolicyTarget>("apply_to").cloned();
            let result = match typ_opt {
                None => commands::list_operator_policies_in(client, vhost),
                Some(typ) => {
                    commands::list_operator_policies_in_and_applying_to(client, vhost, typ)
                }
            };
            res_handler.tabular_result(result);
        }
        "list_matching_object" => {
            let name = args.string_arg("name");
            let typ = args.get_one::<PolicyTarget>("type").cloned().unwrap();
            let result = commands::list_matching_operator_policies_in(client, vhost, &name, typ);
            res_handler.tabular_result(result);
        }
        "patch" => {
            let result = commands::patch_operator_policy_definition(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "update_definition" => {
            let result = commands::update_operator_policy_definition(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "update_definitions_of_all_in" => {
            let result = commands::update_all_operator_policy_definitions_in(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("operator_policies", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_parameters(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "clear" => {
            let result = commands::delete_parameter(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "list_all" => {
            let result = commands::list_all_parameters(client);
            res_handler.tabular_result(result);
        }
        "list" => {
            let result = commands::list_parameters(client, vhost, args);
            res_handler.tabular_result(result);
        }
        "list_in" => {
            let result = commands::list_parameters_of_component_in(client, vhost, args);
            res_handler.tabular_result(result);
        }
        "set" => {
            let result = commands::declare_parameter(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("parameters", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_passwords(
    subcommand: &str,
    args: &ArgMatches,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "salt_and_hash" => {
            let result = commands::salt_and_hash_password(args);
            res_handler.show_salted_and_hashed_value(result);
        }
        _ => return unknown_subcommand("passwords", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_permissions(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "list" => {
            let result = commands::list_permissions(client);
            res_handler.tabular_result(result);
        }
        "declare" => {
            let result = commands::declare_permissions(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "delete" => {
            let result = commands::delete_permissions(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("permissions", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_plugins(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "list_all" => {
            let result = commands::list_plugins_across_cluster(client);
            res_handler.tabular_result(result);
        }
        "list_on_node" => {
            let result = commands::list_plugins_on_node(client, args);
            res_handler.tabular_result(result);
        }
        _ => return unknown_subcommand("plugins", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_policies(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "declare" => {
            let result = commands::declare_policy(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "declare_override" => {
            let result = commands::declare_policy_override(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "declare_blanket" => {
            let result = commands::declare_blanket_policy(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "delete" => {
            let result = commands::delete_policy(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "delete_definition_keys" => {
            let result = commands::delete_policy_definition_keys(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "delete_definition_keys_from_all_in" => {
            let result = commands::delete_policy_definition_keys_in(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "list" => {
            let result = commands::list_policies(client);
            res_handler.tabular_result(result);
        }
        "list_conflicting" => {
            let result = commands::list_policies_with_conflicting_priorities(client);
            res_handler.tabular_result(result);
        }
        "list_conflicting_in" => {
            let result = commands::list_policies_with_conflicting_priorities_in(client, vhost);
            res_handler.tabular_result(result);
        }
        "list_in" => {
            let typ_opt = args.get_one::<PolicyTarget>("apply_to").cloned();
            let result = match typ_opt {
                None => commands::list_policies_in(client, vhost),
                Some(typ) => commands::list_policies_in_and_applying_to(client, vhost, typ),
            };
            res_handler.tabular_result(result);
        }
        "list_matching_object" => {
            let name = args.string_arg("name");
            let typ = args.get_one::<PolicyTarget>("type").cloned().unwrap();
            let result = commands::list_matching_policies_in(client, vhost, &name, typ);
            res_handler.tabular_result(result);
        }
        "patch" => {
            let result = commands::patch_policy_definition(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "update_definition" => {
            let result = commands::update_policy_definition(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "update_definitions_of_all_in" => {
            let result = commands::update_all_policy_definitions_in(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("policies", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_publish(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "message" => {
            let result = commands::publish_message(client, vhost, args);
            res_handler.single_value_output_with_result(result);
        }
        _ => return unknown_subcommand("publish", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_purge(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "queue" => {
            let result = commands::purge_queue(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("purge", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_queues(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "declare" => {
            let result = commands::declare_queue(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "delete" => {
            let result = commands::delete_queue(client, vhost, args);
            res_handler.delete_operation_result(result);
        }
        "list" => {
            let result = commands::list_queues(client, vhost, args);
            res_handler.tabular_result(result);
        }
        "purge" => {
            let result = commands::purge_queue(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "rebalance" => {
            let result = commands::rebalance_queues(client);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("queues", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_rebalance(
    subcommand: &str,
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "queues" => {
            let result = commands::rebalance_queues(client);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("rebalance", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_show(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    endpoint: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "churn" => {
            let result = commands::show_overview(client);
            res_handler.show_churn(result);
        }
        "endpoint" => {
            println!("Using endpoint: {}", endpoint);
            res_handler.no_output_on_success(Ok(()));
        }
        "memory_breakdown_in_bytes" => {
            let result = commands::show_memory_breakdown(client, args);
            res_handler.memory_breakdown_in_bytes_result(result);
        }
        "memory_breakdown_in_percent" => {
            let result = commands::show_memory_breakdown(client, args);
            res_handler.memory_breakdown_in_percent_result(result);
        }
        "overview" => {
            let result = commands::show_overview(client);
            res_handler.show_overview(result);
        }
        _ => return unknown_subcommand("show", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_shovels(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "declare_amqp091" => {
            let source_queue = args.get_one::<String>("source_queue").cloned();
            let source_exchange = args.get_one::<String>("source_exchange").cloned();
            let destination_queue = args.get_one::<String>("destination_queue").cloned();
            let destination_exchange = args.get_one::<String>("destination_exchange").cloned();

            if source_queue.is_none() && source_exchange.is_none() {
                let err = CommandRunError::MissingOptions {
                    message: "either --source-queue or --source-exchange must be provided"
                        .to_string(),
                };
                res_handler.report_pre_command_run_error(&err);
            } else if destination_queue.is_none() && destination_exchange.is_none() {
                let err = CommandRunError::MissingOptions {
                    message:
                        "either --destination-queue or --destination-exchange must be provided"
                            .to_string(),
                };
                res_handler.report_pre_command_run_error(&err);
            } else {
                let result = commands::declare_amqp091_shovel(client, vhost, args);
                res_handler.no_output_on_success(result);
            }
        }
        "declare_amqp10" => {
            let result = commands::declare_amqp10_shovel(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "delete" => {
            let result = commands::delete_shovel(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "list_all" => {
            let result = commands::list_shovels(client);
            res_handler.tabular_result(result);
        }
        "list" => {
            let result = commands::list_shovels_in(client, vhost);
            res_handler.tabular_result(result);
        }
        "disable_tls_peer_verification_for_all_source_uris" => {
            let mut prog_rep = res_handler.instantiate_progress_reporter();
            let result = commands::disable_tls_peer_verification_for_all_source_uris(
                client,
                prog_rep.as_mut(),
            );
            res_handler.no_output_on_success(result);
        }
        "disable_tls_peer_verification_for_all_destination_uris" => {
            let mut prog_rep = res_handler.instantiate_progress_reporter();
            let result = commands::disable_tls_peer_verification_for_all_destination_uris(
                client,
                prog_rep.as_mut(),
            );
            res_handler.no_output_on_success(result);
        }
        "enable_tls_peer_verification_for_all_source_uris" => {
            let mut prog_rep = res_handler.instantiate_progress_reporter();
            let result = commands::enable_tls_peer_verification_for_all_source_uris(
                client,
                args,
                prog_rep.as_mut(),
            );
            res_handler.no_output_on_success(result);
        }
        "enable_tls_peer_verification_for_all_destination_uris" => {
            let mut prog_rep = res_handler.instantiate_progress_reporter();
            let result = commands::enable_tls_peer_verification_for_all_destination_uris(
                client,
                args,
                prog_rep.as_mut(),
            );
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("shovels", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_streams(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "declare" => {
            let result = commands::declare_stream(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "delete" => {
            let result = commands::delete_queue(client, vhost, args);
            res_handler.delete_operation_result(result);
        }
        "list" => {
            let result = commands::list_queues(client, vhost, args);
            res_handler.tabular_result(result);
        }
        _ => return unknown_subcommand("streams", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_users(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "connections" => {
            let result = commands::list_user_connections(client, args);
            res_handler.tabular_result(result);
        }
        "declare" => {
            let result = commands::declare_user(client, args);
            res_handler.no_output_on_success(result);
        }
        "delete" => {
            let result = commands::delete_user(client, args);
            res_handler.delete_operation_result(result);
        }
        "limits" => {
            let result = commands::list_user_limits(client, args);
            res_handler.tabular_result(result);
        }
        "list" => {
            let result = commands::list_users(client, args);
            res_handler.tabular_result(result);
        }
        "permissions" => {
            let result = commands::list_permissions(client);
            res_handler.tabular_result(result);
        }
        _ => return unknown_subcommand("users", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_user_limits(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "list" => {
            let result = commands::list_user_limits(client, args);
            res_handler.tabular_result(result);
        }
        "declare" => {
            let result = commands::declare_user_limit(client, args);
            res_handler.no_output_on_success(result);
        }
        "delete" => {
            let result = commands::delete_user_limit(client, args);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("user_limits", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_vhosts(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "declare" => {
            let result = commands::declare_vhost(client, args);
            res_handler.no_output_on_success(result);
        }
        "delete" => {
            let result = commands::delete_vhost(client, args);
            res_handler.delete_operation_result(result);
        }
        "delete_multiple" => {
            let mut prog_rep = res_handler.instantiate_progress_reporter();
            let result = commands::delete_multiple_vhosts(client, args, &mut *prog_rep);
            match result {
                Ok(Some(vhosts)) => {
                    res_handler.tabular_result(Ok(vhosts));
                }
                Ok(None) => {
                    res_handler.no_output_on_success(Ok(()));
                }
                Err(e) => {
                    res_handler.no_output_on_success::<()>(Err(e));
                }
            }
        }
        "list" => {
            let result = commands::list_vhosts(client);
            res_handler.tabular_result(result);
        }
        "enable_deletion_protection" => {
            let result = commands::enable_vhost_deletion_protection(client, args);
            res_handler.no_output_on_success(result);
        }
        "disable_deletion_protection" => {
            let result = commands::disable_vhost_deletion_protection(client, args);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("vhosts", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}

fn dispatch_vhost_limits(
    subcommand: &str,
    args: &ArgMatches,
    client: APIClient,
    vhost: &str,
    res_handler: &mut ResultHandler,
) -> ExitCode {
    match subcommand {
        "list" => {
            let result = commands::list_vhost_limits(client, vhost);
            res_handler.tabular_result(result);
        }
        "declare" => {
            let result = commands::declare_vhost_limit(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        "delete" => {
            let result = commands::delete_vhost_limit(client, vhost, args);
            res_handler.no_output_on_success(result);
        }
        _ => return unknown_subcommand("vhost_limits", subcommand, res_handler),
    }
    res_handler.exit_code.unwrap_or(ExitCode::Usage)
}
