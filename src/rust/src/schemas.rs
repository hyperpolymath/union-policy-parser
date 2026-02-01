// SPDX-License-Identifier: PMPL-1.0-or-later
//! Union-specific schema definitions and helpers

use crate::error::{PolicyError, Result};
use std::collections::HashMap;

/// Known unions with schema mappings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Union {
    /// National Union of Journalists
    Nuj,
    /// Industrial Workers of the World
    Iww,
    /// University and College Union
    Ucu,
}

impl Union {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "nuj" => Ok(Union::Nuj),
            "iww" => Ok(Union::Iww),
            "ucu" => Ok(Union::Ucu),
            _ => Err(PolicyError::UnknownUnion(s.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Union::Nuj => "nuj",
            Union::Iww => "iww",
            Union::Ucu => "ucu",
        }
    }

    /// Get default schema path for this union
    pub fn default_schema_path(&self) -> &'static str {
        match self {
            Union::Nuj => "schemas/nuj-code-of-ethics.a2ml",
            Union::Iww => "schemas/iww-freelancer-rights.a2ml",
            Union::Ucu => "schemas/ucu-academic-standards.a2ml",
        }
    }

    /// Get required clauses for this union
    pub fn required_clauses(&self) -> Vec<&'static str> {
        match self {
            Union::Nuj => vec![
                "truth-accuracy",
                "independence",
                "fairness",
                "privacy-harassment",
                "accountability",
                "source-protection",
                "anti-discrimination",
                "no-plagiarism",
            ],
            Union::Iww => vec![
                "payment-terms",
                "late-payment-penalty",
                "collective-voice",
                "no-free-trials",
                "no-spec-work",
                "kill-fee-provision",
            ],
            Union::Ucu => vec![
                "academic-freedom",
                "workload-limits",
                "research-time",
                "teaching-load",
                "no-casualization",
            ],
        }
    }

    /// Get recommended clauses (SHOULD have)
    pub fn recommended_clauses(&self) -> Vec<&'static str> {
        match self {
            Union::Nuj => vec![
                "transparency",
                "diversity",
                "accessibility",
                "environmental-impact",
            ],
            Union::Iww => vec![
                "portable-benefits",
                "equipment-allowance",
                "training-budget",
            ],
            Union::Ucu => vec![
                "sabbatical-provision",
                "conference-funding",
                "phd-supervision-limits",
            ],
        }
    }

    /// Get exploitative patterns to watch for
    pub fn red_flag_patterns(&self) -> Vec<&'static str> {
        match self {
            Union::Nuj => vec![
                "all rights",
                "work for hire",
                "perpetual license",
                "no source protection",
                "editorial override",
            ],
            Union::Iww => vec![
                "free trial",
                "spec work",
                "unpaid",
                "payment on publication",
                "no kill fee",
                "NET 60",
                "NET 90",
            ],
            Union::Ucu => vec![
                "unlimited hours",
                "no research time",
                "casualization",
                "zero hours",
                "no sabbatical",
            ],
        }
    }
}

/// Union-specific validation rules
pub struct UnionRules {
    union: Union,
    custom_rules: HashMap<String, String>,
}

impl UnionRules {
    pub fn new(union: Union) -> Self {
        Self {
            union,
            custom_rules: HashMap::new(),
        }
    }

    /// Check if a clause value meets union standards
    pub fn check_clause_value(&self, clause: &str, value: &str) -> Result<bool> {
        match self.union {
            Union::Nuj => self.check_nuj_clause(clause, value),
            Union::Iww => self.check_iww_clause(clause, value),
            Union::Ucu => self.check_ucu_clause(clause, value),
        }
    }

    fn check_nuj_clause(&self, clause: &str, value: &str) -> Result<bool> {
        match clause {
            "source-protection" => {
                // Must be "guaranteed" or "true"
                Ok(value.to_lowercase() == "guaranteed" || value == "true")
            }
            "editorial-independence" => {
                // Must be "true"
                Ok(value == "true")
            }
            "copyright-ownership" => {
                // Must be "freelancer" or "first-publication-only"
                Ok(value == "freelancer" || value == "first-publication-only")
            }
            _ => Ok(true),  // No specific check
        }
    }

    fn check_iww_clause(&self, clause: &str, value: &str) -> Result<bool> {
        match clause {
            "payment-terms.net-days" => {
                // Must be ≤ 30
                let days: u32 = value.parse()
                    .map_err(|_| PolicyError::ValidationError(format!("Invalid NET days: {}", value)))?;
                Ok(days <= 30)
            }
            "late-payment-penalty" => {
                // Must be ≥ 5%
                let penalty: f64 = value.trim_end_matches('%').parse()
                    .map_err(|_| PolicyError::ValidationError(format!("Invalid penalty: {}", value)))?;
                Ok(penalty >= 5.0)
            }
            "kill-fee" => {
                // Must be ≥ 50%
                let fee: f64 = value.trim_end_matches('%').parse()
                    .map_err(|_| PolicyError::ValidationError(format!("Invalid kill fee: {}", value)))?;
                Ok(fee >= 50.0)
            }
            _ => Ok(true),
        }
    }

    fn check_ucu_clause(&self, clause: &str, value: &str) -> Result<bool> {
        match clause {
            "academic-freedom" => {
                // Must be "guaranteed"
                Ok(value.to_lowercase() == "guaranteed")
            }
            "workload-hours-max" => {
                // Must be ≤ 40 hours/week
                let hours: u32 = value.parse()
                    .map_err(|_| PolicyError::ValidationError(format!("Invalid hours: {}", value)))?;
                Ok(hours <= 40)
            }
            _ => Ok(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_union_from_str() {
        assert_eq!(Union::from_str("nuj").unwrap(), Union::Nuj);
        assert_eq!(Union::from_str("NUJ").unwrap(), Union::Nuj);
        assert!(Union::from_str("unknown").is_err());
    }

    #[test]
    fn test_nuj_required_clauses() {
        let clauses = Union::Nuj.required_clauses();
        assert!(clauses.contains(&"source-protection"));
        assert!(clauses.contains(&"editorial-independence"));
    }

    #[test]
    fn test_iww_red_flags() {
        let patterns = Union::Iww.red_flag_patterns();
        assert!(patterns.contains(&"free trial"));
        assert!(patterns.contains(&"spec work"));
    }

    #[test]
    fn test_nuj_clause_check() {
        let rules = UnionRules::new(Union::Nuj);
        assert!(rules.check_clause_value("source-protection", "guaranteed").unwrap());
        assert!(!rules.check_clause_value("source-protection", "optional").unwrap());
    }

    #[test]
    fn test_iww_clause_check() {
        let rules = UnionRules::new(Union::Iww);
        assert!(rules.check_clause_value("payment-terms.net-days", "30").unwrap());
        assert!(!rules.check_clause_value("payment-terms.net-days", "60").unwrap());
    }
}
