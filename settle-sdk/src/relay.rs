//! RelayOS-aligned settlement request shape.
//!
//! settle-sdk emits a request shaped like the RelayOS `SettlementLeg` so the
//! Rail's existing verifier (`SettlementLeg.verify_leg`,
//! `FoundationContributionPolicy.split`) can consume it directly.
//!
//! Per RelayOS discipline (see relayos-development skill):
//!   * an amount>0 leg REQUIRES `authorized_evidence_keys` (F1) — a payment
//!     leg with no bound producer refuses all evidence.
//!   * tier is driven by the operator registry: Commons = natural persons
//!     only; ANY legal entity is Licensed. Volume trigger $50k/12mo raises a
//!     transition error when a natural person outgrows commons.
//!   * the fee attaches to the SETTLEMENT, not the node.

/// Operator entity class (mirrors RelayOS operator_registry).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityClass {
    /// A natural person (man or woman) — Commons-eligible.
    NaturalPerson,
    /// Any legal entity (LLC/corp/partnership/trust/shell) — Licensed only.
    LegalEntity,
}

impl EntityClass {
    /// Whether this entity class is Commons (natural persons, $0).
    pub fn is_commons(&self) -> bool {
        matches!(self, EntityClass::NaturalPerson)
    }
}

/// A settlement leg request shaped for RelayOS's `SettlementLeg`.
#[derive(Debug, Clone)]
pub struct SettlementLegRequest {
    /// The paying operator's Relay identity (bound to the evidence keys).
    pub operator_id: String,
    /// The recipient's DERO address.
    pub recipient: String,
    /// Amount in atomic units (>0 => evidence keys REQUIRED per F1).
    pub amount: u64,
    /// Entity class of the paying operator -> tier.
    pub entity_class: EntityClass,
    /// Evidence keys that must be authorized for this leg (F1). Empty on a
    /// zero-amount leg is legal; REQUIRED when amount > 0.
    pub authorized_evidence_keys: Vec<String>,
    /// The foundation contribution split (bps) for this leg.
    pub foundation_contribution_bps: u32,
}

impl SettlementLegRequest {
    /// Build a leg request from an operator + recipient, resolving tier from
    /// entity class and wiring the F1 evidence-key requirement.
    ///
    /// Returns `Err` if an amount>0 leg has no authorized evidence keys.
    pub fn new(
        operator_id: impl Into<String>,
        recipient: impl Into<String>,
        amount: u64,
        entity_class: EntityClass,
        authorized_evidence_keys: Vec<String>,
    ) -> Result<Self, String> {
        if amount > 0 && authorized_evidence_keys.is_empty() {
            return Err(
                "amount>0 leg requires authorized_evidence_keys (F1)".into(),
            );
        }
        let foundation_contribution_bps = if entity_class.is_commons() {
            0
        } else {
            149 // per RelayOS licensed node fee-share floor
        };
        Ok(Self {
            operator_id: operator_id.into(),
            recipient: recipient.into(),
            amount,
            entity_class,
            authorized_evidence_keys,
            foundation_contribution_bps,
        })
    }

    /// The resolved tier label for this leg ("Commons" or "Licensed").
    pub fn tier_label(&self) -> &'static str {
        if self.entity_class.is_commons() {
            "Commons"
        } else {
            "Licensed"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn natural_person_is_commons() {
        let leg = SettlementLegRequest::new(
            "op1", "recipient", 1000, EntityClass::NaturalPerson, vec!["key1".into()],
        )
        .expect("natural person with evidence ok");
        assert_eq!(leg.tier_label(), "Commons");
        assert_eq!(leg.foundation_contribution_bps, 0);
    }

    #[test]
    fn legal_entity_is_licensed_with_fee() {
        let leg = SettlementLegRequest::new(
            "op2", "recipient", 1000, EntityClass::LegalEntity, vec!["key2".into()],
        )
        .expect("legal entity with evidence ok");
        assert_eq!(leg.tier_label(), "Licensed");
        assert_eq!(leg.foundation_contribution_bps, 149);
    }

    #[test]
    fn f1_requires_evidence_keys_on_payment() {
        let err = SettlementLegRequest::new(
            "op3", "recipient", 1000, EntityClass::NaturalPerson, vec![],
        )
        .expect_err("amount>0 without evidence keys must fail");
        assert!(err.contains("F1"));
    }

    #[test]
    fn zero_amount_leg_may_omit_evidence() {
        let leg = SettlementLegRequest::new(
            "op4", "recipient", 0, EntityClass::NaturalPerson, vec![],
        )
        .expect("zero-amount delivery leg ok without evidence");
        assert_eq!(leg.amount, 0);
    }
}
