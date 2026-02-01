// SPDX-License-Identifier: PMPL-1.0-or-later
//! Contract validation logic

use crate::error::{PolicyError, Result};
use crate::parser::{A2mlDocument, Section};
use std::collections::HashSet;

/// Validation modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    /// Parse syntax only
    Lax,
    /// Validate structure (required fields)
    Checked,
    /// Verify legal compliance
    Attested,
}

/// Validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Contract being validated
    pub contract_path: String,

    /// Schema used for validation
    pub schema_path: String,

    /// Overall validation result
    pub valid: bool,

    /// Errors found
    pub errors: Vec<ValidationError>,

    /// Warnings found
    pub warnings: Vec<ValidationWarning>,

    /// Required clauses checked
    pub required_clauses: Vec<ClauseCheck>,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error type
    pub kind: ErrorKind,

    /// Human-readable message
    pub message: String,

    /// Location in contract (section, line)
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    MissingClause,
    InvalidValue,
    UnresolvedReference,
    StructureError,
    AttestationFailure,
}

#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning message
    pub message: String,

    /// Location
    pub location: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClauseCheck {
    /// Clause name (e.g., "source-protection")
    pub clause: String,

    /// Present in contract?
    pub present: bool,

    /// Value (if applicable)
    pub value: Option<String>,

    /// Expected value
    pub expected: Option<String>,
}

impl ValidationReport {
    pub fn new(contract_path: String, schema_path: String) -> Self {
        Self {
            contract_path,
            schema_path,
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            required_clauses: Vec::new(),
        }
    }

    pub fn add_error(&mut self, kind: ErrorKind, message: String, location: Option<String>) {
        self.valid = false;
        self.errors.push(ValidationError {
            kind,
            message,
            location,
        });
    }

    pub fn add_warning(&mut self, message: String, location: Option<String>) {
        self.warnings.push(ValidationWarning { message, location });
    }

    pub fn add_clause_check(&mut self, check: ClauseCheck) {
        if !check.present {
            self.valid = false;
        }
        self.required_clauses.push(check);
    }
}

/// Validator for contracts against schemas
pub struct Validator {
    schema: A2mlDocument,
    mode: ValidationMode,
}

impl Validator {
    pub fn new(schema: A2mlDocument, mode: ValidationMode) -> Self {
        Self { schema, mode }
    }

    /// Validate a contract against the loaded schema
    pub fn validate(&self, contract: &A2mlDocument, required_clauses: &[String]) -> ValidationReport {
        log::info!("Validating contract (mode: {:?})", self.mode);

        let mut report = ValidationReport::new(
            "contract".to_string(),  // TODO: Get actual path
            "schema".to_string(),
        );

        // Check required clauses
        for clause in required_clauses {
            let present = self.has_clause(contract, clause);
            report.add_clause_check(ClauseCheck {
                clause: clause.clone(),
                present,
                value: None,  // TODO: Extract actual value
                expected: None,  // TODO: Get from schema
            });
        }

        // Mode-specific validation
        match self.mode {
            ValidationMode::Lax => {
                // Just parse - already done
            }
            ValidationMode::Checked => {
                self.validate_structure(contract, &mut report);
            }
            ValidationMode::Attested => {
                self.validate_structure(contract, &mut report);
                self.validate_attestations(contract, &mut report);
            }
        }

        report
    }

    fn has_clause(&self, contract: &A2mlDocument, clause: &str) -> bool {
        // TODO: Implement clause detection
        // For now, just check if section heading matches
        contract.sections.iter().any(|s| {
            s.heading.to_lowercase().contains(&clause.to_lowercase())
        })
    }

    fn validate_structure(&self, contract: &A2mlDocument, report: &mut ValidationReport) {
        // Check for abstract
        if contract.abstract_text.is_none() {
            report.add_error(
                ErrorKind::MissingClause,
                "Contract missing @abstract section".to_string(),
                None,
            );
        }

        // Check for references
        if contract.references.is_empty() {
            report.add_warning(
                "Contract has no @refs section".to_string(),
                None,
            );
        }

        // TODO: More structural checks
    }

    fn validate_attestations(&self, contract: &A2mlDocument, report: &mut ValidationReport) {
        // TODO: Verify attestations against external sources
        // This is the "attested" mode - checks legal compliance

        log::debug!("Checking attestations...");

        // For each attestation, verify:
        // 1. External reference exists
        // 2. Claim is backed by reference
        // 3. Legal requirements met

        // Example: "Must comply with NUJ Code §1"
        // -> Check that contract has source protection clause
        // -> Verify it matches NUJ requirements
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new(
            "test-contract.a2ml".to_string(),
            "nuj-ethics.a2ml".to_string(),
        );

        assert!(report.valid);

        report.add_error(
            ErrorKind::MissingClause,
            "Missing source-protection".to_string(),
            Some("Section 6".to_string()),
        );

        assert!(!report.valid);
        assert_eq!(report.errors.len(), 1);
    }
}
