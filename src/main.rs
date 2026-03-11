use sqlx::postgres::PgPoolOptions;
use tempuspoints_poller::{client::TempusClient, db};
use tokio::time;
use tracing::Instrument;
use tracing_subscriber::EnvFilter;

const POLL_INTERVAL: time::Duration = time::Duration::from_millis(1050);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let tempus = TempusClient::new();
    let pg = PgPoolOptions::new()
        .max_connections(5)
        .connect(
            &std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set"),
        )
        .await?;

    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    let mut poll_interval = time::interval(POLL_INTERVAL);
    let mut hourly_interval = time::interval(time::Duration::from_hours(1));
    poll_interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);

    'poll: loop {
        tokio::select! {
            _ = &mut shutdown => {
                tracing::info!("shutting down");
                break;
            }
            _ = hourly_interval.tick() => {}
        }

        let maps = tempus.get_maps().await?;
        db::upsert_maps(&pg, &maps).await?;

        for map in &maps {
            tokio::select! {
                _ = &mut shutdown => {
                    tracing::info!("shutting down");
                    break 'poll;
                }
                _ = poll_interval.tick() => {}
            }

            let client = tempus.clone();
            let pool = pg.clone();
            let map_id = map.id;

            let span = tracing::info_span!("poll map", map_id);
            tokio::spawn(
                async move {
                    match client.get_map_records(map_id).await {
                        Ok(records_list) => {
                            let all_records: Vec<_> = records_list
                                .records
                                .soldier
                                .into_iter()
                                .chain(records_list.records.demoman.into_iter())
                                .collect();

                            if let Err(e) = db::upsert_users(&pool, &all_records).await {
                                tracing::warn!(map_id, "upsert users failed: {e}");
                            }

                            if let Err(e) =
                                db::upsert_records(&pool, map_id as i32, &all_records).await
                            {
                                tracing::warn!(map_id, "upsert records failed: {e}");
                            }
                        }

                        Err(e) => tracing::warn!(map_id, "fetch failed: {e}"),
                    };
                }
                .instrument(span),
            );
        }
    }

    Ok(())
}
