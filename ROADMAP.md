# Union Policy Parser - Roadmap

## Current Status

This repository implements a hub-and-spoke mirroring strategy for secure, multi-platform code distribution.

### Completed

- [x] **Hub-and-Spoke Mirror Workflow** - Automated mirroring to GitLab, Codeberg, and Bitbucket
- [x] **Security Hardening** (v2)
  - SSH known hosts verification (prevents MITM attacks)
  - Concurrency control (prevents race conditions)
  - Repository name validation (prevents injection attacks)
  - Pinned action versions with SHA hashes (supply chain security)
  - Minimal permissions model (`contents: read` only)
  - Job timeout limits (prevents runaway jobs)
  - Strict bash error handling (`set -euo pipefail`)

---

## Phase 1: Infrastructure & Security

### 1.1 Core Repository Setup
- [ ] Add LICENSE file (AGPL-3.0-or-later)
- [ ] Add README.md with project documentation
- [ ] Add CONTRIBUTING.md guidelines
- [ ] Add SECURITY.md with vulnerability reporting process
- [ ] Configure branch protection rules

### 1.2 CI/CD Enhancements
- [ ] Add workflow status badges to README
- [ ] Implement Slack/Discord notifications for mirror failures
- [ ] Add retry logic for transient network failures
- [ ] Create workflow for testing mirror configuration changes
- [ ] Add scheduled sync job for drift detection

### 1.3 Security Monitoring
- [ ] Implement Dependabot for action updates
- [ ] Add secret scanning configuration
- [ ] Configure CODEOWNERS file
- [ ] Add audit logging for workflow runs

---

## Phase 2: Policy Parser Core

### 2.1 Parser Foundation
- [ ] Define policy schema specification
- [ ] Implement YAML/JSON policy parser
- [ ] Add schema validation layer
- [ ] Create policy AST representation
- [ ] Implement policy normalization

### 2.2 Union Operations
- [ ] Implement policy merge strategies
  - [ ] Override (last wins)
  - [ ] Union (combine)
  - [ ] Intersection (common)
  - [ ] Priority-based merge
- [ ] Add conflict detection and resolution
- [ ] Implement policy inheritance chains

### 2.3 Policy Types Support
- [ ] GitHub Actions permissions policies
- [ ] Repository access control policies
- [ ] Branch protection policies
- [ ] Secret management policies
- [ ] Audit and compliance policies

---

## Phase 3: Integration & Tooling

### 3.1 CLI Tool
- [ ] Create CLI interface for policy management
- [ ] Add `validate` command
- [ ] Add `merge` command
- [ ] Add `diff` command
- [ ] Add `apply` command

### 3.2 GitHub Integration
- [ ] GitHub App for automated policy enforcement
- [ ] PR checks for policy compliance
- [ ] Auto-remediation suggestions
- [ ] Policy drift detection

### 3.3 Multi-Platform Support
- [ ] GitLab CI/CD policy support
- [ ] Bitbucket Pipelines policy support
- [ ] Codeberg/Gitea policy support

---

## Phase 4: Enterprise Features

### 4.1 Governance
- [ ] Organization-wide policy templates
- [ ] Policy versioning and rollback
- [ ] Compliance reporting dashboard
- [ ] Policy exception workflows

### 4.2 Advanced Security
- [ ] Policy signing and verification
- [ ] Immutable policy audit trail
- [ ] SBOM integration
- [ ] SLSA compliance tooling

---

## Security Considerations

### Current Mitigations (Implemented)

| Risk | Mitigation |
|------|------------|
| SSH MITM attacks | Known hosts verification via `ssh-keyscan` |
| Supply chain attacks | Pinned actions with SHA256 hashes |
| Race conditions | Concurrency groups with queuing |
| Command injection | Repository name validation regex |
| Privilege escalation | Minimal `contents: read` permissions |
| Runaway jobs | 10-minute timeout per job |
| Shell errors | Strict mode (`set -euo pipefail`) |

### Future Security Work

- [ ] Implement signed commits for mirror pushes
- [ ] Add OIDC authentication for cloud providers
- [ ] Secret rotation automation
- [ ] Key pinning for SSH connections
- [ ] Workflow attestation with Sigstore

---

## Contributing

See CONTRIBUTING.md for guidelines on how to contribute to this project.

## License

AGPL-3.0-or-later
