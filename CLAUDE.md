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

**Primary Language:** Rust
**UI Framework:** [Ratatui](https://ratatui.rs/) - Terminal User Interface library

#### Core Dependencies
- **ratatui** - TUI framework for building rich terminal interfaces
- **crossterm** - Cross-platform terminal manipulation (backend for ratatui)

#### Development Tools
- **rustc** - Rust compiler
- **cargo** - Rust package manager and build tool
- **rustfmt** - Code formatter
- **clippy** - Linting tool

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

**Rust-Specific Conventions:**
1. **Module Structure:**
   - Use `mod.rs` or single-file modules based on complexity
   - Keep modules focused and cohesive
   - Use `pub` visibility sparingly - prefer private by default

2. **Naming Conventions:**
   - `snake_case` for functions, variables, modules, and fields
   - `PascalCase` for types, traits, and enums
   - `SCREAMING_SNAKE_CASE` for constants and statics
   - Prefix predicates with `is_`, `has_`, `can_`, etc.

3. **Error Handling:**
   - Use `Result<T, E>` for recoverable errors
   - Use `Option<T>` for nullable values
   - Prefer `?` operator over explicit match/unwrap
   - Create custom error types for domain-specific errors
   - Only use `unwrap()` or `expect()` when certain value exists

4. **Code Style:**
   - Run `cargo fmt` before committing
   - Run `cargo clippy` and address warnings
   - Follow Rust API Guidelines: https://rust-lang.github.io/api-guidelines/

5. **TUI-Specific Patterns:**
   - Separate UI rendering from business logic
   - Use the Component pattern for reusable UI elements
   - Keep state management separate from rendering
   - Handle terminal events in dedicated event loop

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

**Rust Testing Patterns:**

1. **Unit Tests:**
   - Place tests in the same file within `#[cfg(test)]` module
   - Use `#[test]` attribute for test functions
   - Run with `cargo test`
   - Example:
     ```rust
     #[cfg(test)]
     mod tests {
         use super::*;

         #[test]
         fn test_function_name() {
             // Test implementation
         }
     }
     ```

2. **Integration Tests:**
   - Place in `tests/` directory at project root
   - Each file is compiled as separate crate
   - Test public API only

3. **Documentation Tests:**
   - Include examples in doc comments
   - Automatically tested with `cargo test`
   - Use `///` for function docs with examples

4. **Test Organization:**
   - Use `assert!`, `assert_eq!`, `assert_ne!` for assertions
   - Use `#[should_panic]` for expected panic tests
   - Use `Result<(), E>` for tests that can return errors

5. **Running Tests:**
   - `cargo test` - Run all tests
   - `cargo test --lib` - Run only library tests
   - `cargo test test_name` - Run specific test
   - `cargo test -- --nocapture` - Show println! output

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

### Standard Rust Project Layout

```
endurance-planner/
├── Cargo.toml              # Project manifest and dependencies
├── Cargo.lock              # Dependency lock file (committed to git)
├── README.md               # Project description
├── LICENSE                 # MIT License
├── CLAUDE.md              # This file (AI assistant documentation)
├── .gitignore             # Git ignore rules
├── src/
│   ├── main.rs            # Application entry point and event handling
│   ├── app.rs             # Application state, screens, and logic
│   ├── ui.rs              # Ratatui UI rendering for all screens
│   ├── models.rs          # Data models (Distance, RaceType, Workout, etc.)
│   └── file_io.rs         # Markdown save/load functionality
└── target/                # Build artifacts (not committed to git)
```

### Key Files

- **Cargo.toml**: Project manifest with ratatui and crossterm dependencies
- **src/main.rs**: Entry point, terminal setup, and keyboard input handlers
- **src/app.rs**: Application state, screen navigation, plan generation
- **src/ui.rs**: Ratatui rendering for all screens (welcome, inputs, plan view, save/load/edit)
- **src/models.rs**: Domain models - Distance, RaceType, RPE, HeartRateZones, Workout, TrainingPlan
- **src/file_io.rs**: Markdown serialization/deserialization for training plans

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

**Maffetone Method (MAF)**
- Maximum Aerobic Function heart rate = 180 - age
- Zone 1 (Recovery): MAF-20 to MAF-10 bpm
- Zone 2 (Aerobic Base): MAF-10 to MAF bpm
- Used for all easy/long runs to build aerobic base

**RPE (Rate of Perceived Exertion)**
- Scale of 1-10 for subjective effort
- Used for interval and tempo workouts
- RPE 6: Tempo pace (comfortably hard)
- RPE 7-8: Interval pace (hard to very hard)

**Periodization Phases**
- Base (40%): Aerobic foundation, easy runs
- Build (30%): Introduce intervals and tempo
- Peak (20%): Quality workouts, race-specific
- Taper (10%): Reduced volume, maintain intensity

### Data Models

- **Distance**: 5K, 10K, Half Marathon, Marathon, 50K, 100K, 100 Miles
- **RaceType**: Road, Trail (affects workout selection)
- **WorkoutType**: EasyRun, LongRun, RecoveryRun, Intervals, TempoRun, HillRepeats, TechnicalTrail, VerticalTraining, Rest
- **TrainingPlan**: Contains UserProfile, HeartRateZones, and weekly schedule

### Application Features

**Keyboard Shortcuts**
- Welcome: `Enter` (new plan), `l` (load), `q` (quit)
- Plan View: `Up/Down` (weeks), `Left/Right` (workouts), `e` (edit), `s` (save), `l` (load), `q` (quit)
- Input Screens: `Enter` (confirm), `Esc` (back), `Up/Down` (select)

**File Operations**
- Plans saved as human-readable Markdown files
- Can be edited externally and reloaded
- Workout descriptions editable in-app

## Common Cargo Commands

### Building and Running
- `cargo build` - Compile the project (debug mode)
- `cargo build --release` - Compile with optimizations
- `cargo run` - Build and run the application
- `cargo run --release` - Build and run optimized version
- `cargo check` - Check code for errors without building

### Testing and Quality
- `cargo test` - Run all tests
- `cargo clippy` - Run linting checks
- `cargo fmt` - Format code according to Rust style
- `cargo fmt -- --check` - Check formatting without modifying files

### Dependencies
- `cargo add <crate>` - Add a dependency (requires cargo-edit)
- `cargo update` - Update dependencies within semver constraints
- `cargo tree` - Display dependency tree

### Documentation
- `cargo doc --open` - Generate and open documentation
- `cargo doc --no-deps` - Generate docs without dependencies

### Cleaning
- `cargo clean` - Remove build artifacts

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

**Rust:**
- The Rust Book: https://doc.rust-lang.org/book/
- Rust by Example: https://doc.rust-lang.org/rust-by-example/
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/
- Rust Standard Library: https://doc.rust-lang.org/std/

**Ratatui:**
- Ratatui Documentation: https://ratatui.rs/
- Ratatui GitHub: https://github.com/ratatui-org/ratatui
- Ratatui Examples: https://github.com/ratatui-org/ratatui/tree/main/examples
- Crossterm Documentation: https://docs.rs/crossterm/latest/crossterm/

**Development Tools:**
- Cargo Book: https://doc.rust-lang.org/cargo/
- Clippy Lints: https://rust-lang.github.io/rust-clippy/

**General:**
- Git Documentation: https://git-scm.com/doc
- Markdown Guide: https://www.markdownguide.org/

### Internal Documentation
*Links to be added as documentation grows*

## Ratatui TUI Development Patterns

### Terminal Setup and Teardown

Always ensure proper terminal cleanup:
```rust
// Initialize terminal
let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
terminal.clear()?;

// Enable raw mode
crossterm::terminal::enable_raw_mode()?;

// ... run application ...

// Always restore terminal state
crossterm::terminal::disable_raw_mode()?;
terminal.show_cursor()?;
```

### Event Loop Pattern

Standard event loop structure:
```rust
loop {
    // Render UI
    terminal.draw(|frame| ui::render(frame, &app))?;

    // Handle events
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                // Handle other keys
                _ => {}
            }
        }
    }

    // Update application state
    app.update();
}
```

### Component Pattern

Structure UI components as composable functions:
```rust
pub fn render_component(frame: &mut Frame, area: Rect, state: &ComponentState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Component Title");

    frame.render_widget(block, area);
}
```

### State Management

- Keep UI state separate from business logic
- Use message-passing or event-driven patterns for state updates
- Implement clear state transition functions

### Layout Best Practices

- Use `Layout` for responsive terminal layouts
- Calculate sizes based on available space
- Handle terminal resize events gracefully

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
- Added Rust + Ratatui technology stack details
- Documented Rust-specific code conventions and patterns
- Added Cargo commands reference
- Included Ratatui TUI development patterns
- Added comprehensive resource links for Rust and Ratatui

---

**Note:** This document is a living guide. Update it as the project evolves and new patterns emerge.
