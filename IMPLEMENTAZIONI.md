# Diario di implementazione — Oops

Interfaccia grafica moderna per Git. Rust + Tauri 2 + Svelte 5.
Stessa impalcatura di Oxiterm (workspace Cargo `core` + `src-tauri`, CI verso repo
privato `oops` + repo pubblico `oops-dist` con artefatti firmati).

## v0.1.0 (2026-06-14) — Fase 1

Prima versione funzionante. Build verificata: `cargo test -p oops-core` (6 test
verdi), `cargo build -p oops` OK, `npm run build` OK. La GUI non è ancora stata
lanciata dal vivo (serve un display: provala con `cargo tauri dev`).

### core/ (Rust puro su git2 / libgit2, vendored)
- `model.rs` — tipi condivisi: `RepoRecente`, `FileModificato`/`StatoFile`,
  `StatoRepo` (ramo, in_stage/non_in_stage, avanti/indietro, vuoto), `VoceLog`, `Ramo`.
- `repo.rs` — `apri_repo`, `init`, `clona`, `stato` (git status diviso tra
  staging e cartella di lavoro), `nome_ramo`, calcolo avanti/indietro vs upstream.
- `stage.rs` — `aggiungi`/`aggiungi_tutto`, `togli`/`togli_tutto` (reset_default o
  remove_path se repo vuoto), `scarta` (checkout_index forzato). + test.
- `commit.rs` — `log` (revwalk), `crea` (commit dai file in stage), data leggibile
  calcolata a mano (niente chrono). + test.
- `rami.rs` — `lista` (locali+remoti, segna il corrente), `crea`, `checkout`,
  `elimina`, `merge` (già aggiornato / fast-forward / merge con commit a 2 genitori;
  errore chiaro sui conflitti). + test.
- `diff.rs` — `file` (staged = tree vs index, unstaged = index vs workdir) e
  `commit` (vs primo genitore), output in testo unificato. + test.
- `remote.rs` — `lista`, `fetch`, `pull` (solo fast-forward), `push` (+ imposta
  upstream). Credenziali via agent SSH o credential helper di sistema.
- `storage.rs` — repository recenti (JSON, dedup, max 15). + test.

### src-tauri/
- `lib.rs` — 25 comandi ponte verso il core. Unico stato: `FileRecenti` (path del
  JSON in app_config_dir, impostato nel setup).
- Plugin: log, dialog, opener, notification, updater, process.

### src/ (Svelte 5, runes)
- `lib/stato.svelte.js` — stato globale (repo aperto, contatore `tic` per i
  ricarichi, toast).
- `lib/api.js` — wrapper di tutti i comandi `invoke`.
- `App.svelte` — toolbar (Fetch/Pull/Push + ramo + badge avanti/indietro), schede
  Modifiche/Cronologia, toast.
- `components/Avvio.svelte` — apri/init/clona + repository recenti.
- `components/BarraLaterale.svelte` — rami locali/remoti: crea, cambia, elimina, merge.
- `components/Modifiche.svelte` — stato file (stage/unstage/scarta) + riquadro commit + diff.
- `components/Cronologia.svelte` — elenco commit + diff del commit.
- `components/Diff.svelte` — diff colorato (verde/rosso/intestazioni).

### CI / packaging
- `.github/workflows/release.yml` — su tag `v*`: build 3 piattaforme (tauri-action),
  job Arch (.pkg.tar.zst via makepkg), pubblicazione su `oops-dist`. Niente nasm
  (non c'è russh); libgit2 vendored vuole cmake (+ libssl-dev/pkg-config su Linux).
- `packaging/PKGBUILD` + `oops.desktop`.

### Da fare prima del primo rilascio
- Generare la coppia di chiavi di firma (`npm run tauri signer generate`) e
  mettere la **pubkey** in `tauri.conf.json` (ora c'è un segnaposto) + i segreti
  `TAURI_SIGNING_PRIVATE_KEY(_PASSWORD)` e `DIST_TOKEN` su GitHub.
- Sostituire le icone segnaposto (copiate da Oxiterm) con quelle di Oops.

### Idee per le prossime fasi
- Pannello diff a righe affiancate (side-by-side) e stage per singola riga/hunk.
- Stash, tag, cherry-pick, rebase interattivo, amend dell'ultimo commit.
- Grafo dei rami nella cronologia; risoluzione conflitti guidata.
- Discard/checkout con conferma; blame; ricerca nei commit.
