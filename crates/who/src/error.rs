use thiserror::Error;

pub type Result<T> = std::result::Result<T, LogLineError>;

#[derive(Debug, Error)]
pub enum LogLineError {
    #[error("canon must be 'logline'")]
    InvalidCanonName,

    #[error("canon runtime_walk must equal the 9 canonical positions")]
    InvalidRuntimeWalk,

    #[error("canon form must contain exactly the 9 canonical positions")]
    InvalidCanonForm,

    #[error("canon law entry {0} is invalid: {1}")]
    InvalidLawEntry(usize, String),

    #[error("canon prohibition entry {0} is invalid: {1}")]
    InvalidProhibitionEntry(usize, String),

    #[error("logline field '{0}' must not be empty")]
    EmptyField(&'static str),

    #[error("{kind} identity must not be empty")]
    EmptyIdentity { kind: &'static str },

    #[error("invalid who origin: {0}")]
    InvalidWhoOrigin(String),

    #[error("logline matches a prohibited canon pattern")]
    Prohibited,

    #[error("unknown route token: {0}")]
    UnknownRouteToken(String),

    #[error("route token '{token}' belongs to {actual}, expected {expected}")]
    RouteClassMismatch {
        token: String,
        actual: &'static str,
        expected: &'static str,
    },

    #[error("unknown lifecycle status: {0}")]
    UnknownStatus(String),

    #[error("unknown LogLine position: {0}")]
    UnknownPosition(String),

    #[error("duplicate LogLine position: {0}")]
    DuplicatePosition(String),

    #[error("missing LogLine position: {0}")]
    MissingPosition(&'static str),

    #[error("runtime did not resolve slot: {0}")]
    MissingRuntimeSlot(&'static str),

    #[error("LogLine positions must appear in canonical order")]
    PositionOrder,

    #[error("unparsed LogLine material: {0}")]
    UnparsedMaterial(String),

    #[error("marked LogLine missing marker: {0}")]
    MissingMarker(&'static str),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
