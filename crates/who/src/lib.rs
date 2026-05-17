#![forbid(unsafe_code)]

pub mod canon;
pub mod error;
pub mod logline;
pub mod parse;
pub mod slot;
pub mod validate;
pub mod who;

pub use canon::{is_form_require_directive, Canon, RouteClass, RouteKinds, StatusLifecycle};
pub use error::{LogLineError, Result};
pub use logline::LogLine;
pub use parse::{parse_keyed_logline, parse_logline, parse_marked_logline};
pub use slot::{
    Branch, Mode, Operation, Phase, RunContext, RuntimeDecision, RuntimeSlot, SelectedBranch, Slot,
    SlotHouse, SlotResolution, StatusTransition,
};
pub use validate::{is_prohibited, validate_against_canon, validate_canon, validate_shape};
pub use who::{
    AccountableOrigin, ActorId, AuthorityId, Jurisdiction, OriginKind, Principal, WhoSlot,
    WitnessKey,
};
