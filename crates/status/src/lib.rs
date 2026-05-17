#![forbid(unsafe_code)]

pub mod receipt;
pub mod status;
pub mod walk;

pub use logline_confirmed_by::{
    EvidenceKind, EvidenceReceipt, EvidenceRequirement, EvidenceResolution, EvidenceView,
};
pub use logline_did::{DidResolution, Verb, VerbClass};
pub use logline_if_doubt::{
    ClarificationRequest, DoubtKind, DoubtRoute, DoubtTrace, SimulationReceipt, SuspensionRecord,
};
pub use logline_if_not::{ProblemRecord, RejectionKind, RejectionRoute};
pub use logline_if_ok::{ReleaseAdapter, ReleaseKind, ReleaseRoute};
pub use logline_this::{LocatedMatter, MatterDigest, MatterRef};
pub use logline_when::{ClockView, FixedClock, TemporalBinding, TemporalResolution};
pub use logline_who::{
    is_form_require_directive, is_prohibited, parse_keyed_logline, parse_logline,
    parse_marked_logline, validate_against_canon, validate_canon, validate_shape,
    AccountableOrigin, ActorId, AuthorityId, Branch, Canon, Jurisdiction, LogLine, LogLineError,
    Mode, Operation, OriginKind, Phase, Principal, Result, RouteClass, RouteKinds, RunContext,
    RuntimeDecision, RuntimeSlot, SelectedBranch, Slot, SlotHouse, SlotResolution, StatusLifecycle,
    StatusTransition, WitnessKey,
};
pub use receipt::{
    canonical_json, receipt_hash, result_hash, ReceiptEncodingError, ReceiptEncodingResult,
};
pub use status::{
    canonical_tuple_digest, lifecycle_transition, CanonicalTuple, LedgerEntry, Lifecycle,
    LifecycleTransition, StatusLedger, StatusSlot,
};
pub use walk::{run, run_with_context, walk, RuntimeLogLine, RuntimeSlots};
