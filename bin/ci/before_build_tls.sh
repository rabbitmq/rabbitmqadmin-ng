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
management.ssl.cacertfile = /certs/ca_certificate.pem
management.ssl.certfile   = /certs/server_certificate.pem
management.ssl.keyfile    = /certs/server_key.pem

# Keep HTTP enabled for other tests
management.tcp.port = 15672
loopback_users      = none
EOF

echo "RabbitMQ TLS configuration written to ${RABBITMQ_CONF}"

# If using Docker, start a container with TLS configuration
if [ -n "$CONTAINER_ID" ]; then
    echo "Note: Docker service container ${CONTAINER_ID} detected."
    echo "For TLS tests, use a standalone Docker container instead."
    echo ""
    echo "To start RabbitMQ with TLS manually:"
    echo "  docker run -d --name rabbitmq-tls \\"
    echo "    -p 15671:15671 -p 15672:15672 -p 5672:5672 \\"
    echo "    -v ${CERTS_DIR}:/certs:ro \\"
    echo "    -v ${RABBITMQ_CONF}:/etc/rabbitmq/rabbitmq.conf:ro \\"
    echo "    rabbitmq:4.0-management"
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
