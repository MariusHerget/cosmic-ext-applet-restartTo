# Cosmic Ext Applet Restart To

Applet that adds capability to restart to any other EFI boot entry

## Installation

A [justfile](./justfile) is included by default for the [casey/just][just] command runner.

- `just` builds the application with the default `just build-release` recipe
- `just run` builds and runs the application
- `just install` installs the project into the system
- `just vendor` creates a vendored tarball
- `just build-vendored` compiles with vendored dependencies from that tarball
- `just check` runs clippy on the project to check for linter warnings
- `just check-json` can be used by IDEs that support LSP

## Translators

[Fluent][fluent] is used for localization of the software. Fluent's translation files are found in the [i18n directory](./i18n). New translations may copy the [English (en) localization](./i18n/en) of the project, rename `en` to the desired [ISO 639-1 language code][iso-codes], and then translations can be provided for each [message identifier][fluent-guide]. If no translation is necessary, the message may be omitted.

## Packaging

If packaging for a Linux distribution, vendor dependencies locally with the `vendor` rule, and build with the vendored sources using the `build-vendored` rule. When installing files, use the `rootdir` and `prefix` variables to change installation paths.

```sh
just vendor
just build-vendored
just rootdir=debian/cosmic-ext-applet-restart-to prefix=/usr install
```

It is recommended to build a source tarball with the vendored dependencies, which can typically be done by running `just vendor` on the host system before it enters the build environment.

## Development

### Prerequisites

Before developing, ensure you have the following installed:

1. **Rust toolchain**: Install [rustup][rustup] and ensure you have a recent stable Rust version:
   ```sh
   rustup install stable
   rustup default stable
   ```

2. **Build dependencies**: On Pop!_OS/Ubuntu/Debian:
   ```sh
   sudo apt install cargo cmake just libexpat1-dev libfontconfig-dev libfreetype-dev libxkbcommon-dev pkgconf
   ```

3. **just command runner**: Install [just][just] for running build commands:
   ```sh
   # On Pop!_OS/Ubuntu/Debian
   sudo apt install just
   # Or via cargo
   cargo install just
   ```

4. **Editor setup**: Configure your editor to use [rust-analyzer][rust-analyzer] for IDE support.

5. **EFI system access**: The applet requires access to EFI variables. Ensure:
   - You're running on a UEFI system (not legacy BIOS)
   - `/sys/firmware/efi/efivars` is mounted (usually automatic)
   - `efibootmgr` is installed: `sudo apt install efibootmgr`
   - You have appropriate permissions (polkit policy is installed automatically with `just install`)

### Building

Build the applet in release mode:
```sh
just build-release
```

Or build in debug mode for development:
```sh
just build-debug
```

### Running Locally

To test the applet without installing it system-wide:

```sh
just run
```

This will compile and run the applet. The applet will appear in your COSMIC panel.

**Note**: When running locally, the applet may not appear automatically. You may need to:
1. Log out and log back in, or
2. Restart the COSMIC panel/session

### Installing for Testing

To install the applet system-wide for testing:

```sh
just install
```

This installs the applet to `/usr/bin` and registers it with the desktop environment. After installation:
1. Log out and log back in, or
2. Restart COSMIC session
3. Add the applet via COSMIC Settings → Panel → Applets

To uninstall:
```sh
just uninstall
```

### Development Workflow

1. **Make changes** to the source code in `src/`

2. **Check for linting issues**:
   ```sh
   just check
   ```

3. **Build and test**:
   ```sh
   just run
   ```

4. **Test the applet**:
   - Click the reboot icon in the panel
   - Verify boot entries are listed correctly
   - Test the confirmation dialog (but don't actually reboot during development!)

### Testing EFI Boot Entry Functionality

⚠️ **Warning**: Testing boot entry selection will actually reboot your system. Use caution!

#### Safe Testing (Without Rebooting)

1. **List boot entries** (read-only, safe):
   ```sh
   # Using efibootmgr (if installed)
   sudo efibootmgr -v
   
   # Or test via the applet UI - just don't click "Restart"!
   ```

2. **Verify EFI access**:
   ```sh
   # Check if EFI variables are accessible
   ls -la /sys/firmware/efi/efivars/ | head
   ```

3. **Test error handling**:
   - Run without root permissions to test permission errors
   - Test on a non-UEFI system (if available) to test EFI detection

#### Testing Boot Entry Selection (Will Reboot!)

If you need to test the actual reboot functionality:

1. **Ensure you have a way to recover**:
   - Have a recovery USB drive ready
   - Know how to access UEFI/BIOS settings
   - Have backups of important data

2. **Test with a safe boot entry**:
   - Select a boot entry you know works (like your current OS)
   - Verify the confirmation dialog appears
   - Confirm and verify the system reboots to the selected entry

3. **Monitor logs** (before rebooting):
   ```sh
   # Watch system logs for EFI variable changes
   sudo journalctl -f | grep -i efi
   ```

### Debugging

#### Logging to journalctl

The applet logs all events, errors, and debug information to `journalctl`. Logs are automatically captured by systemd when running as a service, or can be viewed manually.

**View applet logs:**
```sh
# View recent logs
journalctl -u cosmic-panel -n 100 | grep restart-to

# Follow logs in real-time
journalctl -u cosmic-panel -f | grep restart-to

# View logs with timestamps
journalctl -u cosmic-panel --since "1 hour ago" | grep restart-to

# View only errors
journalctl -u cosmic-panel -p err | grep restart-to
```

**When running manually:**
```sh
# Logs go to stderr, which can be captured
RUST_LOG=debug just run 2>&1 | tee applet.log

# Or view in journalctl if running as user service
journalctl --user -f | grep restart-to
```

#### Enable Debug Logging

Set the `RUST_LOG` environment variable to control log levels:
```sh
# Debug level (most verbose)
RUST_LOG=debug just run

# Info level (default)
RUST_LOG=info just run

# Warning and errors only
RUST_LOG=warn just run

# Errors only
RUST_LOG=error just run

# With backtraces
RUST_BACKTRACE=full RUST_LOG=debug just run
```

**Log levels used:**
- `error!` - Critical errors (permission denied, EFI access failures, reboot failures)
- `warn!` - Warnings (config errors, skipped boot entries)
- `info!` - Important events (applet start, boot entry selection, reboot initiation)
- `debug!` - Detailed debugging (popup open/close, config updates, D-Bus connections)

#### Common Issues

1. **Applet doesn't appear in panel**:
   - Ensure the applet is installed: `just install`
   - Restart COSMIC session or log out/in
   - Check desktop file: `cat /usr/share/applications/com.github.cosmic_ext.restartTo.desktop`

2. **No boot entries found**:
   - Verify you're on a UEFI system: `[ -d /sys/firmware/efi ] && echo "UEFI" || echo "Legacy BIOS"`
   - Check EFI variables access: `ls /sys/firmware/efi/efivars/`
   - May need root permissions or polkit policies

3. **Permission denied errors**:
   - **Error message**: "Permission denied: EFI variable access requires elevated privileges"
   - **Cause**: Setting BootNext EFI variable requires elevated privileges
   - **Solutions**:
     - **Option 1 (Recommended)**: Ensure polkit policy is installed (`just install` installs it automatically)
     - **Option 2**: Verify polkit service is running: `systemctl status polkit`
     - **Option 3**: Ensure you have a polkit authentication agent (usually provided by your desktop environment)
     - **Option 4**: Test with `sudo efibootmgr -n <entry>` to verify EFI access works
   - The applet will automatically request polkit authorization and prompt for your password when needed

4. **Build errors**:
   - Ensure all dependencies are installed
   - Try `cargo clean && just build-release`
   - Check Rust version: `rustc --version` (should be 1.70+)

### Permissions

The applet needs access to EFI variables, which typically requires elevated privileges. The applet now uses **polkit** to request authorization interactively, allowing users to authenticate when needed without running the entire applet as root.

#### Polkit Integration

The applet uses `pkexec` to run `efibootmgr` with elevated privileges. When you select a boot entry and confirm the reboot:

1. **Polkit authorization prompt**: `pkexec` will prompt you to authenticate (enter your password) via your system's authentication agent
2. **Authorization check**: Polkit verifies authorization using the installed policy
3. **EFI variable access**: If authorized, `efibootmgr` sets the BootNext EFI variable with elevated privileges

**Runtime dependency**: The applet requires `efibootmgr` to be installed:
```bash
sudo apt install efibootmgr
```

#### Installing the Polkit Policy

The polkit policy file is automatically installed when you run `just install`. The policy file is located at:
```
/usr/share/polkit-1/actions/com.github.cosmic_ext.restartTo.policy
```

**Manual installation** (if needed):
```bash
sudo cp resources/com.github.cosmic_ext.restartTo.policy /usr/share/polkit-1/actions/
```

**Verify polkit service is running**:
```bash
systemctl status polkit
```

#### Troubleshooting Permission Issues

If you encounter permission errors:

1. **Verify polkit policy is installed**:
   ```bash
   ls -la /usr/share/polkit-1/actions/com.github.cosmic_ext.restartTo.policy
   ```

2. **Check polkit service**:
   ```bash
   systemctl status polkit
   ```

3. **Verify authentication agent**: Ensure you have a polkit authentication agent running (most desktop environments provide this automatically)

4. **Ensure efibootmgr is installed**:
   ```bash
   # Check if efibootmgr is installed
   which efibootmgr
   
   # Install if missing
   sudo apt install efibootmgr
   ```

5. **Test EFI access manually**:
   ```bash
   # List boot entries
   sudo efibootmgr -v
   
   # Test setting BootNext (replace 0001 with your boot entry ID)
   sudo efibootmgr -n 0001
   ```

6. **Check logs**: View applet logs to see detailed error messages:
   ```bash
   journalctl -u cosmic-panel -f | grep restart-to
   ```

#### Fallback Behavior

If polkit authorization fails or is unavailable, the applet will still attempt direct EFI access. This may work if:
- Your user has direct EFI variable access (uncommon)
- You're running the applet with elevated privileges (not recommended)
- Your system has group-based EFI access configured

### Code Quality

Run clippy for linting:
```sh
just check
```

For JSON output (useful for IDE integration):
```sh
just check-json
```

### Project Structure

```
cosmic-ext-applet-restart-to/
├── src/
│   ├── main.rs          # Entry point
│   ├── app.rs           # Main application logic
│   ├── boot.rs          # EFI boot entry management
│   ├── config.rs         # Configuration handling
│   └── i18n.rs          # Localization setup
├── i18n/
│   └── en/              # English translations
├── resources/
│   ├── app.desktop      # Desktop entry file
│   ├── app.metainfo.xml # AppStream metadata
│   └── icon.svg         # Applet icon
├── Cargo.toml           # Rust dependencies
├── justfile             # Build commands
└── README.md            # This file
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `just check` to ensure code quality
5. Test thoroughly (especially EFI functionality)
6. Submit a pull request

### Troubleshooting

#### Applet crashes on startup
- Check logs: `journalctl -u cosmic-panel -n 50`
- Verify all dependencies are installed
- Try rebuilding: `cargo clean && just build-release`

#### Boot entries not loading
- Check EFI access: `sudo efibootmgr`
- Verify UEFI system: `[ -d /sys/firmware/efi ]`
- Check permissions on `/sys/firmware/efi/efivars/`

#### Reboot doesn't work
- Verify D-Bus access: `systemctl --user status`
- Check systemd-logind: `systemctl status systemd-logind`
- Verify zbus dependency is working

For more help, check the [COSMIC Applets repository](https://github.com/pop-os/cosmic-applets) for similar implementations.

[fluent]: https://projectfluent.org/
[fluent-guide]: https://projectfluent.org/fluent/guide/hello.html
[iso-codes]: https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes
[just]: https://github.com/casey/just
[rustup]: https://rustup.rs/
[rust-analyzer]: https://rust-analyzer.github.io/
[sccache]: https://github.com/mozilla/sccache
