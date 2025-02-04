use rig::completion::Prompt;
use sqlx::PgPool;
use std::env;

pub async fn summarize_messages(
    messages_json: String,
) -> Result<String, Box<dyn std::error::Error>> {
    // Create Hyperbolic client
    let client = rig::providers::hyperbolic::Client::new(
        &env::var("HYPERBOLIC_API_KEY").expect("HYPERBOLIC_API_KEY not set"),
    );

    // Create agent with a single context prompt
    let summarizer_agent = client
        .agent("deepseek-ai/DeepSeek-R1")
        .preamble("Your job is to summarize a list of Discord messages from a single day in JSON format.

            The output should be in Markdown and is intended to provide a summary of important events and conversation topics from the day given.

            If there are no messages, simply respond 'Nothing was discussed.'")
        .build();

    let result = summarizer_agent.prompt(&messages_json).await?;

    Ok(result)
}

pub async fn generate_report(pool: &PgPool) -> Result<String, Box<dyn std::error::Error>> {
    let date_yesterday = chrono::Utc::now().date_naive() - chrono::Days::new(1);
    let res: Option<serde_json::Value> =
        sqlx::query_scalar("SELECT jsonb_agg(data) FROM messages WHERE created::date = $1")
            .bind(date_yesterday)
            .fetch_optional(pool)
            .await?;

    let Some(res) = res else {
        return Err("There were no messages in the database :(".into());
    };

    let raw_json = serde_json::to_string_pretty(&res).unwrap();

    let prompt_result = match summarize_messages(raw_json).await {
        Ok(res) => res,
        Err(e) => {
            return Err(
                format!("Something went wrong while trying to summarize messages: {e}").into(),
            )
        }
    };

    if let Err(e) = sqlx::query("INSERT INTO summaries (summary, date) VALUES ($1, $2)")
        .bind(&prompt_result)
        .bind(date_yesterday)
        .execute(pool)
        .await
    {
        return Err(format!("Error ocurred while storing summary: {e}").into());
    };

    Ok(prompt_result)
}
