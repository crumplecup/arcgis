# Planning Document Index

This index tracks all planning and documentation files in the repository
along with their last associated commit.

## Active Planning Documents

| Document | Purpose | Last Commit | Date |
|----------|---------|-------------|------|
| [ARCGIS_REST_API_RESEARCH.md](ARCGIS_REST_API_RESEARCH.md) | ArcGIS REST API research and analysis | `f616763` - feat: initial SDK foundation with type-safe architecture | 2 days ago |
| [FULL_COVERAGE_PLAN.md](FULL_COVERAGE_PLAN.md) | Full coverage implementation plan (70-75% API surface) | `ec11c55` - docs: update FULL_COVERAGE_PLAN.md for Phase 7 completion | Today |
| [AUTHENTICATION_STRATEGY.md](AUTHENTICATION_STRATEGY.md) | Automated authentication strategy (API Key + Client Credentials) | New document | Today |

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
