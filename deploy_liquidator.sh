#!/bin/bash
rm -rf ../dove-liquidator/pkg
wasm-pack build --target nodejs --release --out-dir ../dove-liquidator/pkg -- --features wasm
sed -i 's/flash_mint: FlashMint;//g' ../dove-liquidator/pkg/dove.d.ts
sed -i 's/stable_dvd: StableDvd;//g' ../dove-liquidator/pkg/dove.d.ts
