use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaintSource {
    MsgSender,
    MsgValue,
    MsgData,
    CallData,
    TxOrigin,
    FunctionParam { func: String, param: String },
    ExternalCallReturn { target: String },
    BlockTimestamp,
    BlockNumber,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaintSink {
    EthTransfer,
    TokenTransfer,
    DelegateCall,
    ExternalCall,
    SelfDestruct,
    StorageWrite { var: String },
    AuthorizationCheck,
}

#[derive(Debug, Clone)]
pub struct TaintFlow {
    pub source: TaintSource,
    pub sink: TaintSink,
    pub path: Vec<String>,
    pub exploitability: Exploitability,
    pub exploit_scenario: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaintState {
    Tainted,
    PartiallyTainted,
    Sanitized,
    Clean,
}

#[derive(Debug, Clone)]
pub struct Exploitability {
    pub score: f64,
    pub confidence: f64,
    pub requires: Vec<String>,
    pub impact: ExploitImpact,
}

#[derive(Debug, Clone)]
pub enum ExploitImpact {
    FundsDrain,
    PrivilegeEscalation,
    DoS,
    DataCorruption,
}

pub struct TaintAnalyzer {
    taint_state: HashMap<String, TaintState>,
    sources: Vec<TaintSource>,
    sinks: Vec<TaintSink>,
    flows: Vec<TaintFlow>,
    _sanitizers: HashSet<String>,
}

impl TaintAnalyzer {
    pub fn new() -> Self {
        let mut sanitizers = HashSet::new();
        sanitizers.insert("require".to_string());
        sanitizers.insert("assert".to_string());
        sanitizers.insert("onlyOwner".to_string());

        Self {
            taint_state: HashMap::new(),
            sources: Vec::new(),
            sinks: Vec::new(),
            flows: Vec::new(),
            _sanitizers: sanitizers,
        }
    }

    pub fn add_source(&mut self, source: TaintSource) {
        self.sources.push(source);
    }

    pub fn add_sink(&mut self, sink: TaintSink) {
        self.sinks.push(sink);
    }

    pub fn mark_tainted(&mut self, var: &str) {
        self.taint_state
            .insert(var.to_string(), TaintState::Tainted);
    }

    pub fn is_tainted(&self, var: &str) -> bool {
        matches!(
            self.taint_state.get(var),
            Some(TaintState::Tainted) | Some(TaintState::PartiallyTainted)
        )
    }

    pub fn analyze(&mut self) -> Vec<TaintFlow> {
        self.flows.clear();

        for source in &self.sources.clone() {
            for sink in &self.sinks.clone() {
                if let Some(flow) = self.check_flow(source, sink) {
                    self.flows.push(flow);
                }
            }
        }

        self.flows.clone()
    }

    fn check_flow(&self, source: &TaintSource, sink: &TaintSink) -> Option<TaintFlow> {
        let exploitability = self.assess_exploitability(source, sink);

        if exploitability.score > 0.3 {
            Some(TaintFlow {
                source: source.clone(),
                sink: sink.clone(),
                path: vec![format!("{:?}", source), format!("{:?}", sink)],
                exploitability: exploitability.clone(),
                exploit_scenario: self.generate_exploit_scenario(source, sink, &exploitability),
            })
        } else {
            None
        }
    }

    fn assess_exploitability(&self, source: &TaintSource, sink: &TaintSink) -> Exploitability {
        let mut score: f64 = 0.5;
        let confidence: f64 = 0.7;
        let mut requires = Vec::new();

        let impact = match sink {
            TaintSink::EthTransfer | TaintSink::TokenTransfer => {
                score += 0.4;
                ExploitImpact::FundsDrain
            }
            TaintSink::DelegateCall => {
                score += 0.5;
                ExploitImpact::PrivilegeEscalation
            }
            TaintSink::SelfDestruct => {
                score += 0.5;
                ExploitImpact::DoS
            }
            TaintSink::StorageWrite { .. } => {
                score += 0.2;
                ExploitImpact::DataCorruption
            }
            _ => ExploitImpact::DataCorruption,
        };

        match source {
            TaintSource::FunctionParam { .. } => {
                score += 0.1;
            }
            TaintSource::ExternalCallReturn { .. } => {
                score += 0.15;
                requires.push("Control external contract".to_string());
            }
            TaintSource::TxOrigin => {
                score += 0.2;
                requires.push("Phishing attack".to_string());
            }
            _ => {}
        }

        Exploitability {
            score: score.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            requires,
            impact,
        }
    }

    fn generate_exploit_scenario(
        &self,
        source: &TaintSource,
        sink: &TaintSink,
        exploitability: &Exploitability,
    ) -> String {
        let mut scenario = String::new();

        scenario.push_str("## Exploit Scenario\n\n");
        scenario.push_str(&format!("**Source**: {:?}\n", source));
        scenario.push_str(&format!("**Sink**: {:?}\n", sink));
        scenario.push_str(&format!(
            "**Exploitability**: {:.0}%\n\n",
            exploitability.score * 100.0
        ));

        scenario.push_str("### Attack Steps:\n\n");

        match (source, sink) {
            (TaintSource::FunctionParam { func, param }, TaintSink::EthTransfer) => {
                scenario.push_str(&format!(
                    "1. Attacker calls `{}` with malicious `{}` value\n",
                    func, param
                ));
                scenario.push_str("2. Tainted data flows to ETH transfer\n");
                scenario.push_str("3. Funds sent to attacker-controlled address\n");
            }
            (TaintSource::ExternalCallReturn { target }, TaintSink::StorageWrite { var }) => {
                scenario.push_str(&format!(
                    "1. Attacker deploys malicious contract at {}\n",
                    target
                ));
                scenario.push_str("2. Malicious contract returns crafted data\n");
                scenario.push_str(&format!("3. Tainted data written to storage `{}`\n", var));
            }
            (TaintSource::TxOrigin, TaintSink::AuthorizationCheck) => {
                scenario.push_str("1. Attacker tricks victim into calling malicious contract\n");
                scenario.push_str("2. Malicious contract calls vulnerable function\n");
                scenario.push_str("3. tx.origin check passes with victim's address\n");
            }
            _ => {
                scenario.push_str("1. Attacker provides malicious input\n");
                scenario.push_str("2. Data flows without proper validation\n");
                scenario.push_str("3. Dangerous operation executed\n");
            }
        }

        if !exploitability.requires.is_empty() {
            scenario.push_str("\n### Prerequisites:\n");
            for req in &exploitability.requires {
                scenario.push_str(&format!("- {}\n", req));
            }
        }

        scenario
    }

    pub fn get_flows(&self) -> &[TaintFlow] {
        &self.flows
    }

    pub fn get_high_risk_flows(&self) -> Vec<&TaintFlow> {
        self.flows
            .iter()
            .filter(|f| f.exploitability.score >= 0.7)
            .collect()
    }
}

impl Default for TaintAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_taint_analyzer() {
        let mut analyzer = TaintAnalyzer::new();

        analyzer.add_source(TaintSource::FunctionParam {
            func: "withdraw".to_string(),
            param: "amount".to_string(),
        });

        analyzer.add_sink(TaintSink::EthTransfer);

        let flows = analyzer.analyze();
        assert!(!flows.is_empty());
    }

    #[test]
    fn test_taint_state() {
        let mut analyzer = TaintAnalyzer::new();
        analyzer.mark_tainted("user_input");
        assert!(analyzer.is_tainted("user_input"));
        assert!(!analyzer.is_tainted("safe_var"));
    }
}
