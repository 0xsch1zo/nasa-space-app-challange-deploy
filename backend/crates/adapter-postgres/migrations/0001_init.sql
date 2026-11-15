-- Stations, crew and accounts -------------------------------------------------

CREATE TABLE stations (
    id   UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE astronauts (
    id            UUID PRIMARY KEY,
    station_id    UUID NOT NULL REFERENCES stations (id),
    full_name     TEXT NOT NULL,
    agency        TEXT NOT NULL,
    specialty     TEXT NOT NULL DEFAULT '',
    mission_start DATE NOT NULL
);
CREATE INDEX astronauts_station_idx ON astronauts (station_id);

CREATE TABLE users (
    id            UUID PRIMARY KEY,
    username      TEXT NOT NULL UNIQUE,
    display_name  TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    role          TEXT NOT NULL CHECK (role IN ('astronaut', 'supervisor')),
    station_id    UUID NOT NULL REFERENCES stations (id),
    astronaut_id  UUID UNIQUE REFERENCES astronauts (id) ON DELETE CASCADE,
    -- astronaut accounts are linked to exactly one crew member, supervisors to none
    CHECK ((role = 'astronaut') = (astronaut_id IS NOT NULL))
);

-- Health history aggregates ---------------------------------------------------
--
-- One logical table, physically split into ONE PARTITION PER METRIC
-- (measurements_blood_pressure, measurements_bone_density, ...). Partitions are
-- created by the application at startup from the metric catalog
-- (MeasurementRepository::ensure_storage), so adding a metric needs no migration.
--
-- `components` holds the metric's values keyed by component, e.g.
-- {"systolic": 118, "diastolic": 76}; the schema is enforced by the domain.
-- The primary key doubles as the time-series index:
--   WHERE metric_code = ? AND astronaut_id = ? ORDER BY recorded_at

CREATE TABLE measurements (
    metric_code  TEXT        NOT NULL,
    astronaut_id UUID        NOT NULL REFERENCES astronauts (id) ON DELETE CASCADE,
    recorded_at  TIMESTAMPTZ NOT NULL,
    components   JSONB       NOT NULL,
    source       TEXT        NOT NULL,
    ingested_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (metric_code, astronaut_id, recorded_at)
) PARTITION BY LIST (metric_code);

-- Projection with the newest reading per (astronaut, metric): the dashboard
-- overview, crew list and body map read one row per metric instead of
-- scanning history. Maintained transactionally by the repository on append.
CREATE TABLE latest_measurements (
    astronaut_id UUID        NOT NULL REFERENCES astronauts (id) ON DELETE CASCADE,
    metric_code  TEXT        NOT NULL,
    recorded_at  TIMESTAMPTZ NOT NULL,
    components   JSONB       NOT NULL,
    PRIMARY KEY (astronaut_id, metric_code)
);
