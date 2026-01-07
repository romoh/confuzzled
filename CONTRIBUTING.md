# Contributing to Confuzzled

Thank you for your interest in contributing to confuzzled!

## Prerequisites

- **Rust**: Install via [rustup](https://rustup.rs/) (1.70+ recommended)
- **OpenSSL**: Required for registry support
  - Ubuntu/Debian: `sudo apt install pkg-config libssl-dev`
  - Fedora/RHEL: `sudo dnf install openssl-devel`
  - macOS: `brew install openssl`
- **Docker** (optional): For testing generated images
- **skopeo** (optional): For copying OCI images to Docker

## Building

```bash
# Clone the repository
git clone https://github.com/user/confuzzled.git
cd confuzzled

# Build all crates
cargo build

# Build in release mode
cargo build --release

# Install CLI locally
cargo install --path crates/confuzzled_cli
```

## Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests for a specific crate
cargo test -p confuzzled_lib
cargo test -p confuzzled_cli

# Run a specific test
cargo test test_reference_parsing
```

## Development Workflow

### Adding a New Mutation

1. Choose the appropriate category in `crates/confuzzled_lib/src/mutations/`:
   - `structural.rs` - Layer/manifest structure changes
   - `format.rs` - Media type and format corruptions
   - `semantic.rs` - Valid structure but invalid semantics
   - `extreme.rs` - Edge cases and stress tests

2. Implement the `Mutation` trait:

```rust
use crate::mutations::Mutation;
use crate::oci::Image;
use crate::Result;

pub struct MyNewMutation;

impl Mutation for MyNewMutation {
    fn name(&self) -> &'static str {
        "my_new_mutation"
    }

    fn description(&self) -> &'static str {
        "Description of what this mutation does"
    }

    fn apply(&self, image: &mut Image) -> Result<()> {
        // Mutate the image
        Ok(())
    }
}
```

3. Register it in the category's `mutations()` function in `mod.rs`

4. Add tests for your mutation

### Running the CLI During Development

```bash
# Run with cargo
cargo run -- generate --seed minimal --count 5 --out ./test-corpus

# Run with verbose output
cargo run -- -v generate --seed busybox --mutations all --count 3 --out ./test-corpus

# List mutations
cargo run -- mutations --list
```

### Testing Generated Images

```bash
# Generate images
cargo run -- generate --seed "mcr.microsoft.com/azurelinux/base/core:3.0" --mutations all --count 5 --out ./corpus

# Test with skopeo/docker
skopeo copy oci:./corpus/fuzzed-0000 docker-daemon:test:latest
docker run --rm test:latest echo "Success!"
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Add doc comments to public APIs
- Include tests for new functionality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check for issues without building
cargo check
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b my-feature`
3. Make your changes with tests
4. Run `cargo fmt` and `cargo clippy`
5. Run `cargo test` to ensure all tests pass
6. Commit with descriptive messages
7. Push and open a Pull Request

## Reporting Issues

When reporting bugs, please include:
- Rust version (`rustc --version`)
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Any error messages

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
