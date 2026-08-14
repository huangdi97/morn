//! Governed effects: E0 lifecycle-reversible, E1 transactional, E2 compensatable, E3 irreversible.

use serde::{Deserialize, Serialize};

/// Classification of an effect's reversibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum EffectClass {
    /// Lifecycle-reversible: register/unregister, mount/dispose, temporary context.
    E0LifecycleReversible,
    /// Transactional: uncommitted DB/transaction state, can roll back.
    E1Transactional,
    /// Compensatable: needs an explicit compensation action (create order -> cancel order).
    E2Compensatable,
    /// Irreversible: external email, payment, physical experiment, robot motion, public release.
    E3Irreversible,
}

impl EffectClass {
    pub fn code(&self) -> &'static str {
        match self {
            EffectClass::E0LifecycleReversible => "E0",
            EffectClass::E1Transactional => "E1",
            EffectClass::E2Compensatable => "E2",
            EffectClass::E3Irreversible => "E3",
        }
    }
}

impl std::fmt::Display for EffectClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}

/// Effect contract declared by an action or capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectContract {
    pub class: EffectClass,
    /// Required for E2: id of a compensating action that can undo the effect.
    pub compensation_action: Option<String>,
    pub irreversible_reason: Option<String>,
    pub preview_required: bool,
    pub approval_required: bool,
    pub verification_required: bool,
}

impl EffectContract {
    pub fn e0() -> Self {
        Self {
            class: EffectClass::E0LifecycleReversible,
            compensation_action: None,
            irreversible_reason: None,
            preview_required: false,
            approval_required: false,
            verification_required: false,
        }
    }

    pub fn e1() -> Self {
        Self {
            class: EffectClass::E1Transactional,
            compensation_action: None,
            irreversible_reason: None,
            preview_required: false,
            approval_required: false,
            verification_required: true,
        }
    }

    pub fn e2(compensation_action: impl Into<String>) -> Self {
        Self {
            class: EffectClass::E2Compensatable,
            compensation_action: Some(compensation_action.into()),
            irreversible_reason: None,
            preview_required: true,
            approval_required: false,
            verification_required: true,
        }
    }

    pub fn e3(reason: impl Into<String>) -> Self {
        Self {
            class: EffectClass::E3Irreversible,
            compensation_action: None,
            irreversible_reason: Some(reason.into()),
            preview_required: true,
            approval_required: true,
            verification_required: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e2_requires_compensation() {
        let e2 = EffectContract::e2("cancel_order");
        assert_eq!(e2.class, EffectClass::E2Compensatable);
        assert_eq!(e2.compensation_action.as_deref(), Some("cancel_order"));
    }

    #[test]
    fn e3_defaults_to_preview_approval_verify() {
        let e3 = EffectContract::e3("physical experiment");
        assert!(e3.preview_required);
        assert!(e3.approval_required);
        assert!(e3.verification_required);
    }
}
