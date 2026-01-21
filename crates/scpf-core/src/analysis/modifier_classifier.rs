use scpf_types::*;

/// Known reentrancy guard modifier names
const REENTRANCY_GUARD_NAMES: &[&str] = &[
    "nonReentrant",
    "nonreentrant",
    "noReentrancy",
    "lock",
    "mutex",
    "reentrancyGuard",
];

/// Known access control modifier patterns
const ACCESS_CONTROL_NAMES: &[&str] = &[
    "onlyOwner",
    "onlyAdmin",
    "onlyRole",
    "onlyMinter",
    "onlyGovernance",
    "onlyOperator",
    "onlyAuthorized",
    "whenNotPaused",
    "auth",
];

pub fn classify_modifiers(ctx: &mut ContractContext) {
    for (name, modifier) in &mut ctx.modifiers {
        modifier.modifier_type = classify_modifier(name, modifier);
        modifier.confidence = compute_confidence(modifier);
    }
}

fn classify_modifier(name: &str, modifier: &ModifierContext) -> ModifierType {
    let name_lower = name.to_lowercase();

    // Check known reentrancy guard names
    if REENTRANCY_GUARD_NAMES
        .iter()
        .any(|&n| name_lower.contains(&n.to_lowercase()))
    {
        return ModifierType::ReentrancyGuard;
    }

    // Check known access control names
    if ACCESS_CONTROL_NAMES
        .iter()
        .any(|&n| name_lower.contains(&n.to_lowercase()))
    {
        return ModifierType::AccessControl;
    }

    // Check for pausable pattern
    if name_lower.contains("paused") || name_lower.contains("pause") {
        return ModifierType::Pausable;
    }

    // Analyze modifier implementation for patterns
    if modifier.has_state_check && modifier.can_revert {
        // Could be a custom guard
        return ModifierType::Custom;
    }

    ModifierType::Unknown
}

fn compute_confidence(modifier: &ModifierContext) -> f32 {
    match modifier.modifier_type {
        // Named patterns have high confidence
        ModifierType::ReentrancyGuard => {
            if REENTRANCY_GUARD_NAMES
                .iter()
                .any(|&n| modifier.name.to_lowercase().contains(&n.to_lowercase()))
            {
                0.95
            } else {
                0.75
            }
        }
        ModifierType::AccessControl => 0.85,
        ModifierType::Pausable => 0.80,
        ModifierType::Custom => 0.60,
        ModifierType::InputValidation => 0.70,
        ModifierType::Unknown => 0.0,
    }
}
