use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogLine {
    pub who: String,
    pub did: String,

    #[serde(rename = "this")]
    pub this_: String,

    pub when: String,
    pub confirmed_by: String,
    pub if_ok: String,
    pub if_doubt: String,
    pub if_not: String,
    pub status: String,
}

impl LogLine {
    pub const POSITIONS: [&'static str; 9] = [
        "who",
        "did",
        "this",
        "when",
        "confirmed_by",
        "if_ok",
        "if_doubt",
        "if_not",
        "status",
    ];

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        who: impl Into<String>,
        did: impl Into<String>,
        this_: impl Into<String>,
        when: impl Into<String>,
        confirmed_by: impl Into<String>,
        if_ok: impl Into<String>,
        if_doubt: impl Into<String>,
        if_not: impl Into<String>,
        status: impl Into<String>,
    ) -> Self {
        Self {
            who: who.into(),
            did: did.into(),
            this_: this_.into(),
            when: normalize_when(when.into()),
            confirmed_by: confirmed_by.into(),
            if_ok: if_ok.into(),
            if_doubt: if_doubt.into(),
            if_not: if_not.into(),
            status: status.into(),
        }
    }

    pub fn get(&self, position: &str) -> Option<&str> {
        match position {
            "who" => Some(&self.who),
            "did" => Some(&self.did),
            "this" => Some(&self.this_),
            "when" => Some(&self.when),
            "confirmed_by" => Some(&self.confirmed_by),
            "if_ok" => Some(&self.if_ok),
            "if_doubt" => Some(&self.if_doubt),
            "if_not" => Some(&self.if_not),
            "status" => Some(&self.status),
            _ => None,
        }
    }
}

fn normalize_when(value: String) -> String {
    if value == "any_time" {
        "*".to_string()
    } else {
        value
    }
}
