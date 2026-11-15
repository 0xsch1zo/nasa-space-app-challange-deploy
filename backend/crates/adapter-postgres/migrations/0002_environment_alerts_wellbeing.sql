-- Station-scoped history aggregates (environment) -----------------------------
--
-- Environment metrics (radiation, air quality, surface microbes, gravity)
-- describe a whole station, not one astronaut. Same layout as `measurements`:
-- ONE PARTITION PER METRIC, created by the application from the catalog
-- (station_measurements_radiation, ...), plus a latest-value projection.

CREATE TABLE station_measurements (
    metric_code TEXT        NOT NULL,
    station_id  UUID        NOT NULL REFERENCES stations (id) ON DELETE CASCADE,
    recorded_at TIMESTAMPTZ NOT NULL,
    components  JSONB       NOT NULL,
    source      TEXT        NOT NULL,
    ingested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (metric_code, station_id, recorded_at)
) PARTITION BY LIST (metric_code);

CREATE TABLE latest_station_measurements (
    station_id  UUID        NOT NULL REFERENCES stations (id) ON DELETE CASCADE,
    metric_code TEXT        NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL,
    components  JSONB       NOT NULL,
    PRIMARY KEY (station_id, metric_code)
);

-- Wellbeing notes ---------------------------------------------------------------
--
-- Gently worded descriptions of an astronaut's mental state, produced by an
-- external process and only read by this application. Kept as history.

CREATE TABLE wellbeing_notes (
    id           BIGSERIAL   PRIMARY KEY,
    astronaut_id UUID        NOT NULL REFERENCES astronauts (id) ON DELETE CASCADE,
    written_at   TIMESTAMPTZ NOT NULL,
    body         TEXT        NOT NULL,
    source       TEXT        NOT NULL DEFAULT 'wellbeing-model'
);
CREATE INDEX wellbeing_notes_latest_idx ON wellbeing_notes (astronaut_id, written_at DESC);
