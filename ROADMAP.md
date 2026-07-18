# Roadmap — Oops

> Documento storico e vivo. Traccia la visione, le fasi e lo stato di avanzamento
> verso l'obiettivo: **il miglior commit graph sul mercato, drag&drop per tutte le
> operazioni comuni, UI curatissima (stile Linear/Arc/Raycast)**.
>
> Principio guida: **"Il client deve farti capire cosa sta succedendo, non cosa Git
> sta facendo."** L'utente medio usa sempre le stesse ~15 operazioni: mostrargliene
> 300 è il problema di tutti i client attuali.
>
> Il diario tecnico dettagliato per versione è in `IMPLEMENTAZIONI.md`.
> I riferimenti visivi (mockup Fork, GitKraken, "DevStudio") sono in `mockups/`.

---

## Stato attuale — v0.6.0 (2026-07-18)

Core Git già molto completo (v0.1–0.5) + prima tranche "stile Fork" (v0.6).

### ✅ Fatto
- **Plumbing Git**: apri/init/clona, stage/unstage/scarta (anche per **hunk**),
  commit/amend/**squash**, cronologia + diff (unificato/affiancato/word-diff),
  rami (crea/checkout/merge/rebase/elimina), tag, stash, cherry-pick, revert, reset,
  **rebase interattivo**, conflitti, blame, reflog, submoduli, worktree, patch,
  remoti multipli, fetch/pull/push con credenziali, auto-refresh (filesystem watcher).
- **v0.6 — feature "stile Fork"**:
  - Commit graph moderno: corsie colorate con **curve di Bézier**, nodo di merge
    distinto, **avatar** autore, **tempo relativo**.
  - **Badge dei ref** tipizzati (HEAD ✓ / locale ⎇ / remoto ☁ tratteggiato / tag 🏷).
  - **Cherry-pick drag&drop** (commit → ramo nella barra laterale).
  - **Merge viewer** (scelta del genitore da confrontare).
  - **Conflitti a blocchi** (Accetta nostra/loro/entrambi/manuale, colonne Nostra/Base/Loro).

---

## Le tre scommesse distintive (nord della bussola)

1. **Commit Graph eccezionale** — fluido, interattivo, ricco di informazioni. È qui che
   va speso ~40% del tempo. Culmine: **grafo "Live" animato** (i commit si staccano e
   riattaccano durante rebase/merge) e **Heat Map** (verde/giallo/rosso per quantità di
   modifiche).
2. **Drag & Drop per tutto** — merge, cherry-pick, rebase, squash, tag, fast-forward:
   ridurre al minimo menu e finestre di dialogo.
3. **UI curatissima** — ispirata a Linear / Arc / Raycast: pulita, animazioni leggere,
   focalizzata sulla produttività.

---

## Fasi pianificate

Legenda stato: ⬜ da fare · 🟦 in corso · ✅ fatto

### Fase 7 — Layout "DevStudio" + Dashboard  ✅ (grosso fatto)
Il mockup #3 è la stella polare del layout.
- ✅ **Dashboard d'apertura (Repository Health)** *(v0.6.1)*: salute del repo
  (verde/giallo/rosso), branch corrente + upstream, ahead/behind, ultimo fetch, stato
  remoti, conflitti, working tree (stage/modificati/nuovi), conteggi rami/tag/stash,
  ultimo commit. Tessere cliccabili. Backend: comando unico `panoramica`.
  *Prossimo:* PR aperte, pipeline CI, issue collegate (richiedono integrazioni — Fase 13+).
- ✅ **Sidebar unificata** con navigazione a sezioni + contatori (Modifiche, Timeline)
  e rami/remoti/tag/stash *(v0.7)*.
- 🟦 Toolbar con icone (Cerca, Undo, Git Flow, Fetch/Pull/Push, ⚙). Rifinitura a icone
  piene in corso.
- 🟦 **Branch Manager**: elenco rami con icona e azioni (merge/rebase/elimina) + drag&drop.
  Ahead/behind e ultimo commit per ramo: da aggiungere.
- ✅ **Status bar** in basso (branch, ahead/behind, n. modifiche, vista) *(v0.7)*.

### Fase 8 — Drag & Drop totale  ✅
- ✅ Ramo-su-ramo → menu **Merge / Rebase** *(v0.7)*.
- ✅ Cherry-pick visuale commit→ramo con menu **Copy / Move / Squash** *(v0.8)*.
- ✅ **Rebase interattivo con drag&drop delle righe** + **fixup** *(v0.7-0.8)*.
- ✅ **Menu contestuale** ricco sul commit + **Compare with…** *(v0.7-0.8)*.

### Fase 9 — Working Tree ricca + stage intelligente  ✅
- ✅ Stato file esteso (Modified/Added/Deleted/Renamed/Untracked).
- ✅ Per file: **+/−**, **binary**, **lingua**, **barra proporzionale**, **size** *(v0.7-0.8)*.
- ✅ Stage per **file → hunk → singola riga** (come VSCode) *(v0.8)*.

### Fase 10 — Ricerca, filtri, storia  ✅
- ✅ **Ricerca globale (Ctrl+K)** *(v0.7)*.
- ✅ Filtri del grafo: testo + **autore** + **solo merge** *(v0.8)*.
- ✅ **Graph** paginato + **zoom** *(v0.8)*.
- ✅ **File History** e **Timeline** *(v0.7)*.
- ✅ **Hover card** ricca sul commit (SHA, autore, data, genitori, ref) *(v0.9)*.

### Fase 11 — Produttività  ✅
- ✅ **Undo** (Ctrl+Z) e **Redo** (Ctrl+Shift+Z) *(v0.7-0.9)*.
- ✅ **Diff Viewer**: side-by-side/unified + word diff, **anteprima Markdown** e
  **confronto immagini** *(v0.9)*. *(PDF/JSON/XML = eventuale futuro)*
- ✅ **Stash / Tag / Remote Manager** + **release notes** fra tag *(v0.8)*.
- ✅ **Terminale integrato** *(v0.7)*.
- ✅ **Custom Actions** (comandi salvati, eseguibili dal Terminale) *(v0.9)*.

### Fase 12 — Il "wow"  ✅
- ✅ **Commit Graph "Live"**: animazione d'ingresso + linee che si "disegnano"
  quando entra un nuovo commit *(v0.7-0.9)*.
- ✅ **Heat Map** sul grafo *(v0.7)*.
- ✅ Polish UI: transizioni, palette, micro-interazioni.

### Fase 13+ — Integrazioni ed ecosistema
- ✅ **Repository Insights** *(v0.7-0.8)*: contributori, attività, lingue, file caldi,
  righe totali.
- ✅ **Workspace** (gruppi di repository) *(v0.8)*.
- ✅ **Git Flow Assistant** *(v0.7)*.
- ✅ **AI (Anthropic)** *(v0.8)*: token nelle Impostazioni + genera messaggio di commit
  dal diff in stage (architettura pronta per aggiungere altri provider).
- ⏭️ **Pull Request**, **Jira**, **CI/pipeline**, **Plugin System** — *deprioritizzate
  dall'utente*: richiedono servizi esterni/credenziali. Lasciate fuori di proposito.

> **Nota**: le voci ⏭️ sono state escluse su richiesta. Tutto il resto della roadmap è
> implementato e funzionante in locale; restano solo rifiniture 🟦 (Redo, diff
> immagine/markdown, hover card, grafo live completo, custom actions salvabili).

---

## Sorpassi competitivi (dall'indagine SourceTree — Sonnet 5)

SourceTree nel 2025/26 è stagnante. Leve di sorpasso, molte già coperte:
1. Conflict resolver 3-way interno ✅ (v0.6) — SourceTree delega a tool esterni.
2. Grafo moderno con colori ✅ (v0.6).
3. Undo/Redo universale — Fase 11.
4. Prestazioni native su repo grandi — favorite dal core Rust.
5. Supporto Linux — già multipiattaforma (Tauri).
6. Nessun login obbligatorio — già così.
7. Custom Actions — Fase 11.
8. AI — Fase 13+.

**Anti-pattern da evitare** (da SourceTree): niente delega dei conflitti a tool esterni;
niente lentezza/architetture sincrone; niente login forzato; niente UI ferma per anni;
non ignorare Linux e la community; telemetria non intrusiva; non essere solo
mouse-centrico (scorciatoie complete).

---

## Da integrare

- ⏳ **Indagine #2 — software maggiori / Fork** (Sonnet 5, in corso): confronto Fork /
  GitKraken / Tower / GitHub Desktop / Sublime Merge / Lazygit e in dettaglio *come Fork
  implementa* commit visuali, cherry-pick drag&drop, merge viewer, cronologia moderna,
  conflitti. Aggiornerà priorità e dettagli di design di questa roadmap.
