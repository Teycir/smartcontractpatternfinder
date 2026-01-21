use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

/// The semantic context for an entire contract
#[derive(Debug, Default, Clone)]
pub struct ContractContext {
    pub name: String,
    pub functions: FxHashMap<String, FunctionContext>,
    pub modifiers: FxHashMap<String, ModifierContext>,
    pub state_variables: FxHashMap<String, StateVariable>,
}

/// Context for a single function
#[derive(Debug, Clone)]
pub struct FunctionContext {
    pub name: String,
    pub visibility: Visibility,
    pub mutability: Mutability,
    pub modifiers: Vec<String>,
    pub external_calls: Vec<ExternalCall>,
    pub state_changes: Vec<StateChange>,
    pub protections: ProtectionSet,
    pub start_line: usize,
    pub end_line: usize,
}

/// Classification of modifier types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierType {
    ReentrancyGuard,
    AccessControl,
    Pausable,
    InputValidation,
    Custom,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ModifierContext {
    pub name: String,
    pub modifier_type: ModifierType,
    pub confidence: f32,
    pub has_state_check: bool,
    pub can_revert: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Visibility {
    Public,
    External,
    Internal,
    Private,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Mutability {
    Pure,
    View,
    Payable,
    NonPayable,
}

#[derive(Debug, Clone)]
pub struct StateVariable {
    pub name: String,
    pub var_type: String,
}

/// Represents an external call
#[derive(Debug, Clone)]
pub struct ExternalCall {
    pub kind: ExternalCallKind,
    pub value_sent: bool,
    pub return_checked: ReturnCheckStatus,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalCallKind {
    LowLevelCall,
    LowLevelDelegateCall,
    LowLevelStaticCall,
    Transfer,
    Send,
    InterfaceCall,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnCheckStatus {
    Checked,
    Unchecked,
    SafeWrapper,
    NotApplicable,
}

/// Represents a state change
#[derive(Debug, Clone)]
pub struct StateChange {
    pub variable: String,
    pub change_type: StateChangeType,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateChangeType {
    Assignment,
    Increment,
    Decrement,
    Delete,
    MapUpdate,
    ArrayPush,
    ArrayPop,
}

/// Protection mechanisms detected
#[derive(Debug, Default, Clone)]
pub struct ProtectionSet {
    pub has_reentrancy_guard: bool,
    pub has_access_control: bool,
    pub has_pausable: bool,
    pub uses_checks_effects_interactions: bool,
}

impl Default for FunctionContext {
    fn default() -> Self {
        Self {
            name: String::new(),
            visibility: Visibility::Public,
            mutability: Mutability::NonPayable,
            modifiers: Vec::new(),
            external_calls: Vec::new(),
            state_changes: Vec::new(),
            protections: ProtectionSet::default(),
            start_line: 0,
            end_line: 0,
        }
    }
}
