#!/bin/bash
rm -rf target/deploy
cargo build-sbf
solana airdrop 5 -u localhost
PID=$(solana program deploy target/deploy/dove.so -u localhost | grep -oP "Program Id: \K.*")
cd ..
echo "Program ID: $PID"