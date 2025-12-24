# Planning Document Index

This index tracks all planning and documentation files in the repository
along with their last associated commit.

## Planning Documents

| Document | Purpose | Last Commit | Date |
|----------|---------|-------------|------|
| [ARCGIS_REST_API_RESEARCH.md](ARCGIS_REST_API_RESEARCH.md) | ArcGIS REST API research and analysis | `f616763` - feat: initial SDK foundation with type-safe architecture | 2 days ago |
| [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) | SDK implementation roadmap and milestones | `f616763` - feat: initial SDK foundation with type-safe architecture | 2 days ago |
| [ARCHITECTURE_DECISION.md](ARCHITECTURE_DECISION.md) | Architectural decisions and rationale | `f616763` - feat: initial SDK foundation with type-safe architecture | 2 days ago |
| [USER_EXPERIENCE_COMPARISON.md](USER_EXPERIENCE_COMPARISON.md) | UX comparison and design decisions | `f616763` - feat: initial SDK foundation with type-safe architecture | 2 days ago |
| [PROJECT_STATUS.md](PROJECT_STATUS.md) | Current project status and progress tracking | `47d2728` - docs: update repository URLs to actual GitHub location | 2 days ago |

## Compliance & Standards

| Document | Purpose | Last Commit | Date |
|----------|---------|-------------|------|
| [CLAUDE_MD_COMPLIANCE.md](CLAUDE_MD_COMPLIANCE.md) | CLAUDE.md compliance checklist and status | `4bbda7b` - refactor: bring codebase to CLAUDE.md compliance standards | 2 days ago |
| [CLAUDE_COMPLIANCE_REVIEW.md](CLAUDE_COMPLIANCE_REVIEW.md) | Detailed compliance review findings | `a81b781` - docs: add CLAUDE.md compliance review | 31 hours ago |
| [CLAUDE.md](CLAUDE.md) | Project instructions and coding standards | `babb719` - refactor(error): implement exceptional error handling with type preservation | 26 hours ago |

## Repository Documentation

| Document | Purpose | Last Commit | Date |
|----------|---------|-------------|------|
| [README.md](README.md) | Main repository documentation | `4bbda7b` - refactor: bring codebase to CLAUDE.md compliance standards | 2 days ago |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contribution guidelines | `f616763` - feat: initial SDK foundation with type-safe architecture | 2 days ago |

## Usage

When creating new planning documents:

1. Add the document to this index
2. Categorize it appropriately (Planning, Compliance, Documentation)
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
