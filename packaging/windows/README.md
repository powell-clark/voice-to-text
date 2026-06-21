# Windows packaging

The `.msi` installer is built by `cargo wix` using `wix/main.wxs` at the
project root (cargo-wix expects the `wix/` directory at the package root).

## CI release

GitHub Actions `release.yml` builds the MSI automatically on tag push and
attaches it to the GitHub release as `voice-to-text-installer.msi`.

## Local build

```bash
cargo build --release
cargo wix
```

Produces `target/wix/voice-to-text-*.msi`.

## Future

- Code signing (Authenticode) is tracked in the backlog.
