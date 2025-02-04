use blueprint_sdk::logging;
use blueprint_sdk::runners::core::runner::BlueprintRunner;
use blueprint_sdk::runners::eigenlayer::bls::EigenlayerBLSConfig;
use blueprint_sdk::runners::tangle::tangle::TangleConfig;
use discord_summarizer_rig_blueprint as blueprint;

#[blueprint_sdk::main(env)]
async fn main() {
    // Discord environment vars (token and channel id to listen to)
    let token = std::env::var("DISCORD_TOKEN").context("'DISCORD_TOKEN' was not found")?;
    let channel_id: ChannelId = std::env::var("CHANNEL_ID")
        .context("'CHANNEL_ID' was not found")?
        .parse::<u64>()
        .context("Tried to convert CHANNEL_ID env var but the value is not a valid u64")?
        .into();

    let context = blueprint::ServiceContext {
        config: env.clone(),
        cron: "0 0 * * *".to_string(),
    };

    // Create the event handler from the job
    let summarize_job = blueprint::SummarizeDailyMessagesEventHandler::new(&env, context).await?;
    let tangle_config = TangleConfig::default();
    logging::info!("Starting the event watcher ...");
    BlueprintRunner::new(tangle_config, env)
        .job(summarize_job)
        .run()
        .await?;

    logging::info!("Exiting...");
    Ok(())
}
