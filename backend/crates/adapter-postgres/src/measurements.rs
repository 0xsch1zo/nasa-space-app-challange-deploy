use std::collections::BTreeMap;

use application::ports::outbound::{Bucket, InfraResult, MeasurementRepository, TimeRange};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::crew::{AstronautId, StationId, Subject};
use domain::metrics::{MetricCode, MetricScope, Reading, RecordedReading};
use sqlx::types::Json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::infra;

/// Generic, metric-agnostic storage of history aggregates.
#[derive(Clone)]
pub struct PgMeasurementRepository {
    pool: PgPool,
}

impl PgMeasurementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Table layout for one kind of subject. Both layouts are identical apart
/// from names, so every query below is written once.
struct Tables {
    history: &'static str,
    latest: &'static str,
    subject_col: &'static str,
}

const ASTRONAUT: Tables = Tables {
    history: "measurements",
    latest: "latest_measurements",
    subject_col: "astronaut_id",
};
const STATION: Tables = Tables {
    history: "station_measurements",
    latest: "latest_station_measurements",
    subject_col: "station_id",
};

fn tables_for(subject: Subject) -> (&'static Tables, Uuid) {
    match subject {
        Subject::Astronaut(AstronautId(id)) => (&ASTRONAUT, id),
        Subject::Station(StationId(id)) => (&STATION, id),
    }
}

type Components = Json<BTreeMap<String, f64>>;

fn metric_code(raw: String) -> Option<MetricCode> {
    // Rows of metrics no longer in the code are skipped, not fatal.
    MetricCode::new(raw).ok()
}

fn bucket_interval(b: Bucket) -> &'static str {
    match b {
        Bucket::Hour => "1 hour",
        Bucket::Day => "1 day",
        Bucket::Week => "7 days",
    }
}

impl PgMeasurementRepository {
    async fn append_to(&self, t: &Tables, readings: &[&RecordedReading]) -> InfraResult<u64> {
        if readings.is_empty() {
            return Ok(0);
        }
        let mut codes = Vec::with_capacity(readings.len());
        let mut subjects = Vec::with_capacity(readings.len());
        let mut times = Vec::with_capacity(readings.len());
        let mut components = Vec::with_capacity(readings.len());
        let mut sources = Vec::with_capacity(readings.len());
        for r in readings {
            codes.push(r.metric().as_str().to_owned());
            subjects.push(tables_for(r.subject()).1);
            times.push(r.reading().recorded_at);
            components.push(serde_json::to_string(&r.reading().values).expect("map serializes"));
            sources.push(r.source().to_owned());
        }
        let (history, latest, col) = (t.history, t.latest, t.subject_col);

        let mut tx = self.pool.begin().await.map_err(infra)?;
        let inserted = sqlx::query(&format!(
            "INSERT INTO {history} (metric_code, {col}, recorded_at, components, source)
             SELECT c, s, t, j::jsonb, src
             FROM UNNEST($1::text[], $2::uuid[], $3::timestamptz[], $4::text[], $5::text[])
                  AS x(c, s, t, j, src)
             ON CONFLICT DO NOTHING"
        ))
        .bind(&codes)
        .bind(&subjects)
        .bind(&times)
        .bind(&components)
        .bind(&sources)
        .execute(&mut *tx)
        .await
        .map_err(infra)?
        .rows_affected();

        sqlx::query(&format!(
            "INSERT INTO {latest} ({col}, metric_code, recorded_at, components)
             SELECT DISTINCT ON (s, c) s, c, t, j::jsonb
             FROM UNNEST($1::text[], $2::uuid[], $3::timestamptz[], $4::text[]) AS x(c, s, t, j)
             ORDER BY s, c, t DESC
             ON CONFLICT ({col}, metric_code) DO UPDATE
                SET recorded_at = EXCLUDED.recorded_at, components = EXCLUDED.components
                WHERE {latest}.recorded_at <= EXCLUDED.recorded_at"
        ))
        .bind(&codes)
        .bind(&subjects)
        .bind(&times)
        .bind(&components)
        .execute(&mut *tx)
        .await
        .map_err(infra)?;

        tx.commit().await.map_err(infra)?;
        Ok(inserted)
    }
}

#[async_trait]
impl MeasurementRepository for PgMeasurementRepository {
    async fn ensure_storage(&self, metrics: &[(MetricCode, MetricScope)]) -> InfraResult<()> {
        for (code, scope) in metrics {
            let parent = match scope {
                MetricScope::Astronaut => ASTRONAUT.history,
                MetricScope::Station => STATION.history,
            };
            // `MetricCode` is guaranteed to match [a-z][a-z0-9_]*, so it is safe
            // to use as an identifier and literal here.
            let sql = format!(
                "CREATE TABLE IF NOT EXISTS {parent}_{code} \
                 PARTITION OF {parent} FOR VALUES IN ('{code}')"
            );
            sqlx::query(&sql).execute(&self.pool).await.map_err(infra)?;
        }
        Ok(())
    }

    async fn append(&self, readings: &[RecordedReading]) -> InfraResult<u64> {
        let (astronaut, station): (Vec<_>, Vec<_>) = readings
            .iter()
            .partition(|r| matches!(r.subject(), Subject::Astronaut(_)));
        Ok(self.append_to(&ASTRONAUT, &astronaut).await?
            + self.append_to(&STATION, &station).await?)
    }

    async fn latest(&self, subject: Subject) -> InfraResult<Vec<(MetricCode, Reading)>> {
        let (t, id) = tables_for(subject);
        let rows: Vec<(String, DateTime<Utc>, Components)> = sqlx::query_as(&format!(
            "SELECT metric_code, recorded_at, components FROM {} WHERE {} = $1",
            t.latest, t.subject_col
        ))
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(infra)?;
        Ok(rows
            .into_iter()
            .filter_map(|(code, recorded_at, Json(values))| {
                Some((
                    metric_code(code)?,
                    Reading {
                        recorded_at,
                        values,
                    },
                ))
            })
            .collect())
    }

    async fn latest_for_station(
        &self,
        station: StationId,
    ) -> InfraResult<Vec<(AstronautId, MetricCode, Reading)>> {
        let rows: Vec<(Uuid, String, DateTime<Utc>, Components)> = sqlx::query_as(
            "SELECT l.astronaut_id, l.metric_code, l.recorded_at, l.components
             FROM latest_measurements l
             JOIN astronauts a ON a.id = l.astronaut_id
             WHERE a.station_id = $1",
        )
        .bind(station.0)
        .fetch_all(&self.pool)
        .await
        .map_err(infra)?;
        Ok(rows
            .into_iter()
            .filter_map(|(id, code, recorded_at, Json(values))| {
                Some((
                    AstronautId(id),
                    metric_code(code)?,
                    Reading {
                        recorded_at,
                        values,
                    },
                ))
            })
            .collect())
    }

    async fn series(
        &self,
        subject: Subject,
        metric: &MetricCode,
        range: TimeRange,
        bucket: Option<Bucket>,
    ) -> InfraResult<Vec<Reading>> {
        let (t, id) = tables_for(subject);
        let (history, col) = (t.history, t.subject_col);
        let filter = format!(
            "m.metric_code = $1 AND m.{col} = $2
              AND ($3::timestamptz IS NULL OR m.recorded_at >= $3)
              AND ($4::timestamptz IS NULL OR m.recorded_at <= $4)"
        );

        let query = match bucket {
            None => format!(
                "SELECT m.recorded_at, m.components FROM {history} m
                 WHERE {filter} ORDER BY m.recorded_at"
            ),
            // Average every component inside each time bucket. Generic over the
            // metric's components thanks to jsonb_each_text.
            Some(_) => format!(
                "SELECT b, jsonb_object_agg(k, v)
                 FROM (
                     SELECT date_bin(CAST($5 AS text)::interval, m.recorded_at,
                                     TIMESTAMPTZ '2000-01-03 00:00:00+00') AS b,
                            kv.key AS k, avg(kv.value::float8) AS v
                     FROM {history} m CROSS JOIN LATERAL jsonb_each_text(m.components) kv
                     WHERE {filter}
                     GROUP BY 1, 2
                 ) per_component
                 GROUP BY b ORDER BY b"
            ),
        };

        let mut q = sqlx::query_as::<_, (DateTime<Utc>, Components)>(&query)
            .bind(metric.as_str())
            .bind(id)
            .bind(range.from)
            .bind(range.to);
        if let Some(b) = bucket {
            q = q.bind(bucket_interval(b));
        }
        let rows = q.fetch_all(&self.pool).await.map_err(infra)?;
        Ok(rows
            .into_iter()
            .map(|(recorded_at, Json(values))| Reading {
                recorded_at,
                values,
            })
            .collect())
    }
}
