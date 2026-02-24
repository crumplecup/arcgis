# Planning Document Index

This index tracks all planning and documentation files in the repository
along with their last associated commit.

## Active Planning Documents

| Document | Purpose | Last Commit | Date |
|----------|---------|-------------|------|
| [docs/gap_analysis_testing_2026-02-14.md](docs/gap_analysis_testing_2026-02-14.md) | Gap analysis with testing results and method coverage tracking | Active | 2026-02-14 |

## Project Standards

| Document | Purpose | Last Commit | Date |
|----------|---------|-------------|------|
| [CLAUDE.md](CLAUDE.md) | Project instructions and coding standards | `babb719` - refactor(error): implement exceptional error handling with type preservation | 26 hours ago |

## Repository Documentation

| Document | Purpose | Last Commit | Date |
|----------|---------|-------------|------|
| [README.md](README.md) | Main repository documentation | `4bbda7b` - refactor: bring codebase to CLAUDE.md compliance standards | 2 days ago |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contribution guidelines | `f616763` - feat: initial SDK foundation with type-safe architecture | 2 days ago |

## Archived Documents

The following documents have been archived (removed from the repository
but accessible via git history):

- IMPLEMENTATION_PLAN.md - Original SDK implementation roadmap (superseded by FULL_COVERAGE_PLAN.md)
- ARCHITECTURE_DECISION.md - Architectural decisions and rationale
- USER_EXPERIENCE_COMPARISON.md - UX comparison and design decisions
- PROJECT_STATUS.md - Historical project status tracking
- CLAUDE_MD_COMPLIANCE.md - CLAUDE.md compliance checklist
- CLAUDE_COMPLIANCE_REVIEW.md - Compliance review findings
- PKCE_AUTHENTICATION_STRATEGY.md - OAuth PKCE strategy (wrong approach - requires browser)
- API_KEY_TESTING_STRATEGY.md - API key testing strategy (`e4925a2`)
- ARCGIS_REST_API_RESEARCH.md - ArcGIS REST API research and analysis (`f616763`)
- ASYNC_ELEVATION_PLAN.md - Async elevation service implementation plan (`0286067`)
- CHANGELOG-0.1.2.md - Version 0.1.2 changelog (`8007aaa`)
- COVERAGE_ROADMAP.md - Coverage roadmap (`9c249ea`)
- ESRI_GEOMETRY_INTEGRATION_PLAN.md - Geometry consolidation refactor (COMPLETE) (`a59665d`)
- PLANNING_ITEM_DATA_API.md - Item data API enhancement plan (`d657edf`)
- SERVICE_DEFINITION_TYPING_PLAN.md - Strong typing for service definitions (`0b8a051`)
- AUTHENTICATION_STRATEGY.md - Automated authentication strategy (`dd2c33d`)
- docs/assertion_audit_2026-02-22.md - Assertion audit (100% coverage achieved) (`ed6cbff`)
- docs/example_coverage_assessment.md - Comprehensive example coverage assessment (`2e0f782`)
- docs/examples_expansion_plan.md - Examples expansion plan (25% → 80% coverage) (`3039441`)
- docs/gap_analysis_2026-02-08.md - Gap analysis (35% coverage, 4 example recommendations) (`cb4337e`)
- docs/multi-tier-testing.md - Multi-tier testing strategy (`e4925a2`)

To view archived documents:

```bash
# List all commits that touched a file
git log --all --full-history -- <filename>

# View file at specific commit
git show <commit-hash>:<filename>
```

## Usage

When creating new planning documents:

1. Add the document to this index
2. Categorize it appropriately (Planning, Standards, Documentation)
3. Update this file when significant commits are made to planning docs
4. Use `git log -1 --format="%h - %s (%cr)" -- <file>` to get commit info

## Maintenance

This index should be updated:

- When new planning documents are created
- After major milestones or significant changes to planning docs
- Before release cycles
- During project reviews

To regenerate commit information:

```bash
for file in *.md; do
  echo "=== $file ==="
  git log -1 --format="%h - %s (%cr)" -- "$file"
done
```
