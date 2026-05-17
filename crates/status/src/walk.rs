use crate::status;
use logline_if_doubt::{DoubtTrace, SimulationReceipt};
use logline_who::{
    is_prohibited, validate_against_canon, validate_shape, Branch, Canon, LogLine, LogLineError,
    Mode, Operation, Result, RunContext, RuntimeDecision, RuntimeSlot, SelectedBranch, Slot,
    SlotHouse, StatusTransition,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeSlots {
    pub who: RuntimeSlot,
    pub did: RuntimeSlot,

    #[serde(rename = "this")]
    pub this_: RuntimeSlot,

    pub when: RuntimeSlot,
    pub confirmed_by: RuntimeSlot,
    pub if_ok: RuntimeSlot,
    pub if_doubt: RuntimeSlot,
    pub if_not: RuntimeSlot,
    pub status: RuntimeSlot,
}

impl RuntimeSlots {
    fn from_ordered(results: Vec<(Slot, RuntimeSlot)>) -> Result<Self> {
        let mut who_slot = None;
        let mut did_slot = None;
        let mut this_slot = None;
        let mut when_slot = None;
        let mut confirmed_by_slot = None;
        let mut if_ok_slot = None;
        let mut if_doubt_slot = None;
        let mut if_not_slot = None;
        let mut status_slot = None;

        for (slot, result) in results {
            match slot {
                Slot::Who => who_slot = Some(result),
                Slot::Did => did_slot = Some(result),
                Slot::This => this_slot = Some(result),
                Slot::When => when_slot = Some(result),
                Slot::ConfirmedBy => confirmed_by_slot = Some(result),
                Slot::IfOk => if_ok_slot = Some(result),
                Slot::IfDoubt => if_doubt_slot = Some(result),
                Slot::IfNot => if_not_slot = Some(result),
                Slot::Status => status_slot = Some(result),
            }
        }

        Ok(Self {
            who: who_slot.ok_or(LogLineError::MissingRuntimeSlot("who"))?,
            did: did_slot.ok_or(LogLineError::MissingRuntimeSlot("did"))?,
            this_: this_slot.ok_or(LogLineError::MissingRuntimeSlot("this"))?,
            when: when_slot.ok_or(LogLineError::MissingRuntimeSlot("when"))?,
            confirmed_by: confirmed_by_slot
                .ok_or(LogLineError::MissingRuntimeSlot("confirmed_by"))?,
            if_ok: if_ok_slot.ok_or(LogLineError::MissingRuntimeSlot("if_ok"))?,
            if_doubt: if_doubt_slot.ok_or(LogLineError::MissingRuntimeSlot("if_doubt"))?,
            if_not: if_not_slot.ok_or(LogLineError::MissingRuntimeSlot("if_not"))?,
            status: status_slot.ok_or(LogLineError::MissingRuntimeSlot("status"))?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeLogLine {
    pub mode: Mode,
    pub operation: Operation,
    pub operation_home: Slot,
    pub phase: logline_who::Phase,
    pub logline: LogLine,
    pub runtime: RuntimeSlots,
    pub selected: SelectedBranch,
    pub status: StatusTransition,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub doubt_trace: Option<DoubtTrace>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub simulation_receipt: Option<SimulationReceipt>,
}

pub fn walk(canon: &Canon, logline: &LogLine) -> Result<RuntimeLogLine> {
    let ctx = RunContext::new(canon, Operation::Confirmar);
    run_with_context(logline, ctx)
}

pub fn run(canon: &Canon, logline: &LogLine, operation: Operation) -> Result<RuntimeLogLine> {
    let ctx = RunContext::new(canon, operation);
    run_with_context(logline, ctx)
}

pub fn run_with_context(input: &LogLine, ctx: RunContext<'_>) -> Result<RuntimeLogLine> {
    validate_shape(input)?;

    let (branch, reason) = if is_prohibited(ctx.canon, input) {
        (Branch::Not, "prohibited_by_canon")
    } else {
        validate_against_canon(ctx.canon, input)?;
        logline_confirmed_by::pivot(input, &ctx)
    };

    let selected = selected_branch(input, &branch, reason);
    let status = status::transition(ctx.canon, &input.status, &branch);
    let doubt_trace = DoubtTrace::from_selected_branch(&selected);
    let simulation_receipt =
        SimulationReceipt::from_selected_branch(&selected, status::canonical_tuple_digest(input));
    let decision = RuntimeDecision {
        selected: selected.clone(),
        status: status.clone(),
    };

    let houses: [&dyn SlotHouse; 9] = [
        &logline_who::WhoSlot,
        &logline_did::DidSlot,
        &logline_this::ThisSlot,
        &logline_when::WhenSlot,
        &logline_confirmed_by::ConfirmedBySlot,
        &logline_if_ok::IfOkSlot,
        &logline_if_doubt::IfDoubtSlot,
        &logline_if_not::IfNotSlot,
        &status::StatusSlot,
    ];

    let results = houses
        .iter()
        .map(|house| (house.slot(), house.resolve(input, &ctx, &decision)))
        .collect();

    Ok(RuntimeLogLine {
        mode: ctx.mode,
        operation: ctx.operation,
        operation_home: ctx.operation.home(),
        phase: ctx.operation.phase(),
        logline: input.clone(),
        runtime: RuntimeSlots::from_ordered(results)?,
        selected,
        status,
        doubt_trace,
        simulation_receipt,
    })
}

fn selected_branch(input: &LogLine, branch: &Branch, reason: &str) -> SelectedBranch {
    let position = branch.position();
    let route = match branch {
        Branch::Ok => input.if_ok.clone(),
        Branch::Doubt => input.if_doubt.clone(),
        Branch::Not => input.if_not.clone(),
    };

    SelectedBranch {
        branch: branch.clone(),
        position: position.to_string(),
        route,
        reason: reason.to_string(),
    }
}
