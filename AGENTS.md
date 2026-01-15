# AGENTS.md — Agent Operational Contract for midnite-archive (v2)

## 1. Purpose & Scope

This document defines the **operational contract** for AI agents interacting with the
`midnite-archive` repository.

Its purpose is to:
- Ensure safe, predictable, and high-quality agent behavior
- Minimize ambiguity in decision-making
- Protect repository integrity, security, and maintainability

All agents MUST comply with this document.

---

## 2. Rule Priority Hierarchy

All rules follow a strict priority order. Higher-priority rules override lower-priority ones.

### Priority 1 — Core Agent Rules (Highest)
- This AGENTS.md has absolute authority
- Operational modes (Plan vs Build) MUST be respected
- Explicit user commands override defaults only if compliant with higher-priority rules

### Priority 2 — Authorization & Safety
- Commit authorization via `@do`
- Git safety rules and branch protection
- System and security constraints

### Priority 3 — Agent Operations
- Special command behavior
- Tool usage restrictions
- Communication standards

### Priority 4 — Development Standards
- Code style and naming conventions
- Documentation and testing practices

### Priority 5 — Recommendations (Lowest)
- Compatibility and optimization suggestions

---

## 3. Agent Operational Contract

By interacting with this repository, the agent agrees to:

- Follow the rule priority hierarchy
- Operate strictly within the active mode
- Ask for clarification when intent is ambiguous
- Prefer safety and correctness over speed or creativity

---

## 4. Operational Modes

### 4.1 Plan Mode

**Purpose:** Analysis, design, and proposal.

**Allowed:**
- Read files
- Analyze code and architecture
- Propose changes and plans
- Ask clarifying questions

**Forbidden:**
- Modifying files
- Running destructive commands
- Committing changes

---

### 4.2 Build Mode

**Purpose:** Implementation and execution.

**Allowed:**
- Modify files
- Run tests, linters, and build commands
- Commit changes ONLY with explicit `@do` authorization

**Forbidden:**
- Committing without `@do`
- Modifying protected branches (e.g. `main`)
- Bypassing safety mechanisms

---

## 5. Hard Prohibitions (Non-Negotiable)

Agents MUST NOT:

- Commit changes without `@do`
- Modify files without explicit user permission
- Commit directly to protected branches
- Bypass security checks or safeguards
- Generate malicious or intentionally unsafe code

---

## 6. Restricted Actions (Require Explicit Permission)

The following actions require explicit user approval:

- Installing system packages
- Modifying system configuration
- Changing git configuration, hooks, or repository settings
- Running destructive git commands (force push, hard reset)

---

## 7. Tool Usage Rules

- Use specialized tools when available (Read, Edit, etc.)
- Avoid bash for file manipulation when safer tools exist
- Do not misuse tools outside their intended purpose

---

## 8. Communication Standards

Agents MUST:

- Be clear, concise, and neutral
- Refuse unsafe requests calmly and respectfully
- Ask before making assumptions

Agents MUST NOT:

- Be condescending, preachy, or judgmental
- Add unnecessary meta-commentary
- Use emojis unless explicitly requested

---

## 9. Special Agent Commands

- `@refresh` — Reload and acknowledge updated AGENTS.md rules
- `@help commit` — Show commit examples
- `@do` — Authorize commits (user-only, Build Mode only)

---

## 10. Project-Specific Context (midnite-archive)

### Build / Lint / Test

- Build: `just run <youtube-channel>`
- Lint: `find scripts -name "*.sh" -exec shellcheck {} \;`
- Test: `./scripts/yt/channel_list_generate.sh @testchannel`

---

## 11. Development Standards (Shell Scripts)

### General Rules
- Use `set -euo pipefail`
- Sanitize all user inputs
- Avoid `eval` and unsafe constructs

### Naming Conventions

**Files & Directories**
- `snake_case` only

**Variables**
- Globals/constants: `UPPERCASE_WITH_UNDERSCORES`
- Locals: `lowercase_with_underscores`

**Functions**
- `lowercase_with_underscores`
- Start with action verbs
- Small and focused

---

## 12. Formatting & Style

- 2 spaces indentation
- Max line length: 100 characters
- Always quote variables
- Break long commands for readability

---

## 13. Error Handling

- Validate inputs early
- Provide clear error messages
- Exit with appropriate codes

---

## 14. Security Practices

- Use arrays for command arguments
- Prefer absolute paths
- Clean up temporary files with traps

---

## 15. Development Workflow

1. Operate in Plan Mode
2. Create feature branch
3. Implement changes in Build Mode
4. Run lint and tests
5. Commit only with `@do`

---

## 16. Pre-Commit Checklist

- Pre-commit hooks pass
- Linting issues resolved
- Documentation renders correctly
- Commands tested in clean environment
