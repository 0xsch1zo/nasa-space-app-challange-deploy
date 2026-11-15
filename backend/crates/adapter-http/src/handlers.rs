use application::ports::inbound::SeriesQuery;
use application::ports::outbound::{Bucket, TimeRange};
use application::AppError;
use axum::extract::{Path, Query, State};
use axum::Json;
use domain::crew::AstronautId;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::dto::*;
use crate::error::ApiResult;
use crate::extract::Auth;
use crate::AppState;

pub async fn liveness() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn login(
    State(s): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<LoginResponse> {
    let r = s.auth.login(&req.username, &req.password).await?;
    Ok(Json(LoginResponse {
        token: r.token.token,
        expires_at: r.token.expires_at,
        user: (&r.principal).into(),
    }))
}

pub async fn me(Auth(p): Auth) -> ApiResult<UserDto> {
    Ok(Json((&p).into()))
}

pub async fn metric_catalog(
    State(s): State<AppState>,
    Auth(_): Auth,
) -> ApiResult<Vec<MetricDefinitionDto>> {
    Ok(Json(
        s.health.catalog().all().iter().map(Into::into).collect(),
    ))
}

pub async fn list_crew(State(s): State<AppState>, Auth(p): Auth) -> ApiResult<Vec<CrewMemberDto>> {
    let crew = s.crew.list_crew(&p).await?;
    Ok(Json(crew.into_iter().map(Into::into).collect()))
}

pub async fn get_astronaut(
    State(s): State<AppState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<CrewMemberDto> {
    Ok(Json(
        s.crew.get_astronaut(&p, AstronautId(id)).await?.into(),
    ))
}

pub async fn overview(
    State(s): State<AppState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<OverviewDto> {
    Ok(Json(s.health.overview(&p, AstronautId(id)).await?.into()))
}

pub async fn series(
    State(s): State<AppState>,
    Auth(p): Auth,
    Path((id, code)): Path<(Uuid, String)>,
    Query(q): Query<SeriesParams>,
) -> ApiResult<SeriesDto> {
    let bucket = match q.bucket.as_deref() {
        None | Some("") | Some("raw") => None,
        Some(b) => Some(
            Bucket::parse(b)
                .ok_or_else(|| AppError::Validation(format!("unknown bucket `{b}`")))?,
        ),
    };
    let query = SeriesQuery {
        astronaut: AstronautId(id),
        metric: code,
        range: TimeRange {
            from: q.from,
            to: q.to,
        },
        bucket,
    };
    Ok(Json(s.health.series(&p, query).await?.into()))
}

pub async fn metric_groups(
    State(s): State<AppState>,
    Auth(_): Auth,
) -> ApiResult<Vec<MetricGroupDto>> {
    Ok(Json(
        s.health.catalog().groups().iter().map(Into::into).collect(),
    ))
}

pub async fn alerts(
    State(s): State<AppState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Vec<AlertDto>> {
    let alerts = s.alerts.alerts(&p, AstronautId(id)).await?;
    Ok(Json(alerts.into_iter().map(Into::into).collect()))
}

/// `null` when no note has been written yet.
pub async fn wellbeing_note(
    State(s): State<AppState>,
    Auth(p): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Option<WellbeingNoteDto>> {
    Ok(Json(
        s.wellbeing
            .latest_note(&p, AstronautId(id))
            .await?
            .map(Into::into),
    ))
}
