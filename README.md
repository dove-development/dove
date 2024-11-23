# dove
## Building for Solana
This option builds the Dove protocol smart contract.

Install the Solana CLI:
```sh
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
```
Then, build the project:
```sh
cargo build-sbf
```
The product is `target/deploy/dove.so`.

## Building for WebAssembly
This option builds the WebAssembly library, exposing the primitives of the Dove protocol to TypeScript and JavaScript.

```sh
wasm-pack build --target nodejs --release --out-dir ./pkg -- --features wasm
sed -i 's/flash_mint: FlashMint;//g' pkg/dove.d.ts
sed -i 's/stable_dvd: StableDvd;//g' pkg/dove.d.ts
sed -i 's/dvd_price: DvdPrice;//g' pkg/dove.d.ts
```

Dove primitives can then be imported via `import { ... } from "pkg/dove"`.

## Updating the WASM library for the frontend
To build the `web` version for `dove-frontend`, simply run:
```sh
./deploy_frontend.sh
```
which expects the `dove-frontend` repository to be cloned at `../dove-frontend`.
