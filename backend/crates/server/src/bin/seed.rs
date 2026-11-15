//! Demo seeder: creates stations, crew, accounts, months of realistic history
//! for every metric in the catalog and wellbeing notes.
//!
//! Measurements are sent through the `MeasurementIngestion` input port, the
//! same entry point a real medical-device gateway would use, so they are
//! validated by the domain exactly like production data.
//!
//! The data is a function of the catalog: a new metric gets a plausible
//! history automatically (from its `DemoSignal`). Scenario tweaks below only
//! exist to demonstrate alerts of every level and one profile without alerts.
//!
//! Usage: `seed` (skips if data exists) or `seed --reset`.

use adapter_postgres::PgSeedWriter;
use application::ports::inbound::{DeviceMeasurement, MeasurementIngestion};
use application::ports::outbound::PasswordHasher;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use domain::crew::{AstronautId, StationId, Subject};
use domain::metrics::{MetricDefinition, MetricScope};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Normal};
use server::{build, init_tracing, prepare_database, Config};
use uuid::Uuid;

pub const DEMO_PASSWORD: &str = "orbita2026";

/// Adds `delta` to one component during the last `last_days` days
/// (e.g. a solar particle event, a few bad nights).
struct Shift {
    metric: &'static str,
    component: &'static str,
    last_days: f64,
    delta: f64,
}

const fn shift(metric: &'static str, component: &'static str, last_days: f64, delta: f64) -> Shift {
    Shift {
        metric,
        component,
        last_days,
        delta,
    }
}

/// How a subject's data deviates from the population defaults.
struct Profile {
    /// Per-metric multiplier of the physiological drift (`"*"` = every metric).
    /// Metrics not listed use 1.0, so new metrics get data automatically.
    risk: &'static [(&'static str, f64)],
    shifts: &'static [Shift],
    /// Metrics whose recording stopped N days ago (overdue sampling).
    paused: &'static [(&'static str, i64)],
}

impl Profile {
    fn risk(&self, metric: &str) -> f64 {
        let find = |k: &str| self.risk.iter().find(|(m, _)| *m == k).map(|(_, r)| *r);
        find(metric).or_else(|| find("*")).unwrap_or(1.0)
    }
}

struct StationSeed {
    name: &'static str,
    days_of_history: i64,
    profile: Profile,
}

const STATIONS: &[StationSeed] = &[
    StationSeed {
        name: "Stacja Orbitalna Aurora",
        days_of_history: 170,
        profile: Profile {
            risk: &[],
            // Ongoing solar particle event: dose rate ~3× the usual → warning.
            shifts: &[shift("radiation", "dose_rate", 2.5, 1.0)],
            // Surface swabs not collected for 20 days → "worth attention" notice.
            paused: &[("surface_microbes", 20)],
        },
    },
    StationSeed {
        name: "Lunar Gateway Selene",
        days_of_history: 70,
        profile: Profile {
            risk: &[],
            // Lunar orbit: higher galactic cosmic ray background, still normal.
            shifts: &[shift("radiation", "dose_rate", 1000.0, 0.22)],
            paused: &[],
        },
    },
];

struct CrewSeed {
    username: &'static str,
    name: &'static str,
    agency: &'static str,
    specialty: &'static str,
    station: usize,
    days_in_orbit: i64,
    profile: Profile,
    /// Most recent wellbeing note (older ones come from a neutral pool).
    note: &'static str,
}

const CREW: &[CrewSeed] = &[
    // Critical muscle atrophy + bone loss warning + station alerts: > 3 alerts.
    CrewSeed {
        username: "anna.kowalska",
        name: "Anna Kowalska",
        agency: "POLSA",
        specialty: "Inżynier pokładowy",
        station: 0,
        days_in_orbit: 168,
        profile: Profile {
            risk: &[("blood_pressure", 0.8), ("bone_density", 1.9), ("muscle_atrophy", 1.6)],
            shifts: &[],
            paused: &[],
        },
        note: "Ostatnie dni były intensywne i wypełnione pracą przy systemach stacji. \
               Zaangażowanie i dobra współpraca z załogą są wyraźnie widoczne. Chwila spokoju \
               po zakończeniu zmiany — muzyka, rozmowa z bliskimi czy widok Ziemi z kopuły — \
               może pomóc łagodnie wyciszyć się przed snem.",
    },
    // Several warnings: blood pressure, body temperature, a few short nights.
    CrewSeed {
        username: "marcus.reyes",
        name: "Marcus Reyes",
        agency: "NASA",
        specialty: "Dowódca",
        station: 0,
        days_in_orbit: 168,
        profile: Profile {
            risk: &[("blood_pressure", 2.6), ("body_temperature", 2.4), ("muscle_atrophy", 0.7), ("bone_density", 0.8)],
            shifts: &[
                shift("sleep", "total_sleep", 4.0, -1.7),
                shift("sleep", "efficiency", 4.0, -11.0),
                shift("sleep", "awakenings", 4.0, 2.0),
            ],
            paused: &[],
        },
        note: "Samopoczucie jest stabilne, a rola dowódcy daje poczucie sensu i odpowiedzialności. \
               Ostatnie noce były nieco krótsze niż zwykle, co przy tak wypełnionym grafiku jest \
               zupełnie zrozumiałe. Wspólny posiłek z załogą bywa dobrym momentem na złapanie oddechu.",
    },
    // Critical lab result (urinary calcium) + vitamin D warning.
    CrewSeed {
        username: "yuki.tanaka",
        name: "Yuki Tanaka",
        agency: "JAXA",
        specialty: "Specjalista misji",
        station: 0,
        days_in_orbit: 95,
        profile: Profile {
            risk: &[],
            shifts: &[
                shift("urine_panel", "calcium", 20.0, 135.0),
                shift("urine_panel", "ntx", 20.0, 34.0),
                shift("blood_panel", "vitamin_d", 40.0, -17.0),
            ],
            paused: &[],
        },
        note: "Nastrój jest pogodny, z dużą ciekawością wobec prowadzonych eksperymentów. \
               Rytm dnia jest uporządkowany, a regularny kontakt z zespołem na Ziemi daje dobre \
               poczucie wsparcia.",
    },
    // Personally fine; only the station environment alerts apply (< 3 alerts).
    CrewSeed {
        username: "lena.hoffmann",
        name: "Lena Hoffmann",
        agency: "ESA",
        specialty: "Lekarz pokładowy",
        station: 0,
        days_in_orbit: 31,
        profile: Profile { risk: &[("muscle_atrophy", 0.8)], shifts: &[], paused: &[] },
        note: "Pierwsze tygodnie na stacji przebiegają spokojnie. Adaptacja do nowego otoczenia \
               postępuje naturalnym tempem, a codzienne drobne rytuały pomagają poczuć się \
               tu jak w domu.",
    },
    // No alerts at all: everything in range, station environment normal.
    CrewSeed {
        username: "omar.haddad",
        name: "Omar Haddad",
        agency: "ESA",
        specialty: "Pilot",
        station: 1,
        days_in_orbit: 64,
        profile: Profile { risk: &[("*", 0.4)], shifts: &[], paused: &[] },
        note: "Samopoczucie jest dobre i zrównoważone. Obserwacje Księżyca są źródłem dużej \
               satysfakcji, a regularne rozmowy z rodziną wspierają dobry nastrój.",
    },
];

/// Neutral, gentle notes used for the earlier weeks of every mission.
const NOTE_POOL: &[&str] = &[
    "Tydzień upłynął w spokojnym rytmie. Praca i odpoczynek są w dobrej równowadze, \
     a kontakt z bliskimi przebiega regularnie.",
    "Adaptacja do życia na stacji przebiega płynnie. Widać zaangażowanie w obowiązki \
     i otwartość na współpracę z załogą.",
    "Nastrój jest pogodny. Krótkie przerwy w ciągu dnia i wspólne chwile z załogą \
     sprzyjają dobremu samopoczuciu.",
    "Dni są wypełnione pracą, a wieczory pozwalają na odpoczynek. Ogólne samopoczucie \
     pozostaje stabilne.",
];

struct SupervisorSeed {
    username: &'static str,
    name: &'static str,
    station: usize,
}

const SUPERVISORS: &[SupervisorSeed] = &[
    SupervisorSeed {
        username: "lekarz.aurora",
        name: "dr Ewa Nowak (lekarz lotu)",
        station: 0,
    },
    SupervisorSeed {
        username: "lekarz.selene",
        name: "dr Tomás Ruiz (lekarz lotu)",
        station: 1,
    },
];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let reset = std::env::args().any(|a| a == "--reset");
    let config = Config::from_env()?;
    let pool = adapter_postgres::connect(&config.database_url).await?;
    let container = build(pool.clone(), &config);
    prepare_database(&pool, &container.catalog).await?;

    let writer = PgSeedWriter::new(pool.clone());
    if writer.has_users().await? {
        if !reset {
            tracing::info!("database already seeded, skipping (use --reset to re-create)");
            return Ok(());
        }
        writer.reset().await?;
    }

    let now = Utc::now();
    let catalog = container.catalog.clone();
    let mut batch = Vec::new();

    let station_ids: Vec<Uuid> = STATIONS
        .iter()
        .map(|s| stable_id("station", s.name))
        .collect();
    for (idx, (id, s)) in station_ids.iter().zip(STATIONS).enumerate() {
        writer.station(*id, s.name).await?;
        let mut rng = StdRng::seed_from_u64(500 + idx as u64);
        for def in catalog
            .all()
            .iter()
            .filter(|d| d.scope == MetricScope::Station)
        {
            let subject = Subject::Station(StationId(*id));
            batch.extend(simulate(
                def,
                subject,
                s.days_of_history,
                &s.profile,
                now,
                &mut rng,
            ));
        }
    }

    let hash = container
        .hasher
        .hash(DEMO_PASSWORD)
        .map_err(|e| anyhow::anyhow!(e))?;
    for s in SUPERVISORS {
        writer
            .user(
                s.username,
                s.name,
                &hash,
                "supervisor",
                station_ids[s.station],
                None,
            )
            .await?;
    }

    for (idx, c) in CREW.iter().enumerate() {
        let id = stable_id("astronaut", c.username);
        let start: NaiveDate = (now - Duration::days(c.days_in_orbit - 1)).date_naive();
        writer
            .astronaut(
                id,
                station_ids[c.station],
                c.name,
                c.agency,
                c.specialty,
                start,
            )
            .await?;
        writer
            .user(
                c.username,
                c.name,
                &hash,
                "astronaut",
                station_ids[c.station],
                Some(id),
            )
            .await?;

        let mut rng = StdRng::seed_from_u64(1000 + idx as u64);
        for def in catalog
            .all()
            .iter()
            .filter(|d| d.scope == MetricScope::Astronaut)
        {
            let subject = Subject::Astronaut(AstronautId(id));
            batch.extend(simulate(
                def,
                subject,
                c.days_in_orbit,
                &c.profile,
                now,
                &mut rng,
            ));
        }

        // Weekly wellbeing notes; the newest one is personal.
        let mut week = 1;
        while week * 7 < c.days_in_orbit {
            let at = now - Duration::days(c.days_in_orbit - week * 7) + Duration::hours(2);
            writer
                .wellbeing_note(
                    id,
                    at.min(now),
                    NOTE_POOL[(week as usize + idx) % NOTE_POOL.len()],
                )
                .await?;
            week += 1;
        }
        writer
            .wellbeing_note(id, now - Duration::hours(6), c.note)
            .await?;
    }

    let total = batch.len();
    let mut accepted = 0;
    for chunk in batch.chunks(2000) {
        let report = container.ingestion.ingest(chunk.to_vec()).await?;
        accepted += report.accepted;
        for (i, why) in report.rejected {
            tracing::warn!(index = i, reason = %why, "measurement rejected");
        }
    }
    tracing::info!(accepted, total, "seeded measurements");

    println!("\nSeed complete. Demo accounts (password: {DEMO_PASSWORD}):");
    for s in SUPERVISORS {
        println!(
            "  supervisor  {:<16} {}",
            s.username, STATIONS[s.station].name
        );
    }
    for c in CREW {
        println!(
            "  astronaut   {:<16} {}",
            c.username, STATIONS[c.station].name
        );
    }
    Ok(())
}

/// Generic signal generator driven by each component's `DemoSignal`.
fn simulate(
    def: &MetricDefinition,
    subject: Subject,
    days: i64,
    profile: &Profile,
    now: DateTime<Utc>,
    rng: &mut StdRng,
) -> Vec<DeviceMeasurement> {
    let first_day = (now - Duration::days(days - 1)).date_naive();
    let start = first_day.and_hms_opt(0, 0, 0).expect("midnight").and_utc();
    let step = Duration::hours(i64::from(def.sampling_interval_hours.max(1)));
    let risk = profile.risk(def.code.as_str());
    let stop = profile
        .paused
        .iter()
        .find(|(m, _)| *m == def.code.as_str())
        .map(|(_, d)| now - Duration::days(*d))
        .unwrap_or(now);
    // Derived components (e.g. sleep score) are computed by the domain.
    let components: Vec<_> = def.components.iter().filter(|c| !c.derived).collect();
    // Stable per-subject deviation from the population baseline.
    let offsets: Vec<f64> = components
        .iter()
        .map(|c| normal(c.demo.spread).sample(rng))
        .collect();

    let mut out = Vec::new();
    let mut t = start + Duration::hours(7); // morning measurement, 07:00 UTC
    while t <= stop {
        let jitter = Duration::minutes(rng.gen_range(-40..=40));
        let at = (t + jitter).min(now);
        let day = (at - start).num_seconds() as f64 / 86_400.0;
        let days_ago = (now - at).num_seconds() as f64 / 86_400.0;
        let values = components
            .iter()
            .zip(&offsets)
            .map(|(c, off)| {
                let s = c.demo;
                let shift: f64 = profile
                    .shifts
                    .iter()
                    .filter(|x| {
                        x.metric == def.code.as_str()
                            && x.component == c.key
                            && days_ago <= x.last_days
                    })
                    .map(|x| x.delta)
                    .sum();
                let raw = s.baseline
                    + off
                    + s.drift_per_day * day * risk
                    + shift
                    + normal(s.noise).sample(rng);
                let lo = c.plausible.min.unwrap_or(f64::MIN);
                let hi = c.plausible.max.unwrap_or(f64::MAX);
                let factor = 10f64.powi(i32::from(c.decimals));
                (c.key.clone(), (raw.clamp(lo, hi) * factor).round() / factor)
            })
            .collect();
        out.push(DeviceMeasurement {
            subject,
            metric: def.code.to_string(),
            recorded_at: at,
            values,
            source: format!("sim-{}", def.code),
        });
        t += step;
    }
    out
}

/// Deterministic id derived from a name: re-seeding (`--reset`) keeps the
/// same ids, so tokens issued before a reset still point at existing data.
fn stable_id(kind: &str, name: &str) -> Uuid {
    const NAMESPACE: Uuid = Uuid::from_u128(0x6a1f_3c2e_8b4d_4e7a_9c5f_0d2b_7e81_a4c3);
    Uuid::new_v5(&NAMESPACE, format!("{kind}:{name}").as_bytes())
}

fn normal(sd: f64) -> Normal<f64> {
    Normal::new(0.0, sd.max(f64::EPSILON)).expect("valid normal distribution")
}
