#!/bin/bash
rm -rf pkg
if ! wasm-pack build --target web --release --out-dir pkg -- --features wasm; then
    exit 1
fi
rm pkg/.gitignore
sed -i 's/flash_mint: FlashMint;//g' pkg/dove.d.ts
sed -i 's/stable_dvd: StableDvd;//g' pkg/dove.d.ts

rm -rf ../dove-frontend/pkg
mv pkg ../dove-frontend

rm -rf ../dove-frontend/public/pkg
mkdir ../dove-frontend/public/pkg

mv ../dove-frontend/pkg/dove_bg.wasm ../dove-frontend/public/pkg

# Webpack needs this file to exist due to a URL reference
# in the wasm-bindgen generated code.
touch ../dove-frontend/pkg/dove_bg.wasm