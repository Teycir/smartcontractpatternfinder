use solang_parser::pt::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    Vulnerable,
    Protected(&'static str),
    NotApplicable,
    ParseError,
}

impl ValidationResult {
    pub fn is_vulnerable(&self) -> bool {
        matches!(self, ValidationResult::Vulnerable)
    }
}

pub struct AstValidator;

impl AstValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, ast: &SourceUnit, pattern_id: &str, target_line: usize) -> ValidationResult {
        match pattern_id {
            "unprotected-initialize" => self.validate_initialize(ast, target_line),
            "public-withdraw-no-auth" => self.validate_withdraw(ast, target_line),
            "external-mint-no-modifier" => self.validate_mint(ast, target_line),
            "external-burn-no-modifier" => self.validate_burn(ast, target_line),
            _ => ValidationResult::NotApplicable,
        }
    }

    fn validate_initialize(&self, ast: &SourceUnit, target_line: usize) -> ValidationResult {
        for part in &ast.0 {
            if let SourceUnitPart::ContractDefinition(contract) = part {
                for part in &contract.parts {
                    if let ContractPart::FunctionDefinition(func) = part {
                        if self.is_target_function(func, target_line) {
                            if let Some(name) = &func.name {
                                if name.name.contains("initialize") {
                                    return self.check_initialize_protection(func);
                                }
                            }
                        }
                    }
                }
            }
        }
        ValidationResult::NotApplicable
    }

    fn validate_withdraw(&self, ast: &SourceUnit, target_line: usize) -> ValidationResult {
        for part in &ast.0 {
            if let SourceUnitPart::ContractDefinition(contract) = part {
                for part in &contract.parts {
                    if let ContractPart::FunctionDefinition(func) = part {
                        if self.is_target_function(func, target_line) {
                            if let Some(name) = &func.name {
                                if name.name.contains("withdraw") {
                                    return self.check_access_control(func);
                                }
                            }
                        }
                    }
                }
            }
        }
        ValidationResult::NotApplicable
    }

    fn validate_mint(&self, ast: &SourceUnit, target_line: usize) -> ValidationResult {
        for part in &ast.0 {
            if let SourceUnitPart::ContractDefinition(contract) = part {
                for part in &contract.parts {
                    if let ContractPart::FunctionDefinition(func) = part {
                        if self.is_target_function(func, target_line) {
                            if let Some(name) = &func.name {
                                if name.name == "mint" {
                                    return self.check_access_control(func);
                                }
                            }
                        }
                    }
                }
            }
        }
        ValidationResult::NotApplicable
    }

    fn validate_burn(&self, ast: &SourceUnit, target_line: usize) -> ValidationResult {
        for part in &ast.0 {
            if let SourceUnitPart::ContractDefinition(contract) = part {
                for part in &contract.parts {
                    if let ContractPart::FunctionDefinition(func) = part {
                        if self.is_target_function(func, target_line) {
                            if let Some(name) = &func.name {
                                if name.name == "burn" {
                                    return self.check_access_control(func);
                                }
                            }
                        }
                    }
                }
            }
        }
        ValidationResult::NotApplicable
    }

    fn is_target_function(&self, func: &FunctionDefinition, target_line: usize) -> bool {
        func.loc.start() <= target_line && func.loc.end() >= target_line
    }

    fn check_initialize_protection(&self, func: &FunctionDefinition) -> ValidationResult {
        for attr in &func.attributes {
            if let FunctionAttribute::BaseOrModifier(_, base) = attr {
                let modifier_name = base.name.identifiers.iter()
                    .map(|id| id.name.as_str())
                    .collect::<Vec<_>>()
                    .join(".");
                
                if modifier_name.contains("initializer") || modifier_name.contains("reinitializer") {
                    return ValidationResult::Protected("Has initializer modifier");
                }
            }
        }
        ValidationResult::Vulnerable
    }

    fn check_access_control(&self, func: &FunctionDefinition) -> ValidationResult {
        // Check modifiers
        for attr in &func.attributes {
            if let FunctionAttribute::BaseOrModifier(_, base) = attr {
                let modifier_name = base.name.identifiers.iter()
                    .map(|id| id.name.as_str())
                    .collect::<Vec<_>>()
                    .join(".");
                
                let access_modifiers = [
                    "onlyOwner", "onlyAdmin", "onlyMinter", "onlyBurner",
                    "onlyRole", "onlyGovernance", "onlyController", "onlyManager",
                    "hasRole", "authorized", "onlyAuthorized"
                ];
                
                if access_modifiers.iter().any(|&m| modifier_name.contains(m)) {
                    return ValidationResult::Protected("Has access control modifier");
                }
            }
        }

        // Check inline access control in function body
        if let Some(Statement::Block { statements, .. }) = &func.body {
            for stmt in statements.iter().take(5) {
                if self.has_inline_access_check(stmt) {
                    return ValidationResult::Protected("Has inline access check");
                }
            }
        }

        ValidationResult::Vulnerable
    }

    fn has_inline_access_check(&self, stmt: &Statement) -> bool {
        match stmt {
            Statement::If(_, cond, then_branch, _) => {
                self.is_sender_check(cond) && self.is_revert_or_require(then_branch)
            }
            Statement::Expression(_, expr) => {
                self.is_require_with_sender(expr)
            }
            _ => false,
        }
    }

    fn is_sender_check(&self, expr: &Expression) -> bool {
        // Check for: msg.sender != owner, msg.sender == owner, etc.
        matches!(expr, Expression::Equal(..) | Expression::NotEqual(..) | Expression::More(..) | Expression::Less(..))
    }

    fn is_revert_or_require(&self, stmt: &Statement) -> bool {
        matches!(stmt, Statement::Revert(..) | Statement::Expression(..))
    }

    fn is_require_with_sender(&self, expr: &Expression) -> bool {
        // Check for: require(msg.sender == owner)
        matches!(expr, Expression::FunctionCall(..))
    }
}

impl Default for AstValidator {
    fn default() -> Self {
        Self::new()
    }
}
