# Oops

> Git senza panico. Quando sbagli, niente paura. 🙃

Un'interfaccia grafica moderna per **Git**, multipiattaforma (Windows, macOS, Linux),
costruita con **Rust + Tauri 2 + Svelte 5**. Stessa impalcatura di Oxiterm.

## Cosa fa (v0.1.0)

- Apri / inizializza / clona un repository
- Stato dei file: **stage**, **unstage**, **scarta** modifiche
- **Commit** con i file in staging
- **Cronologia** dei commit con diff
- **Rami**: elenca, crea, cambia, elimina, **merge** (fast-forward e normale)
- **Diff** colorato dei file e dei commit
- **Fetch / Pull / Push** verso il remoto (credenziali via agent SSH o helper di sistema)
- Elenco dei repository recenti

## Architettura

Workspace Cargo con due crate, come Oxiterm:

- **`core/`** — Rust puro su `git2` (libgit2). Nessuna dipendenza da Tauri,
  quindi testabile e riusabile. Moduli: `repo`, `stage`, `commit`, `rami`,
  `diff`, `remote`, `storage`, `model`.
- **`src-tauri/`** — comandi ponte (`invoke`) verso il frontend.
- **`src/`** — interfaccia Svelte 5 (runes).

## Sviluppo

```bash
npm install
cargo tauri dev      # avvia l'app
cargo test -p oops-core   # test del core
```

Serve **cmake** (per compilare libgit2 vendored) e, su Linux, `libssl-dev` + `pkg-config`.

## Rilascio

Push di un tag `vX.Y.Z` → la CI (`.github/workflows/release.yml`) costruisce i
pacchetti per le tre piattaforme + `.pkg.tar.zst` per Arch e pubblica la release
(con gli artefatti firmati per l'auto-update) direttamente su questo repo, che è
pubblico (Actions illimitate, niente repo `-dist` separato).
