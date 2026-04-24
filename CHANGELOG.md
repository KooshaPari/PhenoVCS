# Changelog

All notable changes to this project will be documented in this file.

## 📚 Documentation
- Docs(fr): scaffold FUNCTIONAL_REQUIREMENTS.md

Add FR traceability framework to enable test+spec integration. Wave-5 systemic push.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com> (`21d9687`)
- Docs(readme): expand README.md with purpose, stack, quick-start, related projects (`e34d239`)
- Docs: add PLAN.md (`adf3eaf`)
- Docs: add README.md (`5e8ec89`)
- Docs: add SPEC.md (`b4032a6`)
## 🔨 Other
- Chore(ci): adopt phenotype-tooling workflows (wave-2) (`3e8bda7`)
- Chore(governance): adopt standard CLAUDE.md + AGENTS.md + worklog (wave-2) (`9d3c33a`)
- Ci(legacy-enforcement): add legacy tooling anti-pattern gate (WARN mode)

Adds legacy-tooling-gate.yml monitoring per CLAUDE.md Technology Adoption Philosophy.

Refs: phenotype/repos/tooling/legacy-enforcement/ (`ea667c1`)
- Ci: migrate to reusable workflows from template-commons

- Use reusable-rust-ci.yml, reusable-python-ci.yml, reusable-typescript-ci.yml
- Add security scanning with reusable-security-scan.yml
- Add governance validation with validate-governance.yml (`831c588`)
- Initial: PhenoVCS version control registry (`3d17d03`)