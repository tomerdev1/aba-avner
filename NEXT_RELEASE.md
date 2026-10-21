# Next release

## Microsoft Store: declare all UI languages
`src-tauri/gen/windows/AppxManifest.xml.template` only lists `<Resource Language="en-us" />`, so Partner Center shows the app as English-only even though the UI has 7 languages.

- Add `Resource` entries for `he`, `es`, `zh-Hans`, `fil`, `sw`, `tpi` (keep `en-us`).
- Rebuild the MSIX (CI `windows-msix` artifact) and upload it as a new submission with a higher version.
- Optionally add translated store listing text for the new languages in Partner Center.

## Snap: try moving to core26
The host is Ubuntu 26.04, so the `core24` snap can only be built in LXD (needs the `lxd` group, firewall rules for `lxdbr0` because of Docker, and the `rust-deps` part). Building on `core26` would be native and drop all of that.

- Confirm Snapcraft's `core26` support is ready.
- Switch `base`, and the `gnome` extension moves to a newer GNOME runtime; check the build packages (`libwebkit2gtk-4.1-dev` etc.) exist on 26.04.
- It changes the runtime for every user (fonts, theming, file picker), so release to `edge` first, test, then promote to `stable`.
- Until then: build with `snapcraft pack --use-lxd` after re-adding the Docker firewall rules (they are cleared on reboot):
  `sudo iptables -I DOCKER-USER -i lxdbr0 -j ACCEPT` and
  `sudo iptables -I DOCKER-USER -o lxdbr0 -m state --state ESTABLISHED,RELATED -j ACCEPT`

## Also pending
- Merge PR #8 (snap `rust-deps` part), needed to build the snap in LXD on this Ubuntu 26.04 host: `snapcraft pack --use-lxd`.
- Have native speakers review the Filipino, Swahili and Tok Pisin translations.
- Remove the `zip` advisory ignores in `src-tauri/.cargo/audit.toml` once `tauri-plugin-updater` moves to `zip` >= 9.
