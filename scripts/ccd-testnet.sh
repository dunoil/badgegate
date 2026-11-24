#!/usr/bin/env bash
# Run concordium-client against Concordium testnet from this VM.
#
# Why this exists: direct gRPC from the VM to grpc.testnet.concordium.com
# port 20000 fails (verified 24 Nov 2026, phase 3), and concordium-client
# does not use the HTTP proxy environment variables. This wrapper opens a
# local HTTP CONNECT tunnel with socat through the proxy already present
# in the environment (HTTPS_PROXY), then points concordium-client at the
# local end of the tunnel with TLS and the correct gRPC target name.
# No credentials are stored in this script; the proxy user and password
# are read from HTTPS_PROXY at run time and never printed.
#
# Usage: ./ccd-testnet.sh <concordium-client arguments, without grpc flags>
# Example: ./ccd-testnet.sh raw GetConsensusInfo
set -euo pipefail

CLIENT="${CONCORDIUM_CLIENT:-$HOME/workspace/toolchains/bin/concordium-client}"
LOCAL_PORT="${CCD_TUNNEL_PORT:-12000}"
# Proxy endpoint for the CONNECT tunnel. Defaults match the build VM used
# for the hackathon; override both variables on any other network, or skip
# this wrapper entirely and call concordium-client with the official direct
# flags (--grpc-ip grpc.testnet.concordium.com --grpc-port 20000 --secure).
PROXY_HOST="${CCD_PROXY_HOST:-hatch-egress-proxy}"
PROXY_PORT="${CCD_PROXY_PORT:-3128}"

PA="$(python3 -c "import os,urllib.parse; u=urllib.parse.urlparse(os.environ['HTTPS_PROXY']); print(urllib.parse.unquote(u.username or '')+':'+urllib.parse.unquote(u.password or ''))")"

socat "TCP-LISTEN:${LOCAL_PORT},fork,reuseaddr,bind=127.0.0.1" \
  "PROXY:${PROXY_HOST}:grpc.testnet.concordium.com:20000,proxyport=${PROXY_PORT},proxyauth=${PA}" &
TUNNEL_PID=$!
trap 'kill ${TUNNEL_PID} 2>/dev/null || true' EXIT
sleep 1

exec "${CLIENT}" "$@" \
  --grpc-ip 127.0.0.1 \
  --grpc-port "${LOCAL_PORT}" \
  --secure \
  --grpc-target grpc.testnet.concordium.com
