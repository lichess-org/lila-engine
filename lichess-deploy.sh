#!/bin/sh -e

# Either compile locally with:
cargo +stable build --release --target x86_64-unknown-linux-musl

# Or get the artifact from the github action:
# https://github.com/lichess-org/lila-engine/actions/workflows/build.yml

# Backup old binary
ssh "root@$1.lichess.ovh" mv /usr/local/bin/lila-engine /usr/local/bin/lila-engine.bak || (echo "first deploy on this server? set up service and comment out this line" && false)

# Setup new binary
scp ./target/x86_64-unknown-linux-musl/release/lila-engine "root@$1.lichess.ovh":/usr/local/bin/lila-engine

# Restart both services
ssh "root@$1.lichess.ovh" systemctl restart lila-engine
ssh "root@$1.lichess.ovh" systemctl restart lila-engine-tls
