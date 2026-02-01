// SPDX-License-Identifier: PMPL-1.0-or-later
//! Union Policy Parser - CLI for validating employment contracts against union standards
//!
//! Supports: NUJ (National Union of Journalists), IWW (Industrial Workers of the World),
//! UCU (University and College Union), and general UK employment law compliance.

use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;
use std::fs;

mod parser;
mod validator;
mod reporter;
mod schemas;
mod error;

use crate::error::PolicyError;
use crate::parser::{parse_a2ml_file, parse_a2ml_string};
use crate::validator::{Validator, ValidationMode as ValidatorMode};
use crate::reporter::{GrievanceGenerator, ReportRenderer};
use crate::schemas::Union;

/// Union Policy Parser - Validate contracts against union ethics and employment law
#[derive(Parser)]
#[command(name = "union-policy-parser")]
#[command(author = "Jonathan D.A. Jewell <jonathan.jewell@open.ac.uk>")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Validate employment contracts against union standards (NUJ/IWW/UCU)")]
#[command(long_about = r#"
Union Policy Parser validates employment contracts, editorial policies, and collective
agreements against union ethical standards (NUJ Code of Ethics, IWW Freelancer Rights, etc.)
using A2ML (Attested Markup Language) for structured, machine-readable validation.

Examples:
  # Validate a contract
  union-policy-parser validate contract.a2ml --schema schemas/nuj-ethics.a2ml

  # Generate grievance for violations
  union-policy-parser grievance contract.a2ml --violation source-protection

  # Batch validate multiple contracts
  union-policy-parser batch contracts/ --schema nuj-ethics.a2ml --output report.json

License: PMPL-1.0-or-later (Palimpsest Mozilla Public License)
"#)]
struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a contract against a schema
    Validate {
        /// Path to A2ML contract file
        #[arg(value_name = "CONTRACT")]
        contract: PathBuf,

        /// Path to A2ML schema file (e.g., nuj-code-of-ethics.a2ml)
        #[arg(short, long, value_name = "SCHEMA")]
        schema: PathBuf,

        /// Validation mode: lax, checked, or attested
        #[arg(short, long, default_value = "checked")]
        mode: ValidationMode,

        /// Union to validate for (nuj, iww, ucu)
        #[arg(short, long, value_name = "UNION")]
        union: Option<String>,

        /// Comma-separated list of required clauses
        #[arg(long, value_delimiter = ',')]
        required_clauses: Vec<String>,

        /// Exit with error code if validation fails
        #[arg(long)]
        strict: bool,
    },

    /// Generate an audit report
    Audit {
        /// Path to A2ML contract file
        #[arg(value_name = "CONTRACT")]
        contract: PathBuf,

        /// Path to A2ML schema file
        #[arg(short, long, value_name = "SCHEMA")]
        schema: PathBuf,

        /// Output file path (JSON format)
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,

        /// Union to audit for
        #[arg(short, long)]
        union: Option<String>,
    },

    /// Auto-generate a grievance letter for violations
    Grievance {
        /// Path to A2ML contract file
        #[arg(value_name = "CONTRACT")]
        contract: PathBuf,

        /// Violation type (e.g., missing-source-protection)
        #[arg(short, long)]
        violation: String,

        /// Path to grievance template (Markdown)
        #[arg(short, long, value_name = "TEMPLATE")]
        template: Option<PathBuf>,

        /// Output file path
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,

        /// Union context (nuj, iww, ucu)
        #[arg(short, long)]
        union: Option<String>,

        /// Path to schema for validation context
        #[arg(short, long, value_name = "SCHEMA")]
        schema: Option<PathBuf>,
    },

    /// Batch validate multiple contracts
    Batch {
        /// Directory containing A2ML contract files
        #[arg(value_name = "DIR")]
        dir: PathBuf,

        /// Path to A2ML schema file
        #[arg(short, long, value_name = "SCHEMA")]
        schema: PathBuf,

        /// Output report file (JSON format)
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,

        /// Union to validate for
        #[arg(short, long)]
        union: Option<String>,

        /// Validation mode
        #[arg(short, long, default_value = "checked")]
        mode: ValidationMode,
    },

    /// Check a specific clause value
    CheckClause {
        /// Path to A2ML contract file
        #[arg(value_name = "CONTRACT")]
        contract: PathBuf,

        /// Clause path (e.g., "payment.terms.net-days")
        #[arg(short, long)]
        clause: String,

        /// Expected value
        #[arg(short, long)]
        expected: Option<String>,

        /// Minimum value (for numeric clauses)
        #[arg(long)]
        min: Option<f64>,

        /// Maximum value (for numeric clauses)
        #[arg(long)]
        max: Option<f64>,

        /// Allowed values (comma-separated)
        #[arg(long, value_delimiter = ',')]
        allowed: Vec<String>,

        /// Exit with error if check fails
        #[arg(long)]
        error_if_not: bool,

        /// Warn if check fails (instead of error)
        #[arg(long)]
        warn_if_not: bool,
    },

    /// Get a clause value
    GetClause {
        /// Path to A2ML contract file
        #[arg(value_name = "CONTRACT")]
        contract: PathBuf,

        /// Clause path (e.g., "source-protection.guaranteed")
        #[arg(short, long)]
        clause: String,
    },

    /// Scan for red flag keywords (exploitative clauses)
    ScanRedFlags {
        /// Path to A2ML contract file
        #[arg(value_name = "CONTRACT")]
        contract: PathBuf,

        /// Red flag patterns (e.g., "all rights", "work for hire")
        #[arg(short, long, value_delimiter = ',')]
        patterns: Vec<String>,

        /// Case-insensitive matching
        #[arg(short = 'i', long)]
        case_insensitive: bool,
    },

    /// Render contract to HTML/Markdown
    Render {
        /// Path to A2ML contract file
        #[arg(value_name = "CONTRACT")]
        contract: PathBuf,

        /// Output format (html, markdown, json)
        #[arg(short, long, default_value = "html")]
        format: OutputFormat,

        /// Output file path
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,

        /// Template file (optional)
        #[arg(short, long)]
        template: Option<PathBuf>,
    },

    /// Check schema validity
    CheckSchema {
        /// Path to A2ML schema file
        #[arg(value_name = "SCHEMA")]
        schema: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum ValidationMode {
    /// Parse A2ML syntax only
    Lax,
    /// Validate structure (required fields, references)
    Checked,
    /// Verify legal compliance (UK law, union standards)
    Attested,
}

impl From<ValidationMode> for ValidatorMode {
    fn from(mode: ValidationMode) -> Self {
        match mode {
            ValidationMode::Lax => ValidatorMode::Lax,
            ValidationMode::Checked => ValidatorMode::Checked,
            ValidationMode::Attested => ValidatorMode::Attested,
        }
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum OutputFormat {
    Html,
    Markdown,
    Json,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logger
    let log_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    log::info!("Union Policy Parser v{}", env!("CARGO_PKG_VERSION"));

    // Dispatch to subcommand handlers
    match cli.command {
        Commands::Validate {
            contract,
            schema,
            mode,
            union,
            required_clauses,
            strict,
        } => cmd_validate(contract, schema, mode, union, required_clauses, strict)?,

        Commands::Audit {
            contract,
            schema,
            output,
            union,
        } => cmd_audit(contract, schema, output, union)?,

        Commands::Grievance {
            contract,
            violation,
            template,
            output,
            union,
            schema,
        } => cmd_grievance(contract, violation, template, output, union, schema)?,

        Commands::Batch {
            dir,
            schema,
            output,
            union,
            mode,
        } => cmd_batch(dir, schema, output, union, mode)?,

        Commands::CheckClause {
            contract,
            clause,
            expected,
            min,
            max,
            allowed,
            error_if_not,
            warn_if_not,
        } => cmd_check_clause(contract, clause, expected, min, max, allowed, error_if_not, warn_if_not)?,

        Commands::GetClause { contract, clause } => cmd_get_clause(contract, clause)?,

        Commands::ScanRedFlags {
            contract,
            patterns,
            case_insensitive,
        } => cmd_scan_red_flags(contract, patterns, case_insensitive)?,

        Commands::Render {
            contract,
            format,
            output,
            template,
        } => cmd_render(contract, format, output, template)?,

        Commands::CheckSchema { schema } => cmd_check_schema(schema)?,
    }

    Ok(())
}

// ============================================================================
// Command Handlers
// ============================================================================

fn cmd_validate(
    contract_path: PathBuf,
    schema_path: PathBuf,
    mode: ValidationMode,
    union: Option<String>,
    required_clauses: Vec<String>,
    strict: bool,
) -> Result<()> {
    log::info!("Validating contract: {:?}", contract_path);
    log::info!("Schema: {:?}", schema_path);
    log::info!("Mode: {:?}", mode);

    // Parse contract
    let contract = parse_a2ml_file(&contract_path)?;
    println!("✅ Contract parsed successfully");
    println!("   Abstract: {}", if contract.abstract_text.is_some() { "present" } else { "missing" });
    println!("   Sections: {}", contract.sections.len());
    println!("   References: {}", contract.references.len());
    println!("   Requirements: {}", contract.requirements.len());

    // Parse schema
    let schema = parse_a2ml_file(&schema_path)?;
    println!("✅ Schema parsed successfully");

    // Get union-specific required clauses if union specified
    let mut all_required_clauses = required_clauses.clone();
    if let Some(union_name) = &union {
        let union_enum = Union::from_str(union_name)?;
        all_required_clauses.extend(
            union_enum.required_clauses().iter().map(|s| s.to_string())
        );
        println!("📋 Union: {} ({} required clauses)", union_name.to_uppercase(), union_enum.required_clauses().len());
    }

    // Validate
    let validator = Validator::new(schema, mode.into());
    let report = validator.validate(&contract, &all_required_clauses);

    // Display results
    println!("\n{}", "=".repeat(60));
    if report.valid {
        println!("✅ VALID: Contract complies with schema");
    } else {
        println!("❌ INVALID: Contract has violations");
    }
    println!("{}", "=".repeat(60));

    if !report.errors.is_empty() {
        println!("\n❌ Errors ({}):", report.errors.len());
        for error in &report.errors {
            println!("   - {}", error.message);
            if let Some(loc) = &error.location {
                println!("     Location: {}", loc);
            }
        }
    }

    if !report.warnings.is_empty() {
        println!("\n⚠️  Warnings ({}):", report.warnings.len());
        for warning in &report.warnings {
            println!("   - {}", warning.message);
        }
    }

    if !report.required_clauses.is_empty() {
        println!("\n📋 Required Clauses:");
        for clause_check in &report.required_clauses {
            let status = if clause_check.present { "✓" } else { "✗" };
            println!("   {} {}", status, clause_check.clause);
        }
    }

    if strict && !report.valid {
        anyhow::bail!("Validation failed (strict mode)");
    }

    Ok(())
}

fn cmd_audit(
    contract_path: PathBuf,
    schema_path: PathBuf,
    output_path: PathBuf,
    union: Option<String>,
) -> Result<()> {
    log::info!("Auditing contract: {:?}", contract_path);

    // Parse contract and schema
    let contract = parse_a2ml_file(&contract_path)?;
    let schema = parse_a2ml_file(&schema_path)?;

    // Get union-specific clauses
    let required_clauses = if let Some(union_name) = &union {
        let union_enum = Union::from_str(union_name)?;
        union_enum.required_clauses().iter().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    };

    // Validate
    let validator = Validator::new(schema, ValidatorMode::Attested);
    let report = validator.validate(&contract, &required_clauses);

    // Render to JSON
    let json = ReportRenderer::render_json(&report)?;

    // Write to file
    fs::write(&output_path, json)?;

    println!("✅ Audit report saved to: {:?}", output_path);
    println!("   Valid: {}", report.valid);
    println!("   Errors: {}", report.errors.len());
    println!("   Warnings: {}", report.warnings.len());

    Ok(())
}

fn cmd_grievance(
    contract_path: PathBuf,
    violation: String,
    template_path: Option<PathBuf>,
    output_path: PathBuf,
    union: Option<String>,
    schema_path: Option<PathBuf>,
) -> Result<()> {
    log::info!("Generating grievance for: {}", violation);

    // Parse contract
    let contract = parse_a2ml_file(&contract_path)?;

    // Validate if schema provided
    let report = if let Some(schema_path) = schema_path {
        let schema = parse_a2ml_file(&schema_path)?;
        let validator = Validator::new(schema, ValidatorMode::Attested);
        validator.validate(&contract, &vec![])
    } else {
        validator::ValidationReport::new(
            contract_path.to_string_lossy().to_string(),
            "no-schema".to_string(),
        )
    };

    // Generate grievance
    let generator = GrievanceGenerator::new(union, template_path.as_deref())?;
    let grievance = generator.generate(&violation, &report)?;

    // Write to file
    fs::write(&output_path, grievance)?;

    println!("✅ Grievance letter saved to: {:?}", output_path);

    Ok(())
}

fn cmd_batch(
    dir: PathBuf,
    schema_path: PathBuf,
    output_path: PathBuf,
    union: Option<String>,
    mode: ValidationMode,
) -> Result<()> {
    log::info!("Batch validating contracts in: {:?}", dir);

    use walkdir::WalkDir;

    // Find all .a2ml files
    let mut a2ml_files = Vec::new();
    for entry in WalkDir::new(&dir).follow_links(true) {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if ext == "a2ml" {
                    a2ml_files.push(entry.path().to_path_buf());
                }
            }
        }
    }

    println!("Found {} A2ML files", a2ml_files.len());

    // Parse schema
    let schema = parse_a2ml_file(&schema_path)?;

    // Get union clauses
    let required_clauses = if let Some(union_name) = &union {
        let union_enum = Union::from_str(union_name)?;
        union_enum.required_clauses().iter().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    };

    // Validate each file
    let mut all_reports = Vec::new();
    for file in &a2ml_files {
        println!("Validating: {:?}", file);
        match parse_a2ml_file(file) {
            Ok(contract) => {
                let validator = Validator::new(schema.clone(), mode.into());
                let report = validator.validate(&contract, &required_clauses);
                all_reports.push(serde_json::json!({
                    "file": file.to_string_lossy(),
                    "valid": report.valid,
                    "errors": report.errors.len(),
                    "warnings": report.warnings.len(),
                }));
            }
            Err(e) => {
                eprintln!("❌ Failed to parse {:?}: {}", file, e);
            }
        }
    }

    // Write batch report
    let batch_report = serde_json::json!({
        "total_files": a2ml_files.len(),
        "results": all_reports,
    });

    fs::write(&output_path, serde_json::to_string_pretty(&batch_report)?)?;
    println!("✅ Batch report saved to: {:?}", output_path);

    Ok(())
}

fn cmd_check_clause(
    contract_path: PathBuf,
    clause: String,
    expected: Option<String>,
    min: Option<f64>,
    max: Option<f64>,
    allowed: Vec<String>,
    error_if_not: bool,
    warn_if_not: bool,
) -> Result<()> {
    log::info!("Checking clause: {}", clause);

    let contract = parse_a2ml_file(&contract_path)?;

    // Simple clause lookup (check if section heading contains clause)
    let found = contract.sections.iter().any(|s| {
        s.heading.to_lowercase().contains(&clause.to_lowercase())
    });

    if found {
        println!("✓ Clause '{}' found", clause);
    } else {
        println!("✗ Clause '{}' NOT found", clause);
        if error_if_not {
            anyhow::bail!("Clause check failed");
        }
    }

    Ok(())
}

fn cmd_get_clause(contract_path: PathBuf, clause: String) -> Result<()> {
    log::info!("Getting clause value: {}", clause);

    let contract = parse_a2ml_file(&contract_path)?;

    // Find section with matching heading
    if let Some(section) = contract.sections.iter().find(|s| {
        s.heading.to_lowercase().contains(&clause.to_lowercase())
    }) {
        println!("Clause: {}", section.heading);
        println!("Content:");
        for block in &section.content {
            match block {
                parser::ContentBlock::Paragraph(text) => println!("{}", text),
                parser::ContentBlock::BulletList(items) => {
                    for item in items {
                        println!("- {}", item);
                    }
                }
                _ => {}
            }
        }
    } else {
        println!("Clause '{}' not found", clause);
    }

    Ok(())
}

fn cmd_scan_red_flags(
    contract_path: PathBuf,
    patterns: Vec<String>,
    case_insensitive: bool,
) -> Result<()> {
    log::info!("Scanning for red flags: {:?}", patterns);

    let contract = parse_a2ml_file(&contract_path)?;
    let contract_text = serde_json::to_string(&contract)?;

    let mut found_flags = Vec::new();

    for pattern in &patterns {
        let search_pattern = if case_insensitive {
            pattern.to_lowercase()
        } else {
            pattern.clone()
        };

        let search_text = if case_insensitive {
            contract_text.to_lowercase()
        } else {
            contract_text.clone()
        };

        if search_text.contains(&search_pattern) {
            found_flags.push(pattern.clone());
        }
    }

    if found_flags.is_empty() {
        println!("✅ No red flags found");
    } else {
        println!("⚠️  Red flags detected ({}):", found_flags.len());
        for flag in found_flags {
            println!("   - {}", flag);
        }
    }

    Ok(())
}

fn cmd_render(
    contract_path: PathBuf,
    format: OutputFormat,
    output_path: PathBuf,
    _template: Option<PathBuf>,
) -> Result<()> {
    log::info!("Rendering contract to: {:?}", output_path);

    let contract = parse_a2ml_file(&contract_path)?;

    let output = match format {
        OutputFormat::Json => serde_json::to_string_pretty(&contract)?,
        OutputFormat::Markdown => {
            // Simple Markdown rendering
            let mut md = String::new();
            if let Some(abstract_text) = &contract.abstract_text {
                md.push_str("## Abstract\n\n");
                md.push_str(abstract_text);
                md.push_str("\n\n");
            }
            for section in &contract.sections {
                md.push_str(&"#".repeat(section.level as usize + 1));
                md.push_str(" ");
                md.push_str(&section.heading);
                md.push_str("\n\n");
            }
            md
        }
        OutputFormat::Html => {
            format!("<pre>{}</pre>", serde_json::to_string_pretty(&contract)?)
        }
    };

    fs::write(&output_path, output)?;
    println!("✅ Rendered to: {:?}", output_path);

    Ok(())
}

fn cmd_check_schema(schema_path: PathBuf) -> Result<()> {
    log::info!("Checking schema: {:?}", schema_path);

    let schema = parse_a2ml_file(&schema_path)?;

    println!("✅ Schema is valid A2ML");
    println!("   Sections: {}", schema.sections.len());
    println!("   References: {}", schema.references.len());
    println!("   Requirements: {}", schema.requirements.len());

    // Check for common schema elements
    if schema.abstract_text.is_none() {
        println!("⚠️  Warning: Schema missing abstract");
    }
    if schema.references.is_empty() {
        println!("⚠️  Warning: Schema has no references");
    }

    Ok(())
}
