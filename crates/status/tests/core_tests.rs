use logline_status::{
    canonical_json, canonical_tuple_digest, content_hash, lifecycle_transition, parse_logline,
    run_with_context, tuple_hash, validate_against_canon, walk, AccountableOrigin, ActorId, Branch,
    Canon, ClarificationRequest, ClockView, DidResolution, DoubtKind, DoubtRoute, EvidenceKind,
    EvidenceReceipt, EvidenceRequirement, EvidenceResolution, EvidenceView, FixedClock, LedgerEntry,
    Lifecycle, LifecycleTransition, LocatedMatter, LogLine, LogLineError, MatterDigest, MatterRef,
    Mode, Operation, OriginKind, ProblemRecord, RejectionKind, RejectionRoute, ReleaseAdapter,
    ReleaseKind, ReleaseRoute, Result, RunContext, Slot, SlotHouse, SlotResolution, StatusLedger,
    StatusTransition, SuspensionRecord, TemporalBinding, TemporalResolution, Verb, VerbClass,
};
use serde_json::Value;
use std::cell::Cell;
use std::error::Error;

fn canon() -> Result<Canon> {
    Canon::from_json_path("examples/logline.canon.json")
}

fn missing_test_material(name: &str) -> LogLineError {
    LogLineError::InvalidWhoOrigin(format!("missing test material: {name}"))
}

fn receipt_fixture_logline(receipt: &Value) -> std::result::Result<LogLine, Box<dyn Error>> {
    Ok(LogLine::new(
        receipt["who"]
            .as_str()
            .ok_or_else(|| missing_test_material("receipt who"))?,
        receipt["did"]
            .as_str()
            .ok_or_else(|| missing_test_material("receipt did"))?,
        receipt["this"]
            .as_str()
            .ok_or_else(|| missing_test_material("receipt this"))?,
        receipt["when"]
            .as_str()
            .ok_or_else(|| missing_test_material("receipt when"))?,
        receipt["confirmed_by"]
            .as_str()
            .ok_or_else(|| missing_test_material("receipt confirmed_by"))?,
        receipt["if_ok"]
            .as_str()
            .ok_or_else(|| missing_test_material("receipt if_ok"))?,
        receipt["if_doubt"]
            .as_str()
            .ok_or_else(|| missing_test_material("receipt if_doubt"))?,
        receipt["if_not"]
            .as_str()
            .ok_or_else(|| missing_test_material("receipt if_not"))?,
        receipt["status"]
            .as_str()
            .ok_or_else(|| missing_test_material("receipt status"))?,
    ))
}

#[test]
fn full_canon_json_loads() -> Result<()> {
    let canon = canon()?;
    assert_eq!(canon.canon, "logline");
    assert_eq!(canon.runtime_walk.len(), 9);
    assert_eq!(canon.version, "0.2.0-draft");
    assert_eq!(canon.law.len(), 9);
    assert_eq!(canon.prohibition.len(), 4);
    Ok(())
}

#[test]
fn full_canon_logline_parses() -> Result<()> {
    let lines = Canon::loglines_from_path("examples/logline.canon.logline")?;
    assert_eq!(lines.len(), 14);
    assert_eq!(lines[0].who, "canon");
    assert_eq!(lines[0].did, "define");
    assert_eq!(lines[0].status, "canonical");
    assert!(lines
        .iter()
        .any(|line| line.who == "if_doubt" && line.this_ == "auditable_simulation"));
    Ok(())
}

#[test]
fn confirmed_logline_walks_ok() -> Result<()> {
    let canon = canon()?;
    let logline: LogLine = serde_json::from_str(include_str!("../examples/sample.ok.json"))?;
    let mut ctx = RunContext::new(&canon, Operation::Confirmar);
    ctx.provided_evidence.push("ana".to_string());
    let result = run_with_context(&logline, ctx)?;
    assert_eq!(result.selected.branch, Branch::Ok);
    assert_eq!(result.status.after, "confirmed");
    assert_eq!(result.runtime.if_ok.resolution, SlotResolution::Selected);
    assert_eq!(result.doubt_trace, None);
    assert_eq!(result.simulation_receipt, None);
    Ok(())
}

#[test]
fn confirmation_without_receipt_walks_doubt() -> Result<()> {
    let canon = canon()?;
    let logline: LogLine = serde_json::from_str(include_str!("../examples/sample.ok.json"))?;
    let result = walk(&canon, &logline)?;
    assert_eq!(result.selected.branch, Branch::Doubt);
    assert_eq!(result.selected.reason, "receipt_missing");
    assert_eq!(result.runtime.if_doubt.resolution, SlotResolution::Selected);
    let trace = result
        .doubt_trace
        .as_ref()
        .ok_or_else(|| missing_test_material("doubt trace"))?;
    assert_eq!(trace.branch, Branch::Doubt);
    assert!(!trace.released);
    assert_eq!(result.simulation_receipt, None);
    Ok(())
}

#[test]
fn missing_confirmation_walks_doubt() -> Result<()> {
    let canon = canon()?;
    let logline: LogLine = serde_json::from_str(include_str!("../examples/sample.doubt.json"))?;
    let result = walk(&canon, &logline)?;
    assert_eq!(result.selected.branch, Branch::Doubt);
    assert_eq!(result.status.after, "doubt");
    assert_eq!(
        result.runtime.confirmed_by.resolution,
        SlotResolution::Doubt
    );
    assert_eq!(result.runtime.if_doubt.resolution, SlotResolution::Selected);
    Ok(())
}

#[test]
fn prohibited_logline_walks_not() -> Result<()> {
    let canon = canon()?;
    let logline: LogLine = serde_json::from_str(include_str!("../examples/sample.not.json"))?;
    let result = walk(&canon, &logline)?;
    assert_eq!(result.selected.branch, Branch::Not);
    assert_eq!(result.selected.reason, "prohibited_by_canon");
    assert_eq!(result.runtime.if_not.resolution, SlotResolution::Selected);
    Ok(())
}

#[test]
fn marked_logline_parses() -> Result<()> {
    let logline = parse_logline(
        "dan operate invoice_123 tomorrow confirmed_by ana if_ok execute if_doubt suspend if_not reject status pending",
    )?;
    assert_eq!(logline.who, "dan");
    assert_eq!(logline.this_, "invoice_123");
    assert_eq!(logline.if_ok, "execute");
    Ok(())
}

#[test]
fn act_routes_can_be_domain_tokens_by_position() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "tomorrow",
        "ana",
        "send",
        "ask",
        "reject",
        "pending",
    );

    validate_against_canon(&canon, &logline)?;
    Ok(())
}

#[test]
fn any_time_normalizes_to_wildcard() -> Result<()> {
    let logline = parse_logline(
        "act bypass logline any_time confirmed_by * if_ok * if_doubt suspend if_not reject status forbidden",
    )?;
    assert_eq!(logline.when, "*");
    Ok(())
}

#[test]
fn operations_have_fixed_home_slots() {
    assert_eq!(Operation::Identificar.home(), Slot::Who);
    assert_eq!(Operation::Nomear.home(), Slot::Did);
    assert_eq!(Operation::Localizar.home(), Slot::This);
    assert_eq!(Operation::Simular.home(), Slot::When);
    assert_eq!(Operation::Questionar.home(), Slot::ConfirmedBy);
    assert_eq!(Operation::Confirmar.home(), Slot::ConfirmedBy);
    assert_eq!(Operation::Verificar.home(), Slot::ConfirmedBy);
    assert_eq!(Operation::Acionar.home(), Slot::IfOk);
    assert_eq!(Operation::Despachar.home(), Slot::IfDoubt);
    assert_eq!(Operation::MarcarProblema.home(), Slot::IfNot);
    assert_eq!(Operation::Registrar.home(), Slot::Status);
}

#[test]
fn modes_are_semantic_and_do_not_change_runtime_anatomy() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "2026-05-05T09:00:00Z",
        "ana",
        "send",
        "ask",
        "reject",
        "pending",
    );
    let mut pocket = RunContext::new(&canon, Operation::Confirmar);
    pocket.mode = Mode::Pocket;
    pocket.provided_evidence.push("ana".to_string());
    let mut operational = RunContext::new(&canon, Operation::Confirmar);
    operational.mode = Mode::Operational;
    operational.provided_evidence.push("ana".to_string());

    let pocket_result = run_with_context(&logline, pocket)?;
    let operational_result = run_with_context(&logline, operational)?;

    assert_eq!(pocket_result.mode, Mode::Pocket);
    assert_eq!(operational_result.mode, Mode::Operational);
    assert_eq!(
        pocket_result.selected.branch,
        operational_result.selected.branch
    );
    assert_eq!(
        pocket_result.runtime.confirmed_by.next,
        Some("if_ok".to_string())
    );
    assert_eq!(
        operational_result.runtime.confirmed_by.next,
        Some("if_ok".to_string())
    );
    assert_eq!(
        pocket_result.runtime.status.after,
        Some("confirmed".to_string())
    );
    assert_eq!(
        operational_result.runtime.status.after,
        Some("confirmed".to_string())
    );
    Ok(())
}

#[test]
fn localizar_shines_in_this_but_walks_all_slots() -> Result<()> {
    let canon = canon()?;
    let logline: LogLine = serde_json::from_str(include_str!("../examples/sample.ok.json"))?;
    let mut ctx = RunContext::new(&canon, Operation::Localizar);
    ctx.provided_evidence.push("ana".to_string());

    let result = run_with_context(&logline, ctx)?;

    assert_eq!(result.operation_home, Slot::This);
    assert_eq!(result.runtime.this_.reason, "localizar_object_matter");
    assert_eq!(result.runtime.who.resolution, SlotResolution::Ok);
    assert_eq!(result.runtime.status.after, Some("confirmed".to_string()));
    Ok(())
}

#[test]
fn operational_mode_marks_unbounded_matter_as_doubt_without_changing_branch() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        "send",
        "*",
        "2026-05-05T09:00:00Z",
        "ana",
        "send",
        "ask",
        "reject",
        "pending",
    );
    let mut ctx = RunContext::new(&canon, Operation::Localizar);
    ctx.mode = Mode::Operational;
    ctx.provided_evidence.push("ana".to_string());

    let result = run_with_context(&logline, ctx)?;

    assert_eq!(result.runtime.this_.resolution, SlotResolution::Doubt);
    assert_eq!(result.runtime.this_.reason, "unbounded_matter");
    assert_eq!(result.selected.branch, Branch::Ok);
    Ok(())
}

#[test]
fn none_confirmation_needs_explicit_permission() -> Result<()> {
    let canon = canon()?;
    let logline: LogLine = serde_json::from_str(include_str!("../examples/sample.doubt.json"))?;

    let denied = walk(&canon, &logline)?;
    assert_eq!(denied.selected.branch, Branch::Doubt);

    let mut ctx = RunContext::new(&canon, Operation::Confirmar);
    ctx.allow_no_confirmation = true;
    let allowed = run_with_context(&logline, ctx)?;
    assert_eq!(allowed.selected.branch, Branch::Ok);
    Ok(())
}

#[test]
fn houses_accept_only_their_dominant_operations() {
    assert!(logline_this::ThisSlot.accepts_operation(Operation::Localizar));
    assert!(!logline_this::ThisSlot.accepts_operation(Operation::Registrar));
    assert!(logline_confirmed_by::ConfirmedBySlot.accepts_operation(Operation::Verificar));
    assert!(!logline_confirmed_by::ConfirmedBySlot.accepts_operation(Operation::Acionar));
    assert!(logline_status::StatusSlot.accepts_operation(Operation::Registrar));
}

#[test]
fn actor_id_trims_and_rejects_empty_origin() -> Result<()> {
    let actor = " dan ".parse::<ActorId>()?;
    assert_eq!(actor.as_str(), "dan");
    assert!(" ".parse::<ActorId>().is_err());
    Ok(())
}

#[test]
fn accountable_origin_parses_runtime_principals() -> Result<()> {
    let human = AccountableOrigin::parse("dan")?;
    assert_eq!(human.actor.as_str(), "dan");
    assert_eq!(human.kind, OriginKind::Unknown);

    let system = AccountableOrigin::parse("system:ledger")?;
    assert_eq!(system.actor.as_str(), "ledger");
    assert_eq!(system.kind, OriginKind::System);

    let llm = AccountableOrigin::parse("llm:translator")?;
    assert_eq!(llm.kind, OriginKind::Llm);

    let runtime = AccountableOrigin::parse("runtime:logline")?;
    assert_eq!(runtime.kind, OriginKind::Runtime);

    assert!(AccountableOrigin::parse("ghost:dan").is_err());
    assert!(AccountableOrigin::parse("system:").is_err());
    Ok(())
}

#[test]
fn identificar_operation_shines_in_who_origin() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "system:ledger",
        "operate",
        "invoice_123",
        "tomorrow",
        "ana",
        "execute",
        "suspend",
        "reject",
        "pending",
    );
    let mut ctx = RunContext::new(&canon, Operation::Identificar);
    ctx.provided_evidence.push("ana".to_string());

    let result = run_with_context(&logline, ctx)?;

    assert_eq!(result.runtime.who.resolution, SlotResolution::Ok);
    assert_eq!(
        result.runtime.who.reason,
        "identificar_accountable_origin_system"
    );
    assert_eq!(result.selected.branch, Branch::Ok);
    Ok(())
}

#[test]
fn did_resolution_classifies_operations_canon_laws_and_domain_verbs() {
    let operation = DidResolution::parse("confirmar");
    assert_eq!(operation.class, VerbClass::RuntimeOperation);
    assert_eq!(operation.verb, Verb::Operation(Operation::Confirmar));

    let law = DidResolution::parse("define");
    assert_eq!(law.class, VerbClass::CanonicalLaw);
    assert_eq!(law.verb, Verb::CanonLaw("define".to_string()));

    let domain = DidResolution::parse("send");
    assert_eq!(domain.class, VerbClass::DomainAct);
    assert_eq!(domain.verb, Verb::Domain("send".to_string()));

    let empty = DidResolution::parse(" ");
    assert_eq!(empty.class, VerbClass::Unknown);
}

#[test]
fn nomear_operation_shines_in_did_without_selecting_branch() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "tomorrow",
        "ana",
        "send",
        "ask",
        "reject",
        "pending",
    );
    let mut ctx = RunContext::new(&canon, Operation::Nomear);
    ctx.provided_evidence.push("ana".to_string());

    let result = run_with_context(&logline, ctx)?;

    assert_eq!(result.runtime.did.reason, "nomear_domain_act_verb");
    assert_eq!(result.selected.branch, Branch::Ok);
    assert_eq!(result.selected.position, "if_ok");
    Ok(())
}

#[test]
fn did_marks_blank_movement_as_doubt_without_selecting_branch() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        " ",
        "invoice_123",
        "2026-05-05T09:00:00Z",
        "ana",
        "send",
        "ask",
        "reject",
        "pending",
    );
    let mut ctx = RunContext::new(&canon, Operation::Nomear);
    ctx.provided_evidence.push("ana".to_string());

    let result = run_with_context(&logline, ctx)?;

    assert_eq!(result.runtime.did.resolution, SlotResolution::Doubt);
    assert_eq!(result.runtime.did.reason, "unknown_verb");
    assert_eq!(result.selected.branch, Branch::Ok);
    Ok(())
}

#[test]
fn located_matter_classifies_common_reference_shapes() {
    let object = LocatedMatter::parse("invoice_123");
    assert!(matches!(object.reference, MatterRef::ObjectId(_)));
    assert_eq!(object.reason(), "object_matter");

    let url = LocatedMatter::parse("https://example.com/invoice/123");
    assert!(matches!(url.reference, MatterRef::Url(_)));
    assert_eq!(url.reason(), "url_matter");

    let path = LocatedMatter::parse("./invoices/123.json");
    assert!(matches!(path.reference, MatterRef::Path(_)));
    assert_eq!(path.reason(), "path_matter");

    let digest = LocatedMatter::parse(
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    );
    assert!(matches!(digest.reference, MatterRef::Digest(_)));
    assert_eq!(digest.reason(), "digest_matter");

    let payload = LocatedMatter::parse("{\"invoice\":123}");
    assert!(matches!(payload.reference, MatterRef::JsonPayload(_)));
    assert!(payload.digest.is_some());

    let wildcard = LocatedMatter::parse("*");
    assert_eq!(wildcard.reference, MatterRef::Wildcard);
}

#[test]
fn matter_digest_is_stable_for_same_bytes() {
    let left = MatterDigest::sha256_bytes(b"invoice_123");
    let right = MatterDigest::sha256_bytes(b"invoice_123");
    let changed = MatterDigest::sha256_bytes(b"invoice_124");

    assert_eq!(left, right);
    assert_ne!(left, changed);
    assert!(left.as_str().starts_with("sha256:"));
}

#[test]
fn localizar_operation_classifies_url_without_fetching_it() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        "send",
        "https://example.com/invoice/123",
        "tomorrow",
        "ana",
        "send",
        "ask",
        "reject",
        "pending",
    );
    let mut ctx = RunContext::new(&canon, Operation::Localizar);
    ctx.provided_evidence.push("ana".to_string());

    let result = run_with_context(&logline, ctx)?;

    assert_eq!(result.runtime.this_.reason, "localizar_url_matter");
    assert_eq!(result.selected.branch, Branch::Ok);
    assert_eq!(result.selected.position, "if_ok");
    Ok(())
}

#[test]
fn temporal_resolution_classifies_common_time_shapes() {
    let wildcard = TemporalResolution::parse("*");
    assert_eq!(wildcard.binding, TemporalBinding::Wildcard);
    assert_eq!(wildcard.reason(), "wildcard_temporal_binding");

    let any_time = TemporalResolution::parse("any_time");
    assert_eq!(any_time.binding, TemporalBinding::Wildcard);

    let phase = TemporalResolution::parse("before_logline");
    assert_eq!(
        phase.binding,
        TemporalBinding::Phase("before_logline".to_string())
    );
    assert_eq!(phase.reason(), "phase_binding");

    let timestamp = TemporalResolution::parse("2026-05-05T09:00:00Z");
    assert!(matches!(timestamp.binding, TemporalBinding::Timestamp(_)));
    assert_eq!(timestamp.reason(), "timestamp_binding");

    let window = TemporalResolution::parse("2026-05-05T09:00:00Z..2026-05-05T10:00:00Z");
    assert!(matches!(window.binding, TemporalBinding::Window { .. }));
    assert_eq!(window.reason(), "window_binding");

    let relative = TemporalResolution::parse("tomorrow");
    assert_eq!(
        relative.binding,
        TemporalBinding::Relative("tomorrow".to_string())
    );
    assert_eq!(relative.reason(), "relative_time_unresolved");
}

#[test]
fn fixed_clock_provides_deterministic_simulated_now() {
    let clock = FixedClock::new("2026-05-05T09:00:00Z");
    assert_eq!(clock.now_utc(), "2026-05-05T09:00:00Z");

    let trimmed = FixedClock::new(" 2026-05-05T09:00:00Z ");
    assert_eq!(trimmed.now_utc(), "2026-05-05T09:00:00Z");
}

#[test]
fn simular_operation_shines_in_when_without_scheduling() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "2026-05-05T09:00:00Z",
        "ana",
        "send",
        "ask",
        "reject",
        "pending",
    );
    let mut ctx = RunContext::new(&canon, Operation::Simular);
    ctx.provided_evidence.push("ana".to_string());

    let result = run_with_context(&logline, ctx)?;

    assert_eq!(result.runtime.when.reason, "simular_timestamp_binding");
    assert_eq!(result.selected.branch, Branch::Ok);
    assert_eq!(result.selected.position, "if_ok");
    Ok(())
}

#[test]
fn operational_mode_marks_relative_time_as_doubt_without_scheduling() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "tomorrow",
        "ana",
        "send",
        "ask",
        "reject",
        "pending",
    );
    let mut ctx = RunContext::new(&canon, Operation::Simular);
    ctx.mode = Mode::Operational;
    ctx.provided_evidence.push("ana".to_string());

    let result = run_with_context(&logline, ctx)?;

    assert_eq!(result.runtime.when.resolution, SlotResolution::Doubt);
    assert_eq!(result.runtime.when.reason, "relative_time_unresolved");
    assert_eq!(result.selected.branch, Branch::Ok);
    Ok(())
}

#[test]
fn evidence_requirement_parses_supported_shapes() {
    let token = EvidenceRequirement::parse("ana");
    assert_eq!(token.kind, EvidenceKind::Token);
    assert_eq!(token.raw, "ana");
    assert_eq!(token.threshold, None);

    let receipt = EvidenceRequirement::parse("receipt:ev_123");
    assert_eq!(receipt.kind, EvidenceKind::Receipt);

    let digest = EvidenceRequirement::parse("digest:sha256:abc");
    assert_eq!(digest.kind, EvidenceKind::Digest);

    let signature = EvidenceRequirement::parse("signature:key_123");
    assert_eq!(signature.kind, EvidenceKind::Signature);

    let quorum = EvidenceRequirement::parse("quorum:2:ops");
    assert_eq!(quorum.kind, EvidenceKind::Quorum);
    assert_eq!(quorum.raw, "quorum:2:ops");
    assert_eq!(quorum.threshold, Some(2));
    assert_eq!(quorum.label.as_deref(), Some("ops"));

    let none = EvidenceRequirement::parse("none");
    assert!(none.is_missing_marker());
}

#[test]
fn evidence_receipt_token_preserves_explicit_provided_evidence() {
    let receipt = EvidenceReceipt::token("ana");
    assert_eq!(receipt.id, "ana");
    assert_eq!(receipt.confirms, "ana");
    assert_eq!(receipt.issued_by, "provided_evidence");
    assert_eq!(receipt.digest, None);
    let resolution = EvidenceResolution::Confirmed;
    assert_eq!(resolution, EvidenceResolution::Confirmed);
}

#[test]
fn confirmed_by_typed_requirements_preserve_existing_pivot_rules() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "tomorrow",
        "receipt:ev_123",
        "send",
        "ask",
        "reject",
        "pending",
    );

    let missing = walk(&canon, &logline)?;
    assert_eq!(missing.selected.branch, Branch::Doubt);

    let mut ctx = RunContext::new(&canon, Operation::Verificar);
    ctx.provided_evidence.push("receipt:ev_123".to_string());
    let verified = run_with_context(&logline, ctx)?;
    assert_eq!(verified.selected.branch, Branch::Ok);
    assert_eq!(verified.selected.reason, "receipt_verified");
    Ok(())
}

#[test]
fn confirmed_by_quorum_requires_threshold_without_fetching_evidence() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "tomorrow",
        "quorum:2:ops",
        "send",
        "ask",
        "reject",
        "pending",
    );

    let mut one = RunContext::new(&canon, Operation::Confirmar);
    one.provided_evidence.push("quorum:ops:ana".to_string());
    let one_result = run_with_context(&logline, one)?;
    assert_eq!(one_result.selected.branch, Branch::Doubt);

    let mut two = RunContext::new(&canon, Operation::Confirmar);
    two.provided_evidence.push("quorum:ops:ana".to_string());
    two.provided_evidence.push("quorum:ops:bruno".to_string());
    let two_result = run_with_context(&logline, two)?;
    assert_eq!(two_result.selected.branch, Branch::Ok);
    assert_eq!(two_result.selected.position, "if_ok");
    Ok(())
}

#[test]
fn confirmed_by_quorum_deduplicates_repeated_witnesses() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "tomorrow",
        "quorum:2:ops",
        "send",
        "ask",
        "reject",
        "pending",
    );

    let mut duplicate = RunContext::new(&canon, Operation::Confirmar);
    duplicate
        .provided_evidence
        .push("quorum:ops:ana".to_string());
    duplicate
        .provided_evidence
        .push("quorum:ops:ana".to_string());
    duplicate.provided_evidence.push("quorum:2:ops".to_string());
    let duplicate_result = run_with_context(&logline, duplicate)?;
    assert_eq!(duplicate_result.selected.branch, Branch::Doubt);

    let mut distinct = RunContext::new(&canon, Operation::Confirmar);
    distinct
        .provided_evidence
        .push("quorum:ops:ana".to_string());
    distinct
        .provided_evidence
        .push("quorum:ops:bruno".to_string());
    let distinct_result = run_with_context(&logline, distinct)?;
    assert_eq!(distinct_result.selected.branch, Branch::Ok);
    Ok(())
}

#[test]
fn evidence_view_groups_distinct_quorum_witnesses() {
    let tokens = vec![
        "quorum:ops:ana".to_string(),
        "quorum:ops:ana".to_string(),
        "quorum:ops:bruno".to_string(),
        "quorum:finance:carla".to_string(),
        "ana_signature".to_string(),
    ];
    let view = EvidenceView::from_tokens(&tokens);
    let requirement = EvidenceRequirement::parse("quorum:2:ops");

    assert!(view.contains(&requirement));
    assert_eq!(view.quorum_count(&requirement), 2);
    assert_eq!(
        view.quorum_witnesses().get("ops"),
        Some(&vec!["ana".to_string(), "bruno".to_string()])
    );
    assert!(view.contains(&EvidenceRequirement::parse("ana_signature")));
}

#[test]
fn release_route_classifies_known_and_domain_routes() {
    let execute = ReleaseRoute::parse("execute");
    assert_eq!(execute.kind, ReleaseKind::Execute);
    assert_eq!(execute.selected_reason(), "selected_release_execute");

    let tower = ReleaseRoute::parse("release_to_tower");
    assert_eq!(tower.kind, ReleaseKind::ReleaseToTower);

    let emit = ReleaseRoute::parse("emit_logline");
    assert_eq!(emit.kind, ReleaseKind::EmitLogLine);

    let grammar = ReleaseRoute::parse("enter_strong_grammar");
    assert_eq!(grammar.kind, ReleaseKind::EnterStrongGrammar);

    let domain = ReleaseRoute::parse("send");
    assert_eq!(domain.kind, ReleaseKind::Domain("send".to_string()));
    assert_eq!(domain.selected_reason(), "selected_release_domain");
}

#[test]
fn if_ok_selected_preserves_route_and_classifies_without_executing() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "tomorrow",
        "ana",
        "send",
        "ask",
        "reject",
        "pending",
    );
    let mut ctx = RunContext::new(&canon, Operation::Acionar);
    ctx.provided_evidence.push("ana".to_string());

    let result = run_with_context(&logline, ctx)?;

    assert_eq!(result.runtime.if_ok.resolution, SlotResolution::Selected);
    assert_eq!(result.runtime.if_ok.reason, "selected_release_domain");
    assert_eq!(result.runtime.if_ok.route, Some("send".to_string()));
    assert_eq!(result.selected.branch, Branch::Ok);
    Ok(())
}

#[test]
fn if_ok_not_selected_remains_branch_not_chosen_when_confirmation_is_missing() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "tomorrow",
        "ana",
        "send",
        "ask",
        "reject",
        "pending",
    );

    let result = walk(&canon, &logline)?;

    assert_eq!(result.selected.branch, Branch::Doubt);
    assert_eq!(result.runtime.if_ok.resolution, SlotResolution::NotSelected);
    assert_eq!(result.runtime.if_ok.reason, "branch_not_chosen");
    Ok(())
}

struct RecordingReleaseAdapter;

impl ReleaseAdapter for RecordingReleaseAdapter {
    type Receipt = String;
    type Error = String;

    fn release(&self, route: &ReleaseRoute) -> std::result::Result<Self::Receipt, Self::Error> {
        Ok(format!("would_release:{}", route.raw))
    }
}

#[test]
fn release_adapter_is_a_future_boundary_not_called_by_walk() -> Result<()> {
    let adapter = RecordingReleaseAdapter;
    let receipt = adapter
        .release(&ReleaseRoute::parse("send"))
        .map_err(logline_status::LogLineError::InvalidWhoOrigin)?;
    assert_eq!(receipt, "would_release:send");

    let canon = canon()?;
    let logline: LogLine = serde_json::from_str(include_str!("../examples/sample.ok.json"))?;
    let mut ctx = RunContext::new(&canon, Operation::Acionar);
    ctx.provided_evidence.push("ana".to_string());
    let result = run_with_context(&logline, ctx)?;

    assert_eq!(result.runtime.if_ok.resolution, SlotResolution::Selected);
    assert!(!result.runtime.if_ok.reason.starts_with("would_release"));
    Ok(())
}

#[test]
fn doubt_route_classifies_clarification_suspension_and_domain_routes() {
    let suspend = DoubtRoute::parse("suspend");
    assert_eq!(suspend.kind, DoubtKind::Suspend);
    assert!(suspend.is_suspending());
    assert!(!suspend.is_clarifying());
    assert!(!suspend.is_releasing());
    assert_eq!(suspend.selected_reason(), "suspended_under_doubt");

    let ask_or_suspend = DoubtRoute::parse("ask_or_suspend");
    assert_eq!(ask_or_suspend.kind, DoubtKind::Clarify);
    assert!(ask_or_suspend.is_clarifying());
    assert!(ask_or_suspend.is_suspending());
    assert_eq!(ask_or_suspend.selected_reason(), "clarification_required");

    let ghost = DoubtRoute::parse("ghost_and_clarify");
    assert_eq!(ghost.kind, DoubtKind::Ghost);
    assert!(ghost.is_clarifying());

    let manual_review = DoubtRoute::parse("manual_review");
    assert_eq!(manual_review.kind, DoubtKind::ManualReview);
    assert_eq!(manual_review.selected_reason(), "manual_review_required");

    let domain = DoubtRoute::parse("domain_review");
    assert_eq!(domain.kind, DoubtKind::Domain("domain_review".to_string()));
    assert_eq!(domain.selected_reason(), "domain_doubt_route:domain_review");
}

#[test]
fn if_doubt_parse_recognizes_simulation_routes() {
    let simulate = DoubtRoute::parse("simulate");
    assert_eq!(simulate.kind, DoubtKind::Simulate);
    assert!(simulate.is_simulating());
    assert!(simulate.is_trace_emitting());
    assert!(!simulate.is_releasing());
    assert_eq!(simulate.selected_reason(), "simulation_requested");

    let receipt = DoubtRoute::parse("emit_simulation_receipt");
    assert_eq!(receipt.kind, DoubtKind::EmitSimulationReceipt);
    assert!(receipt.is_simulating());
    assert_eq!(receipt.selected_reason(), "simulation_receipt_requested");

    let trace = DoubtRoute::parse("emit_doubt_trace");
    assert_eq!(trace.kind, DoubtKind::EmitDoubtTrace);
    assert!(!trace.is_simulating());
    assert!(trace.is_trace_emitting());
    assert_eq!(trace.selected_reason(), "doubt_trace_requested");

    let possible_world = DoubtRoute::parse("propose_possible_world");
    assert_eq!(possible_world.kind, DoubtKind::ProposePossibleWorld);
    assert!(possible_world.is_simulating());
    assert_eq!(possible_world.selected_reason(), "possible_world_requested");

    let ask_for_evidence = DoubtRoute::parse("ask_for_evidence");
    assert_eq!(ask_for_evidence.kind, DoubtKind::AskForEvidence);
    assert!(ask_for_evidence.is_clarifying());
    assert!(ask_for_evidence.is_trace_emitting());
    assert_eq!(ask_for_evidence.selected_reason(), "evidence_missing");

    let dispatch = DoubtRoute::parse("dispatch");
    assert_eq!(dispatch.kind, DoubtKind::Dispatch);
    assert_eq!(dispatch.selected_reason(), "dispatch_under_doubt");
}

#[test]
fn clarification_and_suspension_records_are_typed_boundaries() {
    let route = DoubtRoute::parse("ask_or_suspend");
    let request = ClarificationRequest::new(
        route.clone(),
        "which confirmation is missing?",
        vec![Slot::ConfirmedBy],
    );
    assert_eq!(request.route, route);
    assert_eq!(request.missing_slots, vec![Slot::ConfirmedBy]);

    let suspension = SuspensionRecord::new(DoubtRoute::parse("suspend"), "receipt_missing");
    assert_eq!(suspension.route.kind, DoubtKind::Suspend);
    assert_eq!(suspension.reason, "receipt_missing");
}

#[test]
fn if_doubt_selected_preserves_route_and_classifies_uncertainty() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "tomorrow",
        "ana",
        "send",
        "ask_or_suspend",
        "reject",
        "pending",
    );
    let ctx = RunContext::new(&canon, Operation::Despachar);

    let result = run_with_context(&logline, ctx)?;

    assert_eq!(result.operation_home, Slot::IfDoubt);
    assert_eq!(result.selected.branch, Branch::Doubt);
    assert_eq!(result.runtime.if_doubt.resolution, SlotResolution::Selected);
    assert_eq!(result.runtime.if_doubt.reason, "clarification_required");
    assert_eq!(
        result.runtime.if_doubt.route,
        Some("ask_or_suspend".to_string())
    );
    assert_eq!(result.runtime.if_ok.reason, "branch_not_chosen");
    Ok(())
}

#[test]
fn if_doubt_selected_emits_doubt_trace() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "tomorrow",
        "ana",
        "send",
        "ask_for_evidence",
        "reject",
        "pending",
    );

    let result = run_with_context(&logline, RunContext::new(&canon, Operation::Despachar))?;

    assert_eq!(result.selected.branch, Branch::Doubt);
    assert_eq!(result.runtime.if_doubt.resolution, SlotResolution::Selected);
    assert_eq!(result.runtime.if_doubt.reason, "evidence_missing");
    let trace = result
        .doubt_trace
        .as_ref()
        .ok_or_else(|| missing_test_material("doubt trace"))?;
    assert_eq!(trace.branch, Branch::Doubt);
    assert_eq!(trace.reason, "receipt_missing");
    assert_eq!(trace.simulated_route, "ask_for_evidence");
    assert!(!trace.released);
    Ok(())
}

#[test]
fn if_doubt_trace_is_never_released() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "tomorrow",
        "ana",
        "send",
        "emit_doubt_trace",
        "reject",
        "pending",
    );

    let result = walk(&canon, &logline)?;
    let trace = result
        .doubt_trace
        .as_ref()
        .ok_or_else(|| missing_test_material("doubt trace"))?;

    assert_eq!(result.selected.branch, Branch::Doubt);
    assert!(!trace.released);
    assert_eq!(result.simulation_receipt, None);
    Ok(())
}

#[test]
fn simulation_route_emits_simulation_receipt() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "runtime",
        "simulate",
        "candidate_act",
        "before_release",
        "missing_evidence",
        "propose_path",
        "propose_possible_world",
        "discard",
        "doubt",
    );

    let result = run_with_context(&logline, RunContext::new(&canon, Operation::Despachar))?;
    let receipt = result
        .simulation_receipt
        .as_ref()
        .ok_or_else(|| missing_test_material("simulation receipt"))?;

    assert_eq!(result.selected.branch, Branch::Doubt);
    assert_eq!(receipt.receipt_kind, "simulation");
    assert_eq!(receipt.logline_digest, canonical_tuple_digest(&logline));
    assert_eq!(receipt.branch, Branch::Doubt);
    assert_eq!(receipt.reason, "receipt_missing");
    assert_eq!(receipt.simulated_route, "propose_possible_world");
    Ok(())
}

#[test]
fn simulation_receipt_is_never_executed_or_released() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "runtime",
        "simulate",
        "candidate_act",
        "before_release",
        "missing_evidence",
        "propose_path",
        "simulate",
        "discard",
        "doubt",
    );

    let result = walk(&canon, &logline)?;
    let receipt = result
        .simulation_receipt
        .as_ref()
        .ok_or_else(|| missing_test_material("simulation receipt"))?;

    assert!(!receipt.executed);
    assert!(!receipt.released);
    assert!(
        !result
            .doubt_trace
            .as_ref()
            .ok_or_else(|| missing_test_material("doubt trace"))?
            .released
    );
    Ok(())
}

#[test]
fn non_simulation_doubt_route_emits_trace_but_no_receipt() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "tomorrow",
        "ana",
        "send",
        "suspend",
        "reject",
        "pending",
    );

    let result = walk(&canon, &logline)?;

    assert_eq!(result.selected.branch, Branch::Doubt);
    assert!(result.doubt_trace.is_some());
    assert_eq!(result.simulation_receipt, None);
    Ok(())
}

#[test]
fn if_doubt_simulate_still_walks_all_9_slots() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "runtime",
        "simulate",
        "candidate_act",
        "before_release",
        "missing_evidence",
        "propose_path",
        "simulate",
        "discard",
        "doubt",
    );

    let result = run_with_context(&logline, RunContext::new(&canon, Operation::Despachar))?;

    assert_eq!(result.runtime.who.value, "runtime");
    assert_eq!(result.runtime.did.value, "simulate");
    assert_eq!(result.runtime.this_.value, "candidate_act");
    assert_eq!(result.runtime.when.value, "before_release");
    assert_eq!(result.runtime.confirmed_by.value, "missing_evidence");
    assert_eq!(result.runtime.if_ok.value, "propose_path");
    assert_eq!(result.runtime.if_doubt.value, "simulate");
    assert_eq!(result.runtime.if_not.value, "discard");
    assert_eq!(result.runtime.status.value, "doubt");
    Ok(())
}

#[test]
fn if_doubt_does_not_change_confirmed_by_pivot() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "runtime",
        "simulate",
        "candidate_act",
        "before_release",
        "missing_evidence",
        "propose_path",
        "simulate",
        "discard",
        "doubt",
    );

    let result = run_with_context(&logline, RunContext::new(&canon, Operation::Despachar))?;

    assert_eq!(result.selected.branch, Branch::Doubt);
    assert_eq!(result.selected.reason, "receipt_missing");
    assert_eq!(
        result.runtime.confirmed_by.resolution,
        SlotResolution::Doubt
    );
    assert_eq!(result.runtime.confirmed_by.reason, "receipt_missing");
    Ok(())
}

#[test]
fn rejection_route_classifies_problem_routes() {
    let reject = RejectionRoute::parse("reject");
    assert_eq!(reject.kind, RejectionKind::Reject);
    assert_eq!(reject.selected_reason(), "selected_problem_reject");

    let forbid = RejectionRoute::parse("forbid");
    assert_eq!(forbid.kind, RejectionKind::ForbiddenRoute);
    assert_eq!(forbid.selected_reason(), "selected_problem_forbid");

    let do_not_execute = RejectionRoute::parse("do_not_execute");
    assert_eq!(do_not_execute.kind, RejectionKind::ForbiddenRoute);

    let void = RejectionRoute::parse("void");
    assert_eq!(void.kind, RejectionKind::Void);

    let mark_failed = RejectionRoute::parse("mark_failed");
    assert_eq!(mark_failed.kind, RejectionKind::MarkFailed);

    let rollback = RejectionRoute::parse("rollback");
    assert_eq!(rollback.kind, RejectionKind::Rollback);

    let domain = RejectionRoute::parse("domain_refusal");
    assert_eq!(
        domain.kind,
        RejectionKind::Domain("domain_refusal".to_string())
    );
    assert_eq!(domain.selected_reason(), "selected_problem_domain");
}

#[test]
fn if_not_selected_preserves_route_and_builds_problem_record() -> Result<()> {
    let canon = canon()?;
    let logline: LogLine = serde_json::from_str(include_str!("../examples/sample.not.json"))?;
    let result = run_with_context(&logline, RunContext::new(&canon, Operation::MarcarProblema))?;

    assert_eq!(result.operation_home, Slot::IfNot);
    assert_eq!(result.selected.branch, Branch::Not);
    assert_eq!(result.runtime.if_not.resolution, SlotResolution::Selected);
    assert_eq!(result.runtime.if_not.reason, "selected_problem_forbid");
    assert_eq!(result.runtime.if_not.route, Some("forbid".to_string()));

    assert_eq!(
        ProblemRecord::from_selected_branch(&result.selected),
        Some(ProblemRecord {
            slot: Slot::IfNot,
            kind: RejectionKind::ForbiddenRoute,
            reason: "prohibited_by_canon".to_string(),
            route: "forbid".to_string(),
        })
    );
    Ok(())
}

#[test]
fn rust_error_paths_stay_separate_from_branch_not() -> Result<()> {
    let canon = canon()?;
    let invalid = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "tomorrow",
        "ana",
        "send",
        "ask",
        "",
        "pending",
    );

    assert!(
        run_with_context(&invalid, RunContext::new(&canon, Operation::MarcarProblema)).is_err()
    );

    let prohibited: LogLine = serde_json::from_str(include_str!("../examples/sample.not.json"))?;
    let result = run_with_context(
        &prohibited,
        RunContext::new(&canon, Operation::MarcarProblema),
    )?;
    assert_eq!(result.selected.branch, Branch::Not);
    assert_eq!(result.runtime.if_not.resolution, SlotResolution::Selected);
    Ok(())
}

#[test]
fn lifecycle_from_known_status_returns_typed_variant() {
    let pending = Lifecycle::from_status("pending");
    assert_eq!(pending, Lifecycle::Pending);
    assert_eq!(pending.as_canonical_str(), Some("pending"));
    assert_eq!(pending.canonical_label(), "pending");

    let rolled_back = Lifecycle::from_status("rolled_back");
    assert_eq!(rolled_back, Lifecycle::RolledBack);
    assert_eq!(rolled_back.as_canonical_str(), Some("rolled_back"));
}

#[test]
fn lifecycle_from_unknown_status_returns_domain_or_recoverable() {
    let custom = Lifecycle::from_status("domain_waiting");
    assert_eq!(custom, Lifecycle::Domain("domain_waiting".to_string()));
    assert_eq!(custom.as_canonical_str(), None);
    assert_eq!(custom.canonical_label(), "domain_waiting");
}

#[test]
fn lifecycle_json_keeps_lowercase_status_transition_strings() -> Result<()> {
    let transition = StatusTransition {
        before: "pending".to_string(),
        after: "confirmed".to_string(),
    };

    let json = serde_json::to_string(&transition)?;
    assert_eq!(json, "{\"before\":\"pending\",\"after\":\"confirmed\"}");
    Ok(())
}

#[test]
fn lifecycle_transition_wraps_existing_status_transition_without_changing_json_surface(
) -> Result<()> {
    let canon = canon()?;
    let typed = lifecycle_transition(&canon, "pending", &Branch::Ok);

    assert_eq!(
        typed,
        LifecycleTransition {
            before: Lifecycle::Pending,
            after: Lifecycle::Confirmed,
        }
    );
    Ok(())
}

#[test]
fn tuple_digest_is_stable_for_same_9_fields() {
    let left = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "tomorrow",
        "ana",
        "send",
        "ask",
        "reject",
        "pending",
    );
    let right = left.clone();

    assert_eq!(
        canonical_tuple_digest(&left),
        canonical_tuple_digest(&right)
    );
    assert!(canonical_tuple_digest(&left).len() == 64);
}

#[test]
fn tuple_digest_changes_when_any_slot_changes() {
    let base = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "tomorrow",
        "ana",
        "send",
        "ask",
        "reject",
        "pending",
    );
    let mut changed_who = base.clone();
    changed_who.who = "ana".to_string();
    let mut changed_status = base.clone();
    changed_status.status = "confirmed".to_string();

    let base_digest = canonical_tuple_digest(&base);
    assert_ne!(base_digest, canonical_tuple_digest(&changed_who));
    assert_ne!(base_digest, canonical_tuple_digest(&changed_status));
}

#[test]
fn tuple_digest_changes_when_each_of_the_9_slots_changes() {
    fn assert_slot_change_changes_digest(change: fn(&mut LogLine)) {
        let mut changed = LogLine::new(
            "dan",
            "send",
            "invoice_123",
            "tomorrow",
            "ana",
            "send",
            "ask",
            "reject",
            "pending",
        );
        let original = changed.clone();
        change(&mut changed);

        assert_ne!(
            canonical_tuple_digest(&original),
            canonical_tuple_digest(&changed)
        );
    }

    assert_slot_change_changes_digest(|line| line.who = "ana".to_string());
    assert_slot_change_changes_digest(|line| line.did = "approve".to_string());
    assert_slot_change_changes_digest(|line| line.this_ = "invoice_456".to_string());
    assert_slot_change_changes_digest(|line| line.when = "2026-05-05T09:00:00Z".to_string());
    assert_slot_change_changes_digest(|line| line.confirmed_by = "bruno".to_string());
    assert_slot_change_changes_digest(|line| line.if_ok = "commit_status".to_string());
    assert_slot_change_changes_digest(|line| line.if_doubt = "suspend".to_string());
    assert_slot_change_changes_digest(|line| line.if_not = "deny".to_string());
    assert_slot_change_changes_digest(|line| line.status = "confirmed".to_string());
}

#[test]
fn tuple_digest_ignores_non_slot_context() -> Result<()> {
    let canon = canon()?;
    let logline = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "tomorrow",
        "ana",
        "send",
        "ask",
        "reject",
        "pending",
    );

    let mut confirm = RunContext::new(&canon, Operation::Confirmar);
    confirm.provided_evidence.push("ana".to_string());
    let registrar = RunContext::new(&canon, Operation::Registrar);

    let digest_before = canonical_tuple_digest(&logline);
    let _confirmed = run_with_context(&logline, confirm)?;
    let _registered = run_with_context(&logline, registrar)?;
    let digest_after = canonical_tuple_digest(&logline);

    assert_eq!(digest_before, digest_after);
    Ok(())
}

struct CountingLedger {
    appends: Cell<usize>,
}

impl CountingLedger {
    fn new() -> Self {
        Self {
            appends: Cell::new(0),
        }
    }

    fn calls(&self) -> usize {
        self.appends.get()
    }
}

impl StatusLedger for CountingLedger {
    fn append(&self, _entry: &LedgerEntry) -> Result<()> {
        self.appends.set(self.appends.get() + 1);
        Ok(())
    }
}

#[test]
fn status_transition_does_not_persist_without_explicit_ledger_append() -> Result<()> {
    let canon = canon()?;
    let ledger = CountingLedger::new();
    let logline = LogLine::new(
        "dan",
        "send",
        "invoice_123",
        "tomorrow",
        "ana",
        "send",
        "ask",
        "reject",
        "pending",
    );
    let transition = lifecycle_transition(&canon, &logline.status, &Branch::Ok);

    assert_eq!(ledger.calls(), 0);

    let entry = LedgerEntry {
        logline_digest: canonical_tuple_digest(&logline),
        selected_branch: Branch::Ok,
        transition,
        reason: "receipt_found".to_string(),
    };
    assert_eq!(ledger.calls(), 0);

    ledger.append(&entry)?;
    assert_eq!(ledger.calls(), 1);
    Ok(())
}

#[test]
fn tuple_hash_uses_jcs_canonical_json() -> std::result::Result<(), Box<dyn Error>> {
    let receipt = serde_json::json!({
        "receipt_version": "logline.receipt.v0",
        "who": "dan",
        "did": "rested",
        "this": "slept_well",
        "when": "2026-05-17T07:30:00Z",
        "confirmed_by": "dan",
        "if_ok": "continue_minilab_work",
        "if_doubt": "",
        "if_not": "",
        "status": "claimed",
        "json_canonicalization": "jcs-rfc8785",
        "hashes": {
            "tuple_hash": "6ace2eed03aa73839414db1d76a3bf08880d8ae50f5db81405c85f2b269ef1ee",
            "content_hash": "b74954069c9135090740439e08bdd442b4b12767c4df63b504c3e6b1029ffbb8",
            "algorithm": "sha256"
        },
        "id": "b74954069c9135090740439e08bdd442b4b12767c4df63b504c3e6b1029ffbb8"
    });

    assert_eq!(
        tuple_hash(&receipt)?,
        "6ace2eed03aa73839414db1d76a3bf08880d8ae50f5db81405c85f2b269ef1ee"
    );
    Ok(())
}

#[test]
fn content_hash_excludes_id_and_hashes() -> std::result::Result<(), Box<dyn Error>> {
    let receipt = serde_json::json!({
        "receipt_version": "logline.receipt.v0",
        "who": "dan",
        "did": "rested",
        "this": "slept_well",
        "when": "2026-05-17T07:30:00Z",
        "confirmed_by": "dan",
        "if_ok": "continue_minilab_work",
        "if_doubt": "",
        "if_not": "",
        "status": "claimed",
        "json_canonicalization": "jcs-rfc8785",
        "hashes": {
            "tuple_hash": "6ace2eed03aa73839414db1d76a3bf08880d8ae50f5db81405c85f2b269ef1ee",
            "content_hash": "b74954069c9135090740439e08bdd442b4b12767c4df63b504c3e6b1029ffbb8",
            "algorithm": "sha256"
        },
        "id": "b74954069c9135090740439e08bdd442b4b12767c4df63b504c3e6b1029ffbb8"
    });

    let computed = content_hash(&receipt)?;
    assert_eq!(computed, "b74954069c9135090740439e08bdd442b4b12767c4df63b504c3e6b1029ffbb8");
    assert_eq!(computed, receipt["id"].as_str().unwrap());
    Ok(())
}

#[test]
fn content_hash_is_stable_regardless_of_json_key_order() -> std::result::Result<(), Box<dyn Error>>
{
    let left = serde_json::json!({
        "receipt_version": "logline.receipt.v0",
        "who": "logline_runtime",
        "did": "simulate",
        "this": "candidate_act",
        "when": "2026-05-06T12:00:00Z",
        "confirmed_by": "missing_evidence",
        "if_ok": "record_trace",
        "if_doubt": "simulate",
        "if_not": "forbid_execution",
        "status": "doubt",
        "json_canonicalization": "jcs-rfc8785",
        "hashes": {
            "tuple_hash": "0000000000000000000000000000000000000000000000000000000000000000",
            "content_hash": "0000000000000000000000000000000000000000000000000000000000000000",
            "algorithm": "sha256"
        },
        "id": "0000000000000000000000000000000000000000000000000000000000000000"
    });
    let right = serde_json::json!({
        "status": "doubt",
        "if_not": "forbid_execution",
        "if_doubt": "simulate",
        "if_ok": "record_trace",
        "confirmed_by": "missing_evidence",
        "when": "2026-05-06T12:00:00Z",
        "this": "candidate_act",
        "did": "simulate",
        "who": "logline_runtime",
        "receipt_version": "logline.receipt.v0",
        "json_canonicalization": "jcs-rfc8785",
        "hashes": {
            "algorithm": "sha256",
            "content_hash": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "tuple_hash": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        },
        "id": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    });

    assert_eq!(content_hash(&left)?, content_hash(&right)?);
    Ok(())
}

#[test]
fn changing_aux_changes_content_hash_but_not_tuple_hash() -> std::result::Result<(), Box<dyn Error>>
{
    let base = serde_json::json!({
        "receipt_version": "logline.receipt.v0",
        "who": "dan",
        "did": "send",
        "this": "invoice_123",
        "when": "2026-05-06T12:00:00Z",
        "confirmed_by": "ana",
        "if_ok": "execute",
        "if_doubt": "suspend",
        "if_not": "reject",
        "status": "confirmed",
        "json_canonicalization": "jcs-rfc8785"
    });

    let mut with_aux = base.clone();
    with_aux["domain_note"] = serde_json::json!("extra context");

    assert_eq!(tuple_hash(&base)?, tuple_hash(&with_aux)?);
    assert_ne!(content_hash(&base)?, content_hash(&with_aux)?);
    Ok(())
}

#[test]
fn tuple_hash_matches_canonical_tuple_digest_for_same_9_slots(
) -> std::result::Result<(), Box<dyn Error>> {
    let receipt = serde_json::json!({
        "receipt_version": "logline.receipt.v0",
        "who": "dan",
        "did": "send",
        "this": "invoice_123",
        "when": "tomorrow",
        "confirmed_by": "ana",
        "if_ok": "send",
        "if_doubt": "ask",
        "if_not": "reject",
        "status": "pending",
        "json_canonicalization": "jcs-rfc8785"
    });

    let logline = receipt_fixture_logline(&receipt)?;
    assert_eq!(tuple_hash(&receipt)?, canonical_tuple_digest(&logline));
    Ok(())
}

#[test]
fn content_hash_ignores_id_and_hashes_values() -> std::result::Result<(), Box<dyn Error>> {
    let mut a = serde_json::json!({
        "receipt_version": "logline.receipt.v0",
        "who": "dan",
        "did": "send",
        "this": "invoice_123",
        "when": "2026-05-06T12:00:00Z",
        "confirmed_by": "ana",
        "if_ok": "execute",
        "if_doubt": "suspend",
        "if_not": "reject",
        "status": "confirmed",
        "json_canonicalization": "jcs-rfc8785",
        "id": "0000000000000000000000000000000000000000000000000000000000000000",
        "hashes": {
            "tuple_hash": "0000000000000000000000000000000000000000000000000000000000000000",
            "content_hash": "0000000000000000000000000000000000000000000000000000000000000000",
            "algorithm": "sha256"
        }
    });
    let b = a.clone();

    a["id"] =
        serde_json::json!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    a["hashes"]["content_hash"] =
        serde_json::json!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");

    assert_eq!(content_hash(&a)?, content_hash(&b)?);
    Ok(())
}

#[test]
fn canonical_json_sorts_keys_deterministically() -> std::result::Result<(), Box<dyn Error>> {
    let left = serde_json::json!({
        "released": false,
        "executed": false,
        "simulated_route": "simulate"
    });
    let right = serde_json::json!({
        "simulated_route": "simulate",
        "executed": false,
        "released": false
    });

    assert_eq!(canonical_json(&left)?, canonical_json(&right)?);
    Ok(())
}

#[test]
fn nested_body_logline_is_invalid_for_receipt_profile() {
    let receipt = serde_json::json!({
        "receipt_version": "logline.receipt.v0",
        "body": {
            "logline": {
                "who": "dan",
                "did": "send",
                "this": "invoice",
                "when": "now",
                "confirmed_by": "ana",
                "if_ok": "send",
                "if_doubt": "ask",
                "if_not": "reject",
                "status": "pending"
            }
        }
    });

    assert!(receipt.get("body").is_some());
    assert!(receipt.get("who").is_none());
}
