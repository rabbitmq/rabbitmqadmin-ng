#!/bin/sh

CTL=${RUST_HTTP_API_CLIENT_RABBITMQCTL:="sudo rabbitmqctl"}
PLUGINS=${RUST_HTTP_API_CLIENT_RABBITMQ_PLUGINS:="sudo rabbitmq-plugins"}

case $CTL in
        DOCKER*)
          PLUGINS="docker exec ${CTL##*:} rabbitmq-plugins"
          CTL="docker exec ${CTL##*:} rabbitmqctl";;
esac

echo "Will use rabbitmqctl at ${CTL}"
echo "Will use rabbitmq-plugins at ${PLUGINS}"

$PLUGINS enable rabbitmq_management

sleep 3

# guest:guest has full access to /

$CTL add_vhost /
$CTL add_user guest guest
$CTL set_permissions -p / guest ".*" ".*" ".*"

$CTL add_vhost "rust/rabbitmqadmin"
$CTL set_permissions -p "rust/rabbitmqadmin" guest ".*" ".*" ".*"

# set cluster name
$CTL set_cluster_name rabbitmq@localhost

$CTL enable_feature_flag all

# Enable the plugins
$PLUGINS enable rabbitmq_shovel
$PLUGINS enable rabbitmq_shovel_management

$PLUGINS enable rabbitmq_federation
$PLUGINS enable rabbitmq_federation_management

$PLUGINS enable rabbitmq_stream
$PLUGINS enable rabbitmq_stream_management

true
