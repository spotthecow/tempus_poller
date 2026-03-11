use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use sqlx::postgres::PgPoolOptions;
use tempuspoints_poller::{client::TempusClient, db};
use tokio::{task::JoinSet, time};
use tracing::Instrument as _;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

const POLL_INTERVAL: time::Duration = time::Duration::from_millis(1050);
const CYCLE_INTERVAL: time::Duration = time::Duration::from_mins(60);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    let tracer_provider = init_telemetry()?;

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
    let mut cycle_interval = time::interval(CYCLE_INTERVAL);
    poll_interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);

    let mut running = true;
    while running {
        tokio::select! {
            _ = &mut shutdown => {
                tracing::info!("shutting down");
                break;
            }
            _ = cycle_interval.tick() => {}
        }

        let mut tasks = JoinSet::new();

        let maps = tempus.get_maps().await?;
        db::upsert_maps(&pg, &maps).await?;

        for map in &maps {
            tokio::select! {
                _ = &mut shutdown => {
                    tracing::debug!("shutting down");
                    running = false;
                    break;
                }
            _ = poll_interval.tick() => {}
            }

            let client = tempus.clone();
            let pool = pg.clone();
            let map_id = map.id;

            let poll_span = tracing::info_span!("poll_map", map_id);
            tasks.spawn(
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
                .instrument(poll_span),
            );
        }

        tasks.join_all().await;
    }

    tracer_provider.shutdown()?;
    Ok(())
}

fn init_telemetry() -> anyhow::Result<SdkTracerProvider> {
    let resource = opentelemetry_sdk::Resource::builder()
        .with_attributes([opentelemetry::KeyValue::new(
            "service.name",
            "tempuspoints-poller",
        )])
        .build();
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()?;
    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .with_resource(resource)
        .build();
    let tracer = tracer_provider.tracer("tempuspoints-poller");
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::OpenTelemetryLayer::new(tracer))
        .init();
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

    Ok(tracer_provider)
}
