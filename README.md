![Borlaug Logo](logo.png)

# Borlaug Blockchain
Borlaug blockchain network is built for agro economies & self organizing communities.

## Setup

#### Prerequisites
Install stable Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Build project
```bash
git clone git@github.com:BorlaugWebServices/borlaug.git
cd borlaug
```
Use instant seal (for development)
```bash
cargo build --release --features instant_seal
```
Use grandpa/babe (for production). Note that with babe, stopping the chain for any length of time will cause errors.
```bash
cargo build --release --features grandpa_babe
```
#### Run
Run node in "development" mode
```bash
./target/release/borlaug --dev
```

Run tests
```bash
cargo test --all
```

## License
Borlaug is [GPL 3.0 licensed](LICENSE).

 
#
