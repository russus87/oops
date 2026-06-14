# Diario di implementazione — Oops

Interfaccia grafica moderna per Git. Rust + Tauri 2 + Svelte 5.
Stessa impalcatura di Oxiterm (workspace Cargo `core` + `src-tauri`, CI verso repo
privato `oops` + repo pubblico `oops-dist` con artefatti firmati).

## v0.5.0 (2026-06-14) — Fase 5 (tutto il resto: punti 1-8)

Implementati i punti 1-8 della roadmap residua (LFS e bisect esclusi: LFS richiede
il binario esterno git-lfs, bisect non ha API in git2). Build: `cargo test -p oops-core`
(11 verdi, +squash interattivo), `cargo build -p oops` OK, `npm run build` OK.

### core/
- `rebase_int.rs` — rebase interattivo "a mano" (pick/squash/reword/drop) replicando
  i commit con `cherrypick_commit` + `write_tree_to`, poi sposta il ramo. + test.
- `reflog.rs`, `submoduli.rs` (lista/aggiorna), `worktree.rs` (lista/aggiungi),
  `patch.rs` (esporta diff commit / applica .patch con `Diff::from_buffer`).
- `conflitti.rs` — `versioni` (base/nostra/loro/corrente dai 3 stage dell'indice) e
  `salva` per l'editor a 3 vie.
- `diff.rs` — opzione `ignora_spazi` (ignore_whitespace) su file/commit/commit_file.
- `remote.rs` invariato da v0.4; `model.rs` — nuovi tipi (VoceReflog, Submodulo,
  VoceWorktree, ConflittoVersioni, MossaRebase).

### src-tauri/
- Nuovi comandi per tutto quanto sopra. `Osservatore` (notify): comando
  `avvia_osservatore` che osserva la cartella ed emette l'evento `oops-fs`. Dipendenza `notify`.

### src/ (frontend)
- `Diff.svelte` — evidenziazione per parola (prefisso/suffisso comuni) e toggle "ignora spazi".
- `MergeEditor.svelte` — editor di merge a 3 vie (con "Usa" per copiare una versione).
- `RebaseInterattivo.svelte` — editor del piano (azione + riordino ▲▼ + reword/squash).
- `Cronologia.svelte` — grafo a corsie (SVG), ricerca, copia hash, esporta patch,
  apri rebase interattivo sui commit dopo quello scelto.
- `Impostazioni.svelte` — sezioni Patch (applica), Sottomoduli, Worktree, Reflog, Aggiornamenti.
- `App.svelte` — auto-refresh (evento `oops-fs` con debounce + focus finestra), scorciatoie
  (F5/Ctrl+R, Ctrl+1/2), notifiche desktop sulle operazioni di rete, "Apri cartella".
- `stato.svelte.js` — `ignoraSpazi` persistito.

## v0.4.0 (2026-06-14) — Fase 4 (altra tranche corposa)

Build verificata: `cargo test -p oops-core` (10 verdi), `cargo build -p oops` OK,
`npm run build` OK.

### core/
- `remote.rs` — riscritto: `costruisci_callbacks(Option<Credenziali>)` (usa
  utente/password per HTTPS o chiave/passphrase per SSH, altrimenti agent/credential
  helper). `pull` ora accetta una `strategia`: "ff" / "merge" / "rebase" (sui rami
  divergenti). `push(forza)` unifica push normale e forzato. Tutte le funzioni di rete
  (fetch/pull/push/push_tags/elimina_ramo_remoto) e `repo::clona` prendono le credenziali.
- `azioni.rs` — `ripristina_file` (git restore --source <commit> <file>).
- `commit.rs` — `condensa` (squash di commit da uno scelto fino a HEAD: soft reset al
  genitore + commit con l'albero finale).
- `stash.rs` — `diff` (anteprima del contenuto di uno stash).
- `model.rs` — `Credenziali { utente, password, chiave, passphrase }` (mai salvate).

### src-tauri/ — comandi aggiornati (credenziali + strategia pull) e nuovi
(condensa, ripristina_file, stash_diff). `push` ora ha il flag `forza`.

### src/ (frontend)
- `stato.svelte.js` — `chiediCredenziali()` con Promise (modale gestita da App).
- `Credenziali.svelte` — modale HTTPS/SSH; `App.svelte` riprova l'operazione di rete
  con le credenziali quando l'errore è di autenticazione. Menu Pull (ff/merge/rebase).
- `Cronologia.svelte` — barra di ricerca, "Carica altri commit", pulsante Condensa,
  ripristino del singolo file da un commit.
- `BarraLaterale.svelte` — clic su uno stash apre il diff con Applica/Pop/Elimina.

## v0.3.0 (2026-06-14) — Fase 3 (release corposa: tutto il resto)

Build verificata: `cargo test -p oops-core` (10 test verdi, +rebase e +conflitti),
`cargo build -p oops` OK, `npm run build` OK.

### core/ (nuovi moduli + estensioni)
- `conflitti.rs` — `lista`, `risolvi` (nostra/loro dal blob nell'indice),
  `segna_risolto`, `annulla` (reset hard + cleanup_state). + test conflitto→risoluzione.
- `blame.rs` — `blame` riga per riga (blame_file + contenuto della cartella).
- `rami.rs` — `rebase` (non interattivo, abort sui conflitti), `crea_da` (ramo da un
  commit), `checkout_commit` (HEAD staccata). + test rebase lineare.
- `azioni.rs` — `revert` (commit inverso, errore sui conflitti).
- `remote.rs` — `lista_dettagli`, `aggiungi`/`imposta_url`/`rimuovi`, `push_forza`,
  `push_tags`, `elimina_ramo_remoto`.
- `stage.rs` — `scarta_tutto`, `pulisci_non_tracciati`.
- `commit.rs` — `log_file` (cronologia di un singolo file), decorazioni dei ref
  (rami/tag) aggiunte a ogni `VoceLog` via `mappa_riferimenti`.

### src-tauri/ — +24 comandi ponte.

### src/ (frontend)
- `Diff.svelte` — toggle vista **affiancata** (side-by-side) oltre all'unificata.
- `Conflitti.svelte` — pannello risoluzione (nostra/loro/risolto + annulla).
- `Blame.svelte` — modale con Blame e Cronologia del file.
- `Impostazioni.svelte` — modale: tema chiaro/scuro, autore, gestione remoti, info/changelog.
- `Modifiche.svelte` — pannello conflitti, scarta-tutto/pulisci, pulsante Blame per file.
- `Cronologia.svelte` — decorazioni ref sui commit, Revert/Checkout/“Ramo da qui”.
- `BarraLaterale.svelte` — rebase per ramo locale, elimina ramo remoto.
- `App.svelte` — menu Push (force/tag), apertura Impostazioni; tema applicato all'avvio.
- `stato.svelte.js` — tema persistito in localStorage.

## v0.2.0 (2026-06-14) — Fase 2

Aggiunte funzioni "da Git GUI serio". Build verificata: `cargo test -p oops-core`
(8 test verdi), `cargo build -p oops` OK, `npm run build` OK.

### core/ (nuovi moduli + estensioni)
- `stash.rs` — `lista`/`salva`(con file non tracciati)/`applica`/`pop`/`elimina`. + test.
- `tag.rs` — `lista` (distingue leggere/annotate), `crea`, `elimina`. + test.
- `azioni.rs` — `reset` (soft/mixed/hard), `cherry_pick` (applica + commit con autore
  originale, errore sui conflitti), `config_utente`/`imposta_config_utente`.
- `diff.rs` — `lista_file_commit`, `commit_file` (diff di un file in un commit),
  `hunk_stage`/`hunk_scarta`: stage/unstage/scarta del **singolo hunk** via
  `repo.apply` + `hunk_callback`, usando `DiffOptions.reverse(true)` per le direzioni inverse.
- `commit.rs` — `amend` (riscrive l'ultimo commit con lo stage corrente) e
  `ultimo_messaggio` (per precompilare l'amend).

### src-tauri/ — +21 comandi ponte per le funzioni sopra.

### src/ (frontend)
- `Diff.svelte` — divide il diff in hunk; con `onHunk` mostra i pulsanti per hunk.
- `Modifiche.svelte` — azioni per-hunk, checkbox **Amend**, pulsante **Stash**.
- `Cronologia.svelte` — lista file del commit + diff per file, azioni Reset/Cherry-pick.
- `BarraLaterale.svelte` — sezioni **Tag** (crea/elimina) e **Stash** (pop/elimina).
- `App.svelte` — modale ⚙ per impostare nome/email dell'autore.

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
  job Arch (.pkg.tar.zst via makepkg). Il repo è **pubblico**, quindi la release (con
  gli artefatti firmati per l'auto-update) viene pubblicata direttamente qui: niente
  repo `oops-dist` né `DIST_TOKEN`. Niente nasm (non c'è russh); libgit2 vendored vuole
  cmake (+ libssl-dev/pkg-config su Linux).
- `packaging/PKGBUILD` + `oops.desktop`.

### Da fare prima del primo rilascio
- Generare la coppia di chiavi di firma (`npm run tauri signer generate`) e
  mettere la **pubkey** in `tauri.conf.json` (ora c'è un segnaposto) + i segreti
  `TAURI_SIGNING_PRIVATE_KEY` e `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` su GitHub.
- Sostituire le icone segnaposto (copiate da Oxiterm) con quelle di Oops.

### Idee per le prossime fasi
- Pannello diff a righe affiancate (side-by-side) e stage per singola riga/hunk.
- Stash, tag, cherry-pick, rebase interattivo, amend dell'ultimo commit.
- Grafo dei rami nella cronologia; risoluzione conflitti guidata.
- Discard/checkout con conferma; blame; ricerca nei commit.
