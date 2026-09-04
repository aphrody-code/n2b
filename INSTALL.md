# Installing n2b

`n2b` is distributed as one executable. It does not require Bun at runtime for
ordinary scans; commands which invoke Bun still require `bun` on `PATH`.

## Aphrody package manager

The future package manager can use the declarative
`[workspace.metadata.aphrody.package]` table in `Cargo.toml`. It identifies the
repository, Cargo package, installed binaries, version probe, and targets that
have release artifacts. The table deliberately contains no machine-specific
installation prefix.

```console
aphrody package install n2b
aphrody package update n2b
aphrody package uninstall n2b
```

The manager owns the installation receipt and prefix. It should install to a
user-writable bin directory by default, atomically replace the executable on
update, and remove only files recorded in that receipt on uninstall.

## Release archives

Download the archive and matching `.sha256` file for your target from the
GitHub release. Verify the checksum before extracting it, then place `n2b` (or
`n2b.exe`) in a directory on `PATH`.

Release artifacts currently cover:

- Linux x86-64 (`x86_64-unknown-linux-gnu` and static musl fallback)
- macOS Apple Silicon (`aarch64-apple-darwin`)
- Windows x86-64 (`x86_64-pc-windows-msvc`)

Other Rust-supported platforms can use the source installation below.

## Locked source installation

Rust 1.85 or newer is required. A tag makes the selected version explicit and
`--locked` requires the repository's reviewed `Cargo.lock` dependency graph.

```console
cargo install --locked --git https://github.com/aphrody-code/n2b --tag v0.6.1 --package n2b
n2b --version
```

To update, run the same command with the desired newer tag. Cargo replaces its
managed executable after a successful build:

```console
cargo install --locked --force --git https://github.com/aphrody-code/n2b --tag v0.6.1 --package n2b
```

To uninstall a Cargo-managed copy:

```console
cargo uninstall n2b
```

Do not use `cargo uninstall` for an executable installed from a release archive;
remove that executable through the package manager which recorded it, or remove
the single manually installed `n2b`/`n2b.exe` file from its chosen prefix.

## Platform notes

- Linux and macOS: the usual user-local destination is `$HOME/.local/bin`.
- Windows: use a user-owned directory on `%PATH%`; the binary is `n2b.exe`.
- System-wide prefixes may require administrator privileges. They are not
  required and should not be the default for automated package management.
