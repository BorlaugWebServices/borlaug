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
cargo build --release
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
