# CLAUDE.md - AI Assistant Guide for Endurance Planner

> **Last Updated:** 2026-01-01
> **Repository:** endurance-planner
> **License:** MIT
> **Owner:** Lauri Hahne

## Project Overview

Endurance Planner is a project for planning and tracking endurance training activities. The repository is currently in its initial state with core infrastructure being established.

### Project Purpose
This project aims to provide tools and utilities for endurance athletes to plan, track, and analyze their training programs.

## Current Repository State

**Status:** Initial setup phase
**Branch:** `claude/add-claude-documentation-eBcgp`

### Existing Files
- `README.md` - Basic project description
- `LICENSE` - MIT License
- `CLAUDE.md` - This file (AI assistant documentation)

### Technology Stack
*To be determined - Stack will be established as development progresses*

## Development Conventions

### Code Style & Standards

#### General Principles
1. **Simplicity First**: Don't over-engineer solutions. Implement what's needed, not what might be needed.
2. **Security Conscious**: Always consider OWASP Top 10 vulnerabilities:
   - Prevent SQL injection
   - Prevent XSS attacks
   - Prevent command injection
   - Validate input at system boundaries
   - Don't trust external data
3. **Clean Code**: Write self-documenting code. Add comments only when logic isn't self-evident.
4. **Minimal Abstractions**: Don't create helpers, utilities, or abstractions for one-time operations.

#### Code Organization
*To be established as codebase grows*

### Git Workflow

#### Branch Naming Convention
- Feature branches: `claude/feature-description-[session-id]`
- All development branches must start with `claude/` and end with the matching session ID
- Example: `claude/add-claude-documentation-eBcgp`

#### Commit Message Guidelines
1. Use clear, descriptive messages that focus on "why" rather than "what"
2. Follow existing repository commit style
3. Use conventional commit format when appropriate:
   - `feat:` - New feature
   - `fix:` - Bug fix
   - `docs:` - Documentation changes
   - `refactor:` - Code refactoring
   - `test:` - Test additions/changes
   - `chore:` - Maintenance tasks

#### Git Operations Best Practices
1. **Push Commands:**
   - Always use: `git push -u origin <branch-name>`
   - Branch must start with `claude/` and end with session ID
   - Retry up to 4 times with exponential backoff (2s, 4s, 8s, 16s) on network errors

2. **Fetch/Pull Commands:**
   - Prefer specific branches: `git fetch origin <branch-name>`
   - Retry up to 4 times with exponential backoff on network failures

3. **Commit Workflow:**
   - Review `git status` before committing
   - Review `git diff` to understand changes
   - Check recent commits with `git log` to match style
   - Never skip hooks without explicit user request
   - Never force push to main/master

### File Operations

#### Preferred Tools
- **Reading files:** Use `Read` tool (not `cat`/`head`/`tail`)
- **Editing files:** Use `Edit` tool (not `sed`/`awk`)
- **Writing files:** Use `Write` tool (not `echo >` or heredocs)
- **Searching files:** Use `Glob` tool (not `find`/`ls`)
- **Searching content:** Use `Grep` tool (not `grep`/`rg`)

#### File Creation Policy
- **ALWAYS** prefer editing existing files over creating new ones
- Only create new files when absolutely necessary
- Never proactively create documentation files unless explicitly requested
- Avoid creating README files, CHANGELOG files, or other meta-documentation without request

### Testing Standards

*To be established as testing framework is implemented*

### Error Handling

1. **Validation Points:**
   - Validate at system boundaries (user input, external APIs)
   - Trust internal code and framework guarantees
   - Don't add error handling for scenarios that can't happen

2. **Error Messages:**
   - Provide clear, actionable error messages
   - Include context for debugging
   - Don't expose sensitive information in errors

## Project Structure

*This section will be updated as the project structure develops*

### Planned Directories
```
endurance-planner/
├── README.md
├── LICENSE
├── CLAUDE.md
└── [Additional structure to be determined]
```

## Development Workflow for AI Assistants

### Initial Setup Tasks
When starting work on this project:
1. Read this CLAUDE.md file thoroughly
2. Check current branch with `git status`
3. Review recent commits with `git log`
4. Understand the task requirements before making changes

### Before Making Changes
1. **Research First:** Always read relevant files before proposing changes
2. **Plan Complex Tasks:** Use TodoWrite tool for multi-step tasks
3. **Understand Context:** Search and understand existing patterns
4. **Avoid Assumptions:** Never guess about file contents or structure

### Making Changes
1. **Read Before Edit:** Always use Read tool before editing any file
2. **Preserve Style:** Match existing code style and conventions
3. **Test Changes:** Verify changes work as expected
4. **Track Progress:** Use TodoWrite for complex tasks

### After Making Changes
1. **Review Changes:** Use `git diff` to review all modifications
2. **Commit Properly:** Follow commit message guidelines
3. **Push to Correct Branch:** Ensure branch name follows convention
4. **Update Documentation:** Update this file if workflows change

## Domain-Specific Knowledge

### Endurance Training Concepts
*To be documented as domain knowledge is implemented*

Potential areas to cover:
- Training zones and intensity levels
- Periodization and training phases
- Volume and intensity metrics
- Recovery and adaptation
- Performance testing and benchmarking

### Data Models
*To be documented as data structures are defined*

### Business Logic
*To be documented as core functionality is implemented*

## Common Tasks & Patterns

### Adding a New Feature
1. Understand the feature requirements
2. Research existing similar functionality
3. Plan the implementation (use TodoWrite for complex features)
4. Implement following conventions
5. Test the feature
6. Commit with clear message
7. Push to feature branch

### Fixing a Bug
1. Reproduce the issue
2. Identify root cause
3. Implement minimal fix
4. Verify fix resolves the issue
5. Commit with "fix:" prefix
6. Push to feature branch

### Refactoring Code
1. Only refactor when explicitly requested
2. Don't refactor surrounding code during bug fixes
3. Maintain existing functionality
4. Test thoroughly
5. Commit with "refactor:" prefix

## Security Considerations

### Input Validation
- Validate all user input
- Sanitize data before storage
- Escape data before output
- Use parameterized queries for databases

### Authentication & Authorization
*To be documented when implemented*

### Data Protection
*To be documented based on data sensitivity*

## Performance Guidelines

*To be established as performance requirements are defined*

## Debugging Tips

### When Things Go Wrong
1. Check git status and current branch
2. Review recent changes with git diff
3. Check error messages and logs
4. Verify file paths and permissions
5. Test in isolation to identify issue

### Common Issues
*To be documented as patterns emerge*

## Resources & References

### External Documentation
- Git Documentation: https://git-scm.com/doc
- Markdown Guide: https://www.markdownguide.org/

### Internal Documentation
*Links to be added as documentation grows*

## Notes for AI Assistants

### Communication Style
- Be concise and direct
- Focus on technical accuracy
- Avoid emojis unless requested
- Use markdown for formatting
- Output text directly (never use bash echo for communication)

### Tool Usage
- Use Task tool with subagent_type=Explore for codebase exploration
- Use specialized tools over bash commands when available
- Run independent commands in parallel when possible
- Never guess or use placeholders in tool parameters

### Best Practices
1. **Read First:** Never modify files without reading them first
2. **Simple Solutions:** Avoid over-engineering
3. **Security First:** Always consider security implications
4. **Test Thoroughly:** Verify changes work correctly
5. **Track Work:** Use TodoWrite for complex tasks
6. **Update Docs:** Keep this file current

### Anti-Patterns to Avoid
- ❌ Creating files unnecessarily
- ❌ Adding features not requested
- ❌ Over-abstracting simple code
- ❌ Adding error handling for impossible scenarios
- ❌ Refactoring unrelated code
- ❌ Using backwards-compatibility hacks
- ❌ Adding comments to unchanged code
- ❌ Using bash for file operations

## Changelog

### 2026-01-01
- Initial CLAUDE.md creation
- Established basic conventions and workflows
- Documented repository initial state

---

**Note:** This document is a living guide. Update it as the project evolves and new patterns emerge.
