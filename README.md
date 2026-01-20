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
   - You have appropriate permissions (may require root or polkit policies)

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

#### Enable Debug Logging

Run with full backtraces and logging:
```sh
RUST_BACKTRACE=full RUST_LOG=debug just run
```

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
   - EFI variable access typically requires root or proper polkit policies
   - Check if `efibootmgr` works: `sudo efibootmgr`
   - May need to configure polkit rules (see Permissions section)

4. **Build errors**:
   - Ensure all dependencies are installed
   - Try `cargo clean && just build-release`
   - Check Rust version: `rustc --version` (should be 1.70+)

### Permissions

The applet needs access to EFI variables, which typically requires elevated privileges. Options:

1. **Run with sudo** (not recommended for production):
   - The applet would need to be launched with sudo, which is not ideal

2. **Polkit policy** (recommended):
   - Create a polkit policy file to allow EFI variable access
   - Example policy (needs to be created):
     ```xml
     <?xml version="1.0" encoding="UTF-8"?>
     <!DOCTYPE policyconfig PUBLIC
       "-//freedesktop//DTD PolicyKit Policy Configuration 1.0//EN"
       "http://www.freedesktop.org/software/polkit/policyconfig-1.dtd">
     <policyconfig>
       <action id="com.github.cosmic_ext.restartTo.manage-boot">
         <description>Manage EFI boot entries</description>
         <message>Authentication is required to manage boot entries</message>
         <defaults>
           <allow_any>auth_admin</allow_any>
           <allow_inactive>auth_admin</allow_inactive>
           <allow_active>auth_admin</allow_active>
         </defaults>
       </action>
     </policyconfig>
     ```

3. **Group membership**:
   - Add user to a group with EFI access (if configured on your system)

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
