# <h1 align="center">Discord Channel Summarizer Blueprint 🤖</h1>

## 📚 Overview

This Tangle Blueprint provides an automated Discord channel summarization service using Hyperbolic's AI cloud.

Blueprints are specifications for <abbr title="Actively Validated Services">AVS</abbr>s on the Tangle Network. An AVS is
an off-chain service that runs arbitrary computations for a user-specified period of time.

This blueprint demonstrates how to create an automated AI-powered service that:

- Listens to Discord channels
- Aggregates messages over 24-hour periods
- Generates AI summaries using Hyperbolic through Rig's client adapter
- Posts results on-chain for verification and provenance

For more details, please refer to the [project documentation](https://docs.tangle.tools/developers/blueprints/introduction).

## 🚀 Features

- 🤖 Automated 24-hour channel summarization
- 🔗 On-chain result verification
- 🧠 AI-powered content summarization via Hyperbolic
- ⚡ Easy operator registration and management
- 📊 Transparent job execution tracking

## 📋 Prerequisites

Before you can run this project, you will need to have the following software installed on your machine:

- [Rust](https://www.rust-lang.org/tools/install)
- [Forge](https://getfoundry.sh)
- Discord Bot Token
- Hyperbolic API credentials
- Access to Tangle Network (testnet)

You will also need to install [cargo-tangle](https://crates.io/crates/cargo-tangle), our CLI tool for creating and
deploying Tangle Blueprints:

To install the Tangle CLI, run the following command:

> Supported on Linux, MacOS, and Windows (WSL2)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/tangle-network/gadget/releases/download/cargo-tangle-v0.1.2/cargo-tangle-installer.sh | sh
```

Or, if you prefer to install the CLI from crates.io:

```bash
cargo install cargo-tangle --force # to get the latest version.
```

## ⭐ Getting Started

1. Clone the repository and install dependencies:

```sh
cargo build
```

2. Configure environment variables:

```sh
cp .env.example .env
# Edit .env with your credentials:
# - DISCORD_TOKEN
# - HYPERBOLIC_API_KEY
# - TANGLE_WS_URL
```

3. Deploy the blueprint to Tangle:

```sh
cargo tangle blueprint deploy
```

## 🛠️ Development

The blueprint consists of three main components:

1. **Smart Contract** (`HelloBlueprint.sol`):

   - Handles operator registration
   - Processes job requests
   - Verifies and stores summarization results

2. **Service Implementation** (`lib.rs`):

   - Defines the summarization job
   - Implements Discord message collection
   - Manages AI summarization calls

3. **Runner** (`main.rs`):
   - Sets up the service context
   - Initializes event listeners
   - Manages the service lifecycle

## 📜 License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 📬 Feedback and Contributions

We welcome feedback and contributions to improve this blueprint.
Please open an issue or submit a pull request on our GitHub repository.
Please let us know if you fork this blueprint and extend it too!

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
