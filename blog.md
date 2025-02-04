# Building an AI-Powered Discord Summarizer with Tangle Blueprints

Hey there! 👋 Today, we're going to show you how to build a decentralized Discord channel summarizer using Tangle Blueprints and Hyperbolic AI. This service will automatically create daily summaries of your Discord channels and store them on-chain for transparency and verification.

## Getting Started

First, let's create our project using `cargo-tangle`:

```bash
cargo tangle blueprint create --name discord-summarizer-rig-blueprint
```

When prompted, fill in your details:

- GitHub username/organization
- Description
- Homepage URL
- Select No for Nix Flakes
- Yes for Dockerfile (using rustlang/rust:nightly)
- Yes for GitHub Actions workflows

## Setting Up Dependencies

Navigate to your project and add the required dependencies:

```bash
cd discord-summarizer-rig-blueprint

# Add core dependencies with features
cargo add blueprint-sdk --git https://github.com/tangle-network/gadget --features "cronjob,tangle,macros"
cargo add chrono --features "serde"
cargo add dotenv
cargo add rig-core
cargo add serenity
cargo add sqlx --features "runtime-tokio-rustls,postgres,chrono,macros"
```

## Building the AI Components

Our AI implementation consists of two main parts: the message summarization logic and the report generation. Let's break this down step by step.

### 1. Message Summarization

First, we'll create a function that handles the core AI interaction. This function takes JSON-formatted Discord messages and returns a markdown summary using Hyperbolic's DeepSeek R1 model:

```rust
use rig::completion::Prompt;
use std::env;

pub async fn summarize_messages(messages_json: String) -> Result<String, Box<dyn std::error::Error>> {
    // Create Hyperbolic client
    let client = rig::providers::hyperbolic::Client::new(
        &env::var("HYPERBOLIC_API_KEY").expect("HYPERBOLIC_API_KEY not set"),
    );

    // Create agent with DeepSeek R1 model
    let summarizer_agent = client
        .agent("deepseek-ai/DeepSeek-R1")
        .preamble("Your job is to summarize a list of Discord messages from a single day in JSON format.
            The output should be in Markdown and is intended to provide a summary of important events
            and conversation topics from the day given.
            If there are no messages, simply respond 'Nothing was discussed.'")
        .build();

    let result = summarizer_agent.prompt(&messages_json).await?;
    Ok(result)
}
```

A few key points about this implementation:

- We're using Hyperbolic's DeepSeek R1 model, which is great for text summarization
- The preamble instructs the AI to format output in Markdown
- We handle the case where no messages exist

### 2. Report Generation

Next, we'll create a function that handles the database interaction and report generation workflow:

```rust
use sqlx::PgPool;

pub async fn generate_report(pool: &PgPool) -> Result<String, Box<dyn std::error::Error>> {
    let date_yesterday = chrono::Utc::now().date_naive() - chrono::Days::new(1);

    // Get messages from yesterday
    let res: Option<serde_json::Value> =
        sqlx::query_scalar("SELECT jsonb_agg(data) FROM messages WHERE created::date = $1")
            .bind(date_yesterday)
            .fetch_optional(pool)
            .await?;

    let Some(res) = res else {
        return Err("There were no messages in the database :(".into());
    };

    // Generate and store summary
    let raw_json = serde_json::to_string_pretty(&res).unwrap();
    let prompt_result = summarize_messages(raw_json).await?;

    sqlx::query("INSERT INTO summaries (summary, date) VALUES ($1, $2)")
        .bind(&prompt_result)
        .bind(date_yesterday)
        .execute(pool)
        .await?;

    Ok(prompt_result)
}
```

This function handles several important tasks:

- Retrieves messages from the previous day using PostgreSQL's `jsonb_agg`
- Formats the messages for the AI model
- Stores the generated summary back in the database
- Returns the summary for Discord posting

Both functions are part of our `llm.rs` module, which serves as the AI integration layer of our blueprint.

## Defining the Blueprint Job

Now let's set up our service context and job definition in `src/lib.rs`:

```rust
use blueprint_sdk::event_listeners;
use blueprint_sdk::event_listeners::cronjob::CronJobDefinition;
use blueprint_sdk::macros::contexts::TangleClientContext;
use blueprint_sdk::{config::GadgetConfiguration, event_listeners::cronjob::CronJob};
use serenity::all::{ChannelId, Http};
use std::convert::Infallible;

pub mod llm;

#[derive(Clone, TangleClientContext)]
pub struct ServiceContext {
    #[config]
    pub config: GadgetConfiguration,
    pub cron: String,
}

impl CronJobDefinition for ServiceContext {
    fn cron(&self) -> impl Into<String> {
        self.config.cron.clone()
    }
}

#[blueprint_sdk::job(
    id = 0,
    params(channel_id, token),
    result(_),
    event_listener(
        listener = CronJob::<ServiceContext>,
    ),
)]
pub async fn summarize_daily_messages(
    channel_id: ChannelId,
    token: String,
    context: ServiceContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let http_client = Http::new(&token);
    let pool = context.config.get_pool().await?;

    let report = match llm::generate_report(&pool).await {
        Ok(res) => res,
        Err(e) => {
            println!("{e}");
            return Ok(()); // Return Ok to keep job running
        }
    };

    if let Err(e) = http_client
        .send_message(channel_id, Vec::new(), &report)
        .await
    {
        println!("Something went wrong while sending summary message: {e}");
    }

    Ok(())
}
```

## Setting Up the Runner

Now let's implement our main entry point in `src/main.rs` that will tie everything together:

```rust
use blueprint_sdk::logging;
use blueprint_sdk::runners::core::runner::BlueprintRunner;
use blueprint_sdk::runners::tangle::tangle::TangleConfig;
use discord_summarizer_rig_blueprint as blueprint;
use serenity::all::ChannelId;

#[blueprint_sdk::main(env)]
async fn main() {
    // Load environment variables
    dotenv::dotenv().ok();

    // Get Discord configuration
    let token = std::env::var("DISCORD_TOKEN")
        .expect("'DISCORD_TOKEN' was not found");
    let channel_id: ChannelId = std::env::var("CHANNEL_ID")
        .expect("'CHANNEL_ID' was not found")
        .parse::<u64>()
        .expect("Invalid channel ID")
        .into();

    // Create service context with daily cron schedule
    let context = blueprint::ServiceContext {
        config: env.clone(),
        cron: "0 0 * * *".to_string(), // Run at midnight every day
    };

    // Create the event handler from the job
    let summarize_job = blueprint::SummarizeDailyMessagesEventHandler::new(&env, context).await?;

    logging::info!("Starting the event watcher ...");
    let tangle_config = TangleConfig::default();
    BlueprintRunner::new(tangle_config, env)
        .job(summarize_job)
        .run()
        .await?;

    logging::info!("Exiting...");
    Ok(())
}
```

## Database Setup

Our service needs a PostgreSQL database to store messages and summaries. Let's set up the database components:

First, create `src/db.rs` to handle database connections and table creation:

```rust
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub async fn setup_database() -> Result<PgPool, Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Configure and create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await?;

    // Create tables if they don't exist
    create_tables(&pool).await?;

    Ok(pool)
}

async fn create_tables(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // Create messages table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            id SERIAL PRIMARY KEY,
            data JSONB NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
        );"#,
    )
    .execute(pool)
    .await?;

    // Create summaries table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS summaries (
            id SERIAL PRIMARY KEY,
            summary VARCHAR NOT NULL,
            date DATE NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
        );"#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
```

Next, create a helper script `scripts/setup_db.sh` to initialize the database:

```bash
#!/bin/bash

# Load environment variables
set -a
source .env
set +a

# Check if psql is installed
if ! command -v psql &> /dev/null; then
    echo "Error: PostgreSQL client (psql) is not installed"
    exit 1
fi

# Extract database name from DATABASE_URL
DB_NAME=$(echo $DATABASE_URL | sed 's/.*\///g')

# Create database if it doesn't exist
psql -h localhost -U postgres -c "CREATE DATABASE $DB_NAME;" 2>/dev/null || true

echo "Database setup complete!"
```

Make the script executable and run it:

```bash
chmod +x scripts/setup_db.sh
./scripts/setup_db.sh
```

Create a `.env` file with your configuration:

```env
DATABASE_URL=postgres://username:password@localhost:5432/discord_summaries
DISCORD_TOKEN=your_discord_bot_token
CHANNEL_ID=your_discord_channel_id
HYPERBOLIC_API_KEY=your_hyperbolic_api_key
CRON_SCHEDULE="0 0 * * *"  # Optional: defaults to midnight if not set
```

## Testing Locally

1. Make sure PostgreSQL is running and set up the database:

```bash
./scripts/setup_db.sh
```

1. Build the blueprint:

```bash
cargo build
```

2. Run the service:

```bash
cargo run
```

## Deploying to Tangle Network

1. First, make sure your blueprint is ready for deployment:

```bash
cargo b
```

This should succeed and generate a `blueprint.json` that contains all the necessary information for deployment.

2. Deploy to the Tangle testnet:

```bash
cargo tangle blueprint deploy --network testnet
```

The deployment process will:

- Deploy your blueprint
- Deploy the smart contracts

## How It Works

Let's break down what happens when our service runs:

1. The cron job triggers at midnight every day
2. The service queries the database for all messages from the previous day
3. It sends these messages to Hyperbolic's AI for summarization
4. The summary is stored in the database and posted to Discord
5. All actions are recorded on-chain for verification

## Customization Options

You can easily customize this blueprint:

- Change the cron schedule for different summary frequencies
- Modify the AI prompt for different summary styles
- Add multiple channel support
- Include message analytics or statistics

## Conclusion

We've built a decentralized Discord summarizer that combines the power of AI with the transparency of blockchain technology. This blueprint demonstrates how Tangle Network can be used to create verifiable off-chain services that interact with both Web2 (Discord) and Web3 (blockchain) systems.

The complete code for this tutorial is available on GitHub at [discord-summarizer-rig-blueprint](https://github.com/tangle-network/discord-summarizer-rig-blueprint).

Happy building! 🚀

---

_This tutorial was inspired by the excellent article ["Building a Discord Summarizer bot with DeepSeek, Rig & Rust"](https://www.shuttle.dev/blog/2025/01/29/discord-summarizer-deepseek-rig-rust) from the Shuttle team. We've adapted their implementation to work with Tangle Blueprints, demonstrating how to build verifiable, decentralized services that can interact with both Web2 and Web3 systems._
