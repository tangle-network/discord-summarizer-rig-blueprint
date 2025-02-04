use blueprint_sdk::logging;
use blueprint_sdk::runners::core::runner::BlueprintRunner;
use blueprint_sdk::runners::tangle::tangle::TangleConfig;
use discord_summarizer_rig_blueprint as blueprint;
use serenity::all::ChannelId;

mod db;

#[blueprint_sdk::main(env)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Set up database connection
    let pool = db::setup_database()
        .await
        .expect("Failed to set up database");

    // Get Discord configuration
    let token = std::env::var("DISCORD_TOKEN").expect("'DISCORD_TOKEN' was not found");
    let channel_id: ChannelId = std::env::var("CHANNEL_ID")
        .expect("'CHANNEL_ID' was not found")
        .parse::<u64>()
        .expect("Invalid channel ID")
        .into();

    // Get cron schedule from env or use default
    let cron_schedule = std::env::var("CRON_SCHEDULE").unwrap_or_else(|_| "0 0 * * *".to_string());

    // Create service context with cron schedule
    let context = blueprint::ServiceContext {
        config: env.clone(),
        cron: cron_schedule,
    };

    // Store pool in configuration
    env.set_pool(pool);

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
