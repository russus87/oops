# Diario di implementazione — Oops

Interfaccia grafica moderna per Git. Rust + Tauri 2 + Svelte 5.
Stessa impalcatura di Oxiterm (workspace Cargo `core` + `src-tauri`, CI verso repo
privato `oops` + repo pubblico `oops-dist` con artefatti firmati).

## v0.9.0 (2026-07-18) — Rifiniture (le voci 🟦 della roadmap)

Completate le rifiniture rimaste. Build: `cargo test -p oops-core` (12 verdi),
`cargo build -p oops` OK, `npm run build` OK.

### core/
- `contenuto.rs` (nuovo) — lettura file per le anteprime: `testo_lavoro` (markdown),
  `b64_lavoro`/`b64_head` (immagini in base64, con encoder base64 fatto in casa + test).

### src-tauri/ — comandi `leggi_testo_lavoro`, `leggi_b64_lavoro`, `leggi_b64_head`.

### src/ (frontend)
- **Diff**: toggle **Anteprima** — Markdown renderizzato (mini-parser in `util.js`) e
  **confronto immagini prima/dopo** (HEAD vs working, con sfondo a scacchi).
- **App**: **Redo** (Ctrl+Shift+Z) che ripristina lo stato prima dell'Undo (cattura
  HEAD e `reset --hard`), con pulsante in toolbar.
- **Cronologia**: **hover card** ricca sul commit (SHA, autore+email, data, genitori,
  ref); **grafo "live"**: le linee si disegnano quando entra un nuovo commit.
- **Impostazioni** + **Terminale**: **Azioni personalizzate** (comandi git salvati in
  localStorage, eseguibili con un clic dal Terminale).
- `util.js` — `markdownToHtml`, `mimeDa`, `ESTENSIONI_IMG`.
- `stato.svelte.js` — `azioniGit` (persistite), `redoOid`.

## v0.8.0 (2026-07-18) — Completamento roadmap "offline" + AI (Anthropic)

Seconda grande tranche: implementate tutte le voci realizzabili offline delle Fasi
8-13 rimaste, più l'assistente AI. Escluse solo (per scelta dell'utente) le
integrazioni che richiedono servizi esterni: PR, Jira, CI, plugin system. Build:
`cargo test -p oops-core` (11 verdi), `cargo build -p oops` OK, `npm run build` OK.

### core/
- `rami.rs` — `lista` ora riporta avanti/indietro e ultimo commit per ramo.
- `rebase_int.rs` — azione **fixup**; `rimuovi(id)` (drop di un commit in mezzo).
- `azioni.rs` — cherry-pick **squash** e **move** (copia + rimuovi dall'origine).
- `diff.rs` — `tra_commit` (Compare with…), `staged_tutto` (per l'AI),
  `applica_indice` (stage per singola riga), size dei file nelle statistiche.
- `commit.rs` — `tra(da, a)` per le release notes fra due ref.
- `insights.rs` — file più modificati + totale righe aggiunte/rimosse.
- `storage.rs` — workspace (gruppi di repository) in un JSON.
- `model.rs` — `Workspace`; campi extra su `Ramo`, `StatFile`, `Insights`.

### src-tauri/
- Comandi: diff_tra_commit, stage_righe, cherry_pick_squash/muovi, commit_rimuovi,
  note_release, workspace_lista/salva/elimina, genera_commit_ai.
- **AI Anthropic** via `ureq`: `genera_commit_ai` invia il diff in stage all'API
  Messages e restituisce il messaggio di commit.

### src/ (frontend)
- **Diff**: modalità "per riga" (selezione di singole righe → patch parziale → stage).
- **Cronologia**: zoom, filtro per autore e "solo merge", **Compare with…** (banner
  di confronto fra due commit).
- **BarraLaterale**: drop commit→ramo con menu **Copy/Move/Squash**; badge
  avanti/indietro e tooltip ultimo commit sui rami.
- **Modifiche**: pulsante **✨ Genera** (messaggio di commit via AI); barre +/- e lingua.
- **Impostazioni**: sezione **AI (Anthropic)** con token (solo locale) e modello.
- **Insights**: card "file più modificati" e "righe cambiate".
- **Avvio**: **Workspace** (crea gruppi di repository dai recenti e apri al volo).
- `stato.svelte.js` — `aiToken`/`aiModello` persistiti; `impostaAi`.

## v0.7.0 (2026-07-18) — Fasi 7-13 (grande tranche: la roadmap "concorrente di Fork")

Ondata ampia che copre gran parte della roadmap (dettagli e stato in `ROADMAP.md`).
Build verificata: `cargo test -p oops-core` (11 verdi), `cargo build -p oops` OK,
`npm run build` OK. Restano fuori (richiedono rete/credenziali) le integrazioni PR/
Jira/CI, il plugin system e l'AI: scelta consapevole, non implementabili offline.

### core/
- `rami.rs` — `merge_rami(sorgente, destinazione)` e `rebase_rami(sorgente,
  destinazione)`: logica del drag&drop ramo-su-ramo (checkout + merge/rebase).
- `diff.rs` — `stat_lavoro`/`stat_commit` (righe +/- e flag binario per file, per le
  barre proporzionali) e `calore` (churn per commit, per la heat map).
- `insights.rs` (nuovo) — statistiche locali: top contributori, attività per settimana
  e per giorno, linguaggi (per estensione nell'albero HEAD).
- `azioni.rs` — `annulla_ultima`: undo universale via reflog (reset allo stato prima
  dell'ultima operazione che ha mosso HEAD).
- `esegui.rs` (nuovo) — runner del terminale git integrato (`git <args>`, solo il
  binario git).
- `model.rs` — nuovi tipi `StatFile`, `Insights`/`Conteggio`, `Calore`.

### src-tauri/ — comandi: merge_rami, rebase_rami, stat_lavoro, stat_commit, calore,
insights, annulla_ultima, esegui_git.

### src/ (frontend) — nuove viste e componenti
- **Navigazione** spostata in barra laterale (Panoramica/Modifiche/Cronologia/Insights/
  Timeline/Terminale) con contatori; **status bar** in basso; **toolbar** con Cerca,
  Undo, Git Flow.
- `Insights.svelte` — grafici (contributori, attività settimana/giorno, linguaggi).
- `Timeline.svelte` — cronologia delle azioni di sessione (da `stato.registra`).
- `Terminale.svelte` — terminale git con storia comandi (↑/↓) e refresh della UI.
- `Ricerca.svelte` — palette globale **Ctrl+K** (commit/rami/tag/file → azioni).
- `GitFlow.svelte` — assistente guidato New Feature → Branch → Commit → Push → Merge.
- `BarraStat.svelte` — barre "+/-" riusabili; pallino "lingua" per estensione.
- **Fase 8**: drag&drop ramo-su-ramo con menu **Merge/Rebase**; **menu contestuale**
  (clic destro) sui commit; **rebase interattivo con drag-to-reorder**.
- **Fase 9**: barre +/- e lingua nelle liste file (Modifiche + Cronologia).
- **Fase 12**: **heat map** del grafo (🔥, colore verde→rosso per churn) e animazione
  "live" all'ingresso dei nuovi commit (each keyed + transizione).
- Undo con **Ctrl+Z**; salto-al-commit dalla ricerca; File History dai file del commit.
- `stato.svelte.js` — `vista`, `trascina` (commit/ramo), `heatMap`, `azioni`
  (timeline), `ricercaAperta`, `commitScelto`, `storiaFile`, `registra`, `vaiACommit`.

## v0.6.1 (2026-07-18) — Fase 7 (avvio): Dashboard "Repository Health"

Prima consegna della Fase 7 (layout "DevStudio"). Build: `cargo test -p oops-core`
(11 verdi), `cargo build -p oops` OK, `npm run build` OK.

### core/
- `panoramica.rs` (nuovo) — `panoramica(percorso)` assembla in una sola chiamata
  ramo/upstream, avanti/indietro, `ultimo_fetch` (mtime di `.git/FETCH_HEAD`),
  conteggi (rami locali/remoti, tag, stash, conflitti), stato della working tree
  (in stage / modificati / non tracciati), remoti e ultimo commit. Riusa i moduli
  esistenti (repo/rami/tag/stash/conflitti/remote/commit).
- `model.rs` — nuovo tipo `Panoramica`.

### src-tauri/ — nuovo comando `panoramica`.

### src/ (frontend)
- `Dashboard.svelte` (nuovo) — tessere "Repository Health": badge di salute
  (verde/giallo/rosso), sincronia ↑↓, ultimo fetch, conflitti, working tree, conteggi
  rami/tag/stash, ultimo commit con avatar. Tessere cliccabili per saltare alle viste.
- `App.svelte` — nuova scheda **Panoramica** (vista predefinita all'apertura);
  scorciatoie rimappate Ctrl+1 Panoramica / Ctrl+2 Modifiche / Ctrl+3 Cronologia.
- `api.js` — `panoramica`.
- `ROADMAP.md` (nuovo) — piano storico delle fasi (7→13+) con la visione completa.

## v0.6.0 (2026-07-18) — Fase 6 (feature "stile Fork": grafo, drag&drop, conflitti)

Prima tranche del piano "concorrente di Fork/SourceTree": grafo moderno, cherry-pick
drag&drop, merge viewer e risoluzione conflitti a blocchi. Build verificata:
`cargo test -p oops-core` OK, `cargo build -p oops` OK, `npm run build` OK.

### core/
- `model.rs` — `VoceLog` arricchita: `timestamp` (Unix, per il tempo relativo) e
  `decori: Vec<Riferimento>` al posto di `riferimenti: Vec<String>`. Nuovo tipo
  `Riferimento { nome, tipo }` con tipo "testa" (HEAD), "locale", "remoto" o "tag".
- `commit.rs` — `mappa_riferimenti` ora classifica i ref per tipo (branch/remote/tag)
  e aggiunge HEAD; `in_voce` ordina i badge (HEAD → locali → remoti → tag) e popola
  timestamp/decori.
- `azioni.rs` — `cherry_pick_su(percorso, id, ramo)`: checkout del ramo + cherry-pick
  (logica dietro il drag&drop di un commit su un ramo).
- `diff.rs` — `commit_vs_genitore(percorso, id, genitore, ignora_spazi)`: diff di un
  commit rispetto a un genitore scelto (per vedere i due lati di un merge).

### src-tauri/ — nuovi comandi `cherry_pick_su`, `diff_commit_genitore`.

### src/ (frontend)
- `lib/util.js` (nuovo) — avatar deterministici (iniziali + colore da email),
  `tempoRelativo` ("3 h fa"), e `calcolaGrafo` che produce le corsie del grafo con i
  segmenti (linee/curve) da disegnare riga per riga.
- `Cronologia.svelte` — grafo a corsie con **curve di Bézier** tra i nodi, **avatar**
  dell'autore, **badge dei ref** colorati per tipo (stile Fork), **tempo relativo**;
  righe **trascinabili** (drag&drop) e **merge viewer** (scelta del genitore da
  confrontare per i commit di merge).
- `BarraLaterale.svelte` — i rami locali sono **bersagli di drop**: trascinaci un
  commit per fare cherry-pick su quel ramo (con evidenziazione del bersaglio).
- `MergeEditor.svelte` — riscritto **a blocchi**: il file viene spezzato nei suoi
  conflitti; per ognuno si sceglie *Accetta nostra / loro / entrambi / manuale*
  (colonne Nostra/Base/Loro), con conteggio "risolti" e salvataggio solo a conflitti
  chiusi. Sostituisce la copia dell'intero file.
- `stato.svelte.js` — `trascina` (commit in trascinamento, per evidenziare i bersagli).
- `api.js` — `cherryPickSu`, `diffCommitGenitore`.

### Indagini di mercato (in `mockups/` i riferimenti visivi: Fork, GitKraken, DevStudio)
- SourceTree: stagnante (UI ferma, no Linux, no 3-way conflitti, lento su repo grandi).
  Sorpassi possibili: conflict resolver interno, undo universale, prestazioni, Linux,
  no login obbligatorio, AI. Vedi roadmap Fase 7+.

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
