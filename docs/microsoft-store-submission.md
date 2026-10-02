# Microsoft Store Submission

**Recommended path: MSIX.** It needs no purchased code-signing certificate (Microsoft
re-signs it for free during certification) and gets free automatic updates via Windows
Update — Store policy 10.2.9 requires a real Trusted-Root-chained certificate for a
plain EXE/MSI submission, which costs money either way (a CA cert or Azure Artifact
Signing). MSIX sidesteps that entirely. An EXE/MSI path still exists below as an
alternative (e.g. for distributing outside the Store), but it costs money and needs
the custom self-updater to get any update mechanism at all.

## MSIX (recommended)

Built via the community [tauri-windows-bundle](https://github.com/Choochmeque/tauri-windows-bundle)
tool — Tauri doesn't produce MSIX natively. Already wired into this repo:
`src-tauri/gen/windows/bundle.config.json` + `AppxManifest.xml.template` (generated via
`init`, committed to git), source icons for the MSIX tile sizes at
`src-tauri/icons/{StoreLogo,Square44x44Logo,Square150x150Logo}.png`, and a CI step
in `.github/workflows/ci.yml`'s `windows-latest` job that builds it on every push and
uploads it as the `windows-msix` artifact.

### One-time setup
1. Register a Partner Center developer account: https://partner.microsoft.com/dashboard/registration
   (free as of May 2026).
2. **If you already reserved "Aba Avner" as an EXE/MSI app** (see the alternative path
   below), delete that reservation first — Microsoft won't let the same name be reserved
   twice, MSIX or not.
3. Partner Center → Apps and games → **New product** → **MSIX or PWA app**, reserve the name.
4. Partner Center issues a real **Publisher CN**, **Package/Identity Name**, and
   **PublisherDisplayName** for the reservation (visible on the app's Identity page) —
   all three must match the manifest *exactly*, or Partner Center rejects the package
   at upload with a "doesn't match your publisher display name"-style error. Already
   set in `src-tauri/gen/windows/bundle.config.json` from a real reservation:
   - `publisher: "CN=A8E7E0D9-58CE-4E17-B47A-403BDD24E966"`
   - `publisherDisplayName: "tomerdev"`
   - `identifier: "tomerdev.AbaAvner"` — overrides the cross-platform app identifier
     (`com.tomer.aba-avner` in `tauri.conf.json`, used by the NSIS/macOS/Linux builds)
     specifically for MSIX, since Partner Center assigned a different one for this
     package identity. Same override mechanism as `displayName` above.
   If you ever re-reserve under a different Partner Center account/app, these three
   values need to change to match the new reservation's Identity page.

### Get the built package
Either download the `windows-msix` artifact from the latest CI run
(`src-tauri/target/msix/*.msix` / `*.msixbundle`), or build locally on a Windows
machine:
```powershell
cargo install msixbundle-cli   # one-time
npx @choochmeque/tauri-windows-bundle@latest build --arch x64 --runner cargo
```

### Submission (Partner Center → your app → new submission)
- **Packages**: upload the `.msix`/`.msixbundle` file directly — no external hosting
  needed, unlike EXE/MSI (no redirect-URL problem to work around either).
- **Store listing / Age ratings / Category**: same as the EXE/MSI path below — reuse
  `snap/snapcraft.yaml`'s `summary`/`description`, offline/no-data-collection age
  rating answers, Photo & Video category.
- No silent-install-switch fields to fill in — MSIX installs are always silent by
  the format's design.

### Updates
Once submitted as MSIX, **updates are free and automatic via Windows Update** — the
OS checks every ~24h and installs new versions with no app code involved. The
`tauri-plugin-updater` wiring described below (self-signed `.tauri-updater.key`,
`checkForUpdate()`/`installUpdate()`, the confirm-dialog prompt) is unnecessary for
this distribution channel — it stays relevant only if you also distribute the raw
NSIS `.exe` outside the Store (a direct download from your own site, say).

### Capabilities
`bundle.config.json`'s `capabilities.general` is currently `[]` — Aba Avner runs
fully offline and doesn't need `internetClient` or any other declared capability.
Add one only if a real feature needs it later (Store review flags capabilities that
don't match actual app behavior).

### Known tool limitation
`tauri-windows-bundle`'s asset generation only copies from `src-tauri/icons/` when it
finds files with the *exact* expected names (`StoreLogo.png`, `Square44x44Logo.png`,
`Square150x150Logo.png`) — those didn't exist in this repo, so the tool silently wrote
solid-gray placeholders instead of resizing the real icon (confirmed by pixel-checking
the generated PNGs). Fixed by adding those three correctly-sized files (resized from
`src-tauri/icons/logo-1080.png`) to `src-tauri/icons/`, matching what the tool expects.
Re-running with `--regenerate-assets` should now render them correctly instead of the
gray placeholder.

A second limitation, hit and fixed via CI (`makeappx.exe`/Windows SDK only exists on
`windows-latest`, not any Linux dev machine): the tool assumes the compiled binary is
always named `productName` with spaces stripped (`AbaAvner.exe`) — ours is `aba-avner`
(kebab-case, set explicitly in `Cargo.toml`, unrelated to `productName`), and the tool
has no config knob for that mismatch (confirmed against the latest version, 0.1.28).
Fixed with the smallest-blast-radius option: `bundle.config.json`'s `displayName`
overrides the tauri.conf.json-derived default the tool otherwise computes this from,
so it's set to `"aba-avner"` there — matches the real binary, MSIX-config-local, zero
risk to the already-working Snap/NSIS builds. Trade-off: the installed app's Start Menu
tile shows "aba-avner" (lowercase) instead of "Aba Avner". Partner Center's own Store
listing title is unaffected — that's independent, set directly in Partner Center. Fix
properly later by renaming the Cargo `[[bin]]` to `AbaAvner` if the cosmetic mismatch
matters enough to justify also updating `snap/snapcraft.yaml`'s `command: bin/aba-avner`
(and re-verifying the Snap build, which needs `snapcraft` tooling to test).

---

## Alternative: EXE/MSI direct submission

The Store also accepts a plain Win32 installer directly, no MSIX packaging — the NSIS
`.exe` from `cargo tauri build --bundles nsis`. CI no longer builds this automatically
(only the MSIX path runs on every push); build it locally/on-demand instead. Worth it
only if you specifically want to distribute this build outside MSIX/the Store too;
otherwise the MSIX path above is strictly less work and less cost.

### One-time setup
1. Register a Partner Center developer account (see above).
2. Partner Center → Apps and games → **New product** → **EXE or MSI app**, reserve a name.
3. **Get a code-signing certificate.** Unlike MSIX, the Store does **not** re-sign
   EXE/MSI installers — yours must already be signed with a cert chaining to the
   [Microsoft Trusted Root Program](https://learn.microsoft.com/en-us/security/trusted-root/participants-list)
   before you can submit (Store policy 10.2.9). Self-signed doesn't count. Options:
   - [Azure Artifact Signing](https://azure.microsoft.com/en-us/products/artifact-signing)
     (formerly Trusted Signing) — ~$10/mo, fastest to set up, but only available to
     individual developers in the US/Canada or organizations in the EU/UK.
   - A traditional OV code-signing certificate from a CA (DigiCert, Sectigo,
     SSL.com, …) — works from anywhere, costs more (~$200–500/yr), identity
     verification can take days.
   - Sign the built installer with `signtool sign /fd SHA256 /a aba-avner-setup.exe`
     (or your CA's signing tool) after each build, before uploading anywhere.

### Build the installer
```sh
./scripts/release-check.sh windows   # on a Windows machine/runner
```
Installer lands at `src-tauri/target/release/bundle/nsis/*.exe` — actual name is
`Aba Avner_<version>_x64-setup.exe` (from `productName` in `tauri.conf.json`,
confirmed against a real CI build back when CI still built this automatically).
Sign it before uploading anywhere — see the certificate step above.

### Submission (Partner Center → your app → new submission)
- **Packages**: MSI/EXE apps aren't uploaded directly — you give Partner Center a
  *versioned URL* where the installer is hosted, and it **must not redirect** —
  Partner Center's own validation rejects a redirecting URL outright. This rules
  out GitHub *Releases* download links: they always 302 to a signed,
  hour-expiring `release-assets.githubusercontent.com` URL, verified against the
  actual v0.1.0 release. Use `raw.githubusercontent.com` instead — commit the
  installer straight into a versioned folder in the public `aba-avner-releases`
  repo (not as a Release asset) and it's served directly, no redirect:
  ```sh
  cd /tmp && git clone https://github.com/tomerdev1/aba-avner-releases.git && cd aba-avner-releases
  mkdir v0.1.0 && cp /path/to/aba-avner-setup.exe v0.1.0/
  git add v0.1.0/aba-avner-setup.exe && git commit -m "Add v0.1.0 installer" && git push
  ```
  URL: `https://raw.githubusercontent.com/tomerdev1/aba-avner-releases/main/v0.1.0/aba-avner-setup.exe`
  (verified with `curl -I` — a plain 200, no redirect). This repo holds only
  release files, kept apart from the private `Aba-Avner` source repo so the
  source itself never has to go public. **The binary at that path must never
  change after submission** — add a new version folder for updates instead of
  overwriting an existing one.
- **Silent install switches**: `/S` (Tauri's NSIS installer supports it by default).
- **Silent uninstall switches**: `/S` (same NSIS uninstaller).
- **Install command line arguments**: leave blank.
- **Store listing**: description/screenshots — reuse the copy in `snap/snapcraft.yaml`'s
  `summary`/`description`, it's already Store-appropriate marketing copy.
- **Age ratings**: fill in the IDSK questionnaire (no network/data collection — app
  runs fully offline).
- **Properties → Category**: Photo & Video.

Before submitting, update `bundle.publisher` / `bundle.copyright` in
`src-tauri/tauri.conf.json` — currently placeholders — so the installed app's
Windows properties match your real Partner Center publisher name.

### Notes
- `installMode: "currentUser"` (set in `tauri.conf.json`) installs per-user, no UAC
  prompt — keeps the install flow simple.

### Updates
Microsoft's own docs are explicit about unpackaged EXE/MSI apps: *"The Store does
not provide these updates automatically or manually to existing users."* Unlike
MSIX, there's no Windows Update integration — the app has to check for and
install its own updates. That's what `tauri-plugin-updater` (wired into
`src-tauri/src/main.rs`) is for.

**How it works here:**
- `src-tauri/tauri.conf.json` → `plugins.updater.endpoints` points at a JSON
  manifest URL (`latest.json`) that lists the current version + a signed
  download URL. This one stays on GitHub *Releases* (unlike the Package URL
  above) — Tauri's own HTTP client follows redirects fine, only Partner
  Center's crawler refuses to.
- `plugins.updater.pubkey` is the public half of a signing keypair — already
  generated and embedded (see below). The app refuses to install anything not
  signed by the matching private key.
- On every launch (outside the test harness), `ui/src/services/update-check.js`
  calls `checkForUpdate()`; if a newer version is live, it prompts the user
  (`window.confirm`) and calls `installUpdate(rid)` if they accept. On Windows
  that spawns the new installer and exits the app — the installer relaunches it.

**The private signing key** — `.tauri-updater.key` at the repo root — was
generated with `cargo tauri signer generate` and is **git-ignored on purpose**.
Back it up somewhere safe outside git (a password manager or secrets vault).
If it's lost, you can never sign an update the existing install base will
accept again — you'd have to ship a new pubkey, which means every existing
user has to manually reinstall.

**Publishing an update:**
1. Bump `version` in `src-tauri/tauri.conf.json`.
2. Build the installer (`cargo tauri build --bundles nsis` on Windows). With
   `TAURI_SIGNING_PRIVATE_KEY` (contents of `.tauri-updater.key`) and, if you
   set one, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` set as env vars, the bundler
   also emits a `.sig` file next to the installer.
3. Write `latest.json` (the updater's endpoint is `.../releases/latest/download/latest.json`,
   so this file must be attached to whichever release GitHub marks "latest"). The
   installer's built name `Aba Avner_0.2.0_x64-setup.exe` gets its space sanitized
   to a period by GitHub on upload — the served URL is `Aba.Avner_0.2.0_x64-setup.exe`:
   ```json
   {
     "version": "0.2.0",
     "notes": "What changed in this release",
     "pub_date": "2026-08-22T00:00:00Z",
     "platforms": {
       "windows-x86_64": {
         "signature": "<contents of the .sig file from step 2>",
         "url": "https://github.com/tomerdev1/aba-avner-releases/releases/download/v0.2.0/Aba.Avner_0.2.0_x64-setup.exe"
       }
     }
   }
   ```
4. Publish all three as one release in the public `aba-avner-releases` repo (no
   need to rename anything — `gh` uploads the files as-is):
   ```sh
   gh release create v0.2.0 \
     "bundle/nsis/Aba Avner_0.2.0_x64-setup.exe" \
     "bundle/nsis/Aba Avner_0.2.0_x64-setup.exe.sig" \
     latest.json \
     --repo tomerdev1/aba-avner-releases --title v0.2.0 --notes "What changed"
   ```
5. Also submit the new installer URL as a normal Partner Center update (new
   customers still install through the Store listing, not the updater).

Skipped for now: a proper in-app banner/dialog instead of a native
`window.confirm()`, and localized retry/backoff if the check fails (it just
logs and gives up silently). Revisit if a bare confirm() feels too plain, or
update checks need to be more resilient than "try once on launch."
