# endurance-planner

A Rust-based terminal application for planning and tracking endurance training activities using the Ratatui TUI framework.

## Development Practices

### Test-Driven Development (TDD) - REQUIRED

This project follows **strict Test-Driven Development** using the **Red-Green-Refactor** cycle:

#### 🔴 RED → 🟢 GREEN → 🔵 REFACTOR

**Before writing any code:**
1. **🔴 RED**: Write a failing test first
2. **🟢 GREEN**: Write minimal code to make it pass
3. **🔵 REFACTOR**: Clean up while keeping tests green

#### Quick Example

```rust
// 1. RED: Write failing test
#[test]
fn test_calculate_maf_heart_rate() {
    assert_eq!(calculate_maf_heart_rate(30), 150);
}

// 2. GREEN: Minimal implementation
pub fn calculate_maf_heart_rate(age: u32) -> u32 {
    180 - age
}

// 3. REFACTOR: Improve if needed (keeping tests green)
```

#### TDD Rules

- ✅ **Always** write a failing test before production code
- ✅ Run `cargo test` after every change
- ✅ Commit after each successful Red-Green-Refactor cycle
- ❌ **Never** write production code without a test first

See [CLAUDE.md](CLAUDE.md) for comprehensive TDD guidelines and examples.

## Building and Testing

```bash
# Run tests (do this frequently!)
cargo test

# Build and run
cargo run

# Format code
cargo fmt

# Run linter
cargo clippy
```

## License

MIT