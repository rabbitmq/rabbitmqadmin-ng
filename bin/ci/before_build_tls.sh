#!/bin/sh

# TLS test setup script for CI
# Generates certificates using tls-gen and configures RabbitMQ with TLS-enabled management API

set -e

CTL=${RUST_HTTP_API_CLIENT_RABBITMQCTL:="sudo rabbitmqctl"}
PLUGINS=${RUST_HTTP_API_CLIENT_RABBITMQ_PLUGINS:="sudo rabbitmq-plugins"}

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CERTS_DIR="${REPO_ROOT}/tests/tls/certs"

# Docker container ID (passed via environment or extracted from CTL)
CONTAINER_ID=""

case $CTL in
    DOCKER*)
        CONTAINER_ID="${CTL##*:}"
        PLUGINS="docker exec ${CONTAINER_ID} rabbitmq-plugins"
        CTL="docker exec ${CONTAINER_ID} rabbitmqctl"
        ;;
esac

echo "Will use rabbitmqctl at ${CTL}"
echo "Will use rabbitmq-plugins at ${PLUGINS}"

# Create certs directory
mkdir -p "${CERTS_DIR}"

# Check if tls-gen is available
TLSGEN_DIR="${TLSGEN_DIR:-}"
if [ -z "$TLSGEN_DIR" ]; then
    echo "TLSGEN_DIR not set, cloning tls-gen..."
    TLSGEN_DIR="${REPO_ROOT}/target/tls-gen"
    if [ ! -d "$TLSGEN_DIR" ]; then
        git clone --depth 1 https://github.com/rabbitmq/tls-gen.git "$TLSGEN_DIR"
    fi
fi

echo "Using tls-gen at ${TLSGEN_DIR}"

# Generate certificates using basic profile
cd "${TLSGEN_DIR}/basic"
make CN=localhost
make alias-leaf-artifacts

# Copy certificates to the test directory
cp result/ca_certificate.pem "${CERTS_DIR}/"
cp result/server_certificate.pem "${CERTS_DIR}/"
cp result/server_key.pem "${CERTS_DIR}/"
cp result/client_certificate.pem "${CERTS_DIR}/"
cp result/client_key.pem "${CERTS_DIR}/"

echo "Certificates generated and copied to ${CERTS_DIR}"

# Create RabbitMQ configuration for TLS
RABBITMQ_CONF="${CERTS_DIR}/rabbitmq.conf"
cat > "${RABBITMQ_CONF}" << 'EOF'
# Enable TLS on management plugin
management.ssl.port       = 15671
management.ssl.cacertfile = /etc/rabbitmq/certs/ca_certificate.pem
management.ssl.certfile   = /etc/rabbitmq/certs/server_certificate.pem
management.ssl.keyfile    = /etc/rabbitmq/certs/server_key.pem

# Keep HTTP enabled for other tests
management.tcp.port = 15672
EOF

echo "RabbitMQ TLS configuration written to ${RABBITMQ_CONF}"

# If using Docker, copy certificates and configuration to container
if [ -n "$CONTAINER_ID" ]; then
    echo "Copying certificates to Docker container ${CONTAINER_ID}..."

    docker exec "${CONTAINER_ID}" mkdir -p /etc/rabbitmq/certs
    docker cp "${CERTS_DIR}/ca_certificate.pem" "${CONTAINER_ID}:/etc/rabbitmq/certs/"
    docker cp "${CERTS_DIR}/server_certificate.pem" "${CONTAINER_ID}:/etc/rabbitmq/certs/"
    docker cp "${CERTS_DIR}/server_key.pem" "${CONTAINER_ID}:/etc/rabbitmq/certs/"
    docker cp "${RABBITMQ_CONF}" "${CONTAINER_ID}:/etc/rabbitmq/conf.d/20-tls.conf"

    # Set proper permissions
    docker exec "${CONTAINER_ID}" chmod 644 /etc/rabbitmq/certs/*.pem
    docker exec "${CONTAINER_ID}" chmod 600 /etc/rabbitmq/certs/server_key.pem

    echo "Restarting RabbitMQ to apply TLS configuration..."
    docker exec "${CONTAINER_ID}" rabbitmqctl stop_app
    docker exec "${CONTAINER_ID}" rabbitmqctl start_app

    sleep 5

    # Verify TLS listener is active
    echo "Verifying TLS listener..."
    docker exec "${CONTAINER_ID}" rabbitmq-diagnostics listeners | grep -E "15671|ssl" || echo "Warning: TLS listener may not be active"
fi

# Enable management plugin (should already be enabled in the management image)
$PLUGINS enable rabbitmq_management

sleep 3

# Configure vhosts and users (same as before_build.sh)
$CTL add_vhost /
$CTL add_user guest guest || true
$CTL set_permissions -p / guest ".*" ".*" ".*"

# Clean up test vhosts
cd "${REPO_ROOT}"
cargo -q run '--' vhosts delete_multiple --name-pattern "^rabbitmqadmin" --dry-run --table-style modern || true
cargo -q run '--' --non-interactive vhosts delete_multiple --name-pattern "^rabbitmqadmin" || true

$CTL add_vhost "rust/rabbitmqadmin"
$CTL set_permissions -p "rust/rabbitmqadmin" guest ".*" ".*" ".*"

# Set cluster name
$CTL set_cluster_name rabbitmq@localhost

$CTL enable_feature_flag all

# Enable additional plugins
$PLUGINS enable rabbitmq_shovel
$PLUGINS enable rabbitmq_shovel_management
$PLUGINS enable rabbitmq_federation
$PLUGINS enable rabbitmq_federation_management
$PLUGINS enable rabbitmq_stream
$PLUGINS enable rabbitmq_stream_management

# Export certificate paths for tests
echo ""
echo "=== TLS Test Environment ==="
echo "CA Certificate: ${CERTS_DIR}/ca_certificate.pem"
echo "Client Certificate: ${CERTS_DIR}/client_certificate.pem"
echo "Client Key: ${CERTS_DIR}/client_key.pem"
echo "TLS Endpoint: https://localhost:15671/api"
echo ""
echo "To run TLS tests:"
echo "  TLS_CERTS_DIR=${CERTS_DIR} cargo nextest run -E 'binary(tls_tests)' --run-ignored=only"
echo ""

true
