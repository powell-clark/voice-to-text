# Contributing to Voice to Text

Thank you for contributing! This guide will help you get started.

## Development Workflow

### 1. Fork and Clone
```bash
git clone https://github.com/powell-clark/voice-to-text.git
cd voice-to-text
```

### 2. Create a Branch
```bash
git checkout -b feature/your-feature-name
```

### 3. Make Changes

Follow the project coding style:
- **Commits**: Use conventional commits (`feat:`, `fix:`, `chore:`, `docs:`)
- **C code**: Follow Linux kernel style (indent with tabs)
- **Python**: Follow PEP 8

### 4. Test Your Changes

```bash
# Linux
make -f Makefile.linux
./vtt-linux

# macOS
make complete
open VTT.app

# Run tests
cd tests && make test
```

### 5. Submit PR

Push your branch and create a pull request. CI will automatically:
- Build for Linux and macOS
- Run unit tests
- Check code quality
- Verify packaging

## Project Structure

```
src/
├── common/       # Cross-platform code
├── linux/        # Linux implementation
└── macos/        # macOS implementation

tests/            # Unit tests
debian/           # Debian packaging
Casks/            # Homebrew cask formulas
```

## Testing

All PRs must pass CI checks:
- ✅ Linux build
- ✅ macOS build
- ✅ Unit tests
- ✅ Code linting
- ✅ No security issues

Run tests locally:
```bash
cd tests
make test
```

## Commit Messages

Use conventional commit format:

```
feat: add custom hotkey support
fix: resolve UTF-8 typing bug
chore: update dependencies
docs: improve installation guide
```

## Code Review

Maintainers will review your PR. Please:
- Respond to feedback promptly
- Keep PRs focused and atomic
- Add tests for new features
- Update documentation

## Getting Help

- **Issues**: https://github.com/powell-clark/voice-to-text/issues
- **Discussions**: GitHub Discussions

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.
