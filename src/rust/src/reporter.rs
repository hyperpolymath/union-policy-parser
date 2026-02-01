// SPDX-License-Identifier: PMPL-1.0-or-later
//! Grievance and report generation

use crate::error::{PolicyError, Result};
use crate::validator::ValidationReport;
use std::path::Path;
use std::fs;

/// Grievance generator
pub struct GrievanceGenerator {
    /// Union context (nuj, iww, ucu)
    union: Option<String>,

    /// Template content
    template: Option<String>,
}

impl GrievanceGenerator {
    pub fn new(union: Option<String>, template_path: Option<&Path>) -> Result<Self> {
        let template = if let Some(path) = template_path {
            Some(fs::read_to_string(path)?)
        } else {
            None
        };

        Ok(Self { union, template })
    }

    /// Generate a grievance letter for a violation
    pub fn generate(
        &self,
        violation: &str,
        validation_report: &ValidationReport,
    ) -> Result<String> {
        log::info!("Generating grievance for: {}", violation);

        let template = self.template.as_ref().ok_or_else(|| {
            PolicyError::TemplateError("No template provided".to_string())
        })?;

        // TODO: Implement template substitution
        // Variables:
        // - {{violation}}
        // - {{date}}
        // - {{contract_id}}
        // - {{union}}
        // - {{nuj_code_section}}
        // - {{legal_reference}}
        // - {{required_action}}

        Ok(format!(
            "# GRIEVANCE LETTER\n\n\
            Violation: {}\n\
            Union: {}\n\
            Contract: {}\n\
            Schema: {}\n\n\
            Errors found:\n{}\n",
            violation,
            self.union.as_deref().unwrap_or("N/A"),
            validation_report.contract_path,
            validation_report.schema_path,
            self.format_errors(&validation_report.errors),
        ))
    }

    fn format_errors(&self, errors: &[crate::validator::ValidationError]) -> String {
        errors
            .iter()
            .map(|e| format!("  - {}", e.message))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Report renderer (JSON, HTML, Markdown)
pub struct ReportRenderer;

impl ReportRenderer {
    /// Render validation report as JSON
    pub fn render_json(report: &ValidationReport) -> Result<String> {
        serde_json::to_string_pretty(&serde_json::json!({
            "contract": report.contract_path,
            "schema": report.schema_path,
            "valid": report.valid,
            "errors": report.errors.iter().map(|e| serde_json::json!({
                "kind": format!("{:?}", e.kind),
                "message": e.message,
                "location": e.location,
            })).collect::<Vec<_>>(),
            "warnings": report.warnings.iter().map(|w| serde_json::json!({
                "message": w.message,
                "location": w.location,
            })).collect::<Vec<_>>(),
            "required_clauses": report.required_clauses.iter().map(|c| serde_json::json!({
                "clause": c.clause,
                "present": c.present,
                "value": c.value,
                "expected": c.expected,
            })).collect::<Vec<_>>(),
        }))
        .map_err(|e| e.into())
    }

    /// Render validation report as Markdown
    pub fn render_markdown(report: &ValidationReport) -> Result<String> {
        let mut md = String::new();

        md.push_str("# Validation Report\n\n");
        md.push_str(&format!("**Contract:** `{}`\n", report.contract_path));
        md.push_str(&format!("**Schema:** `{}`\n\n", report.schema_path));

        if report.valid {
            md.push_str("## ✅ VALID\n\n");
        } else {
            md.push_str("## ❌ INVALID\n\n");
        }

        if !report.errors.is_empty() {
            md.push_str("### Errors\n\n");
            for error in &report.errors {
                md.push_str(&format!("- **{:?}**: {}\n", error.kind, error.message));
                if let Some(loc) = &error.location {
                    md.push_str(&format!("  - Location: {}\n", loc));
                }
            }
            md.push_str("\n");
        }

        if !report.warnings.is_empty() {
            md.push_str("### Warnings\n\n");
            for warning in &report.warnings {
                md.push_str(&format!("- {}\n", warning.message));
            }
            md.push_str("\n");
        }

        if !report.required_clauses.is_empty() {
            md.push_str("### Required Clauses\n\n");
            md.push_str("| Clause | Present | Value |\n");
            md.push_str("|--------|---------|-------|\n");
            for clause in &report.required_clauses {
                let present = if clause.present { "✓" } else { "✗" };
                let value = clause.value.as_deref().unwrap_or("N/A");
                md.push_str(&format!("| {} | {} | {} |\n", clause.clause, present, value));
            }
        }

        Ok(md)
    }

    /// Render validation report as HTML
    pub fn render_html(report: &ValidationReport) -> Result<String> {
        let md = Self::render_markdown(report)?;
        // TODO: Convert Markdown to HTML
        // Options: pulldown-cmark, comrak
        Ok(format!("<pre>{}</pre>", html_escape(&md)))
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator::{ValidationReport, ErrorKind};

    #[test]
    fn test_render_json() {
        let mut report = ValidationReport::new(
            "test.a2ml".to_string(),
            "nuj.a2ml".to_string(),
        );
        report.add_error(
            ErrorKind::MissingClause,
            "Missing clause".to_string(),
            Some("Section 1".to_string()),
        );

        let json = ReportRenderer::render_json(&report).unwrap();
        assert!(json.contains("test.a2ml"));
        assert!(json.contains("Missing clause"));
    }
}
