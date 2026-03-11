use crate::models::{Map, Record};
use sqlx::PgPool;

#[tracing::instrument]
pub async fn upsert_maps(pool: &PgPool, maps: &[Map]) -> anyhow::Result<()> {
    for map in maps {
        let zone_counts = serde_json::to_value(&map.zone_counts)?;
        let authors = serde_json::to_value(&map.authors)?;

        sqlx::query!(
            r#"
            INSERT INTO maps (id, name, tier_soldier, tier_demoman, rating_soldier, rating_demoman, zone_counts, authors, video_soldier, video_demoman, fetched_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now())
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                tier_soldier = EXCLUDED.tier_soldier,
                tier_demoman = EXCLUDED.tier_demoman,
                rating_soldier = EXCLUDED.rating_soldier,
                rating_demoman = EXCLUDED.rating_demoman,
                zone_counts = EXCLUDED.zone_counts,
                authors = EXCLUDED.authors,
                video_soldier = EXCLUDED.video_soldier,
                video_demoman = EXCLUDED.video_demoman,
                fetched_at = EXCLUDED.fetched_at
            "#,
            map.id as i32,
            &map.name,
            map.tier_info.soldier as i16,
            map.tier_info.demoman as i16,
            map.rating_info.soldier as i16,
            map.rating_info.demoman as i16,
            zone_counts,
            authors,
            map.videos.soldier.as_deref(),
            map.videos.demoman.as_deref(),
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

#[tracing::instrument]
pub async fn upsert_users(pool: &PgPool, records: &[Record]) -> anyhow::Result<()> {
    for rec in records {
        sqlx::query!(
            r#"
            INSERT INTO users (id, steamid, name)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE SET
                steamid = EXCLUDED.steamid,
                name = EXCLUDED.name
            "#,
            rec.user_id as i32,
            &rec.steamid,
            &rec.name,
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

#[tracing::instrument]
pub async fn upsert_records(pool: &PgPool, map_id: i32, records: &[Record]) -> anyhow::Result<()> {
    for rec in records {
        sqlx::query!(
            r#"
            INSERT INTO records (id, map_id, user_id, class, duration, date, fetched_at)
            VALUES ($1, $2, $3, $4, $5, to_timestamp($6), now())
            ON CONFLICT (user_id, map_id, class) DO UPDATE SET
                id = EXCLUDED.id,
                duration = EXCLUDED.duration,
                date = EXCLUDED.date,
                fetched_at = EXCLUDED.fetched_at
            "#,
            rec.id as i32,
            map_id,
            rec.user_id as i32,
            rec.class as i16,
            rec.duration,
            rec.date,
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}
