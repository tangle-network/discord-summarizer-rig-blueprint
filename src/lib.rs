use blueprint_sdk::event_listeners;
use blueprint_sdk::event_listeners::cronjob::CronJobDefinition;
use blueprint_sdk::macros::contexts::{ServicesContext, TangleClientContext};
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

/// Generates a daily summary of Discord messages and posts it to the specified channel
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

    let report = match generate_report(&pool).await {
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
