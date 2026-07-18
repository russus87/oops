<script>
  // Vista "Cronologia": a sinistra il grafo dei commit (corsie colorate con
  // curve, avatar, badge dei riferimenti, tempo relativo); a destra il dettaglio
  // del commit (azioni + file toccati + diff). I commit si possono trascinare
  // su un ramo nella barra laterale per fare cherry-pick (drag&drop).
  import { confirm, save } from "@tauri-apps/plugin-dialog";
  import { fly } from "svelte/transition";
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";
  import { calcolaGrafo, iniziali, coloreAvatar, tempoRelativo, coloreLingua, estensione } from "../lib/util.js";
  import Diff from "./Diff.svelte";
  import RebaseInterattivo from "./RebaseInterattivo.svelte";
  import BarraStat from "./BarraStat.svelte";

  let statFile = $state({}); // percorso -> stat del commit selezionato

  let mostraRamo = $state(false);
  let nomeRamo = $state("");
  let mostraCondensa = $state(false);
  let msgCondensa = $state("");
  let rebaseInt = $state(null); // { base, commits } quando aperto

  let commit = $state([]);
  let scelto = $state(null); // id del commit selezionato
  let file = $state([]); // file toccati dal commit
  let fileScelto = $state(null); // percorso del file mostrato (null = tutto il commit)
  let diffTesto = $state("");
  let filtro = $state(""); // testo di ricerca nella cronologia
  let limite = $state(100); // quanti commit caricare
  let genitoreDiff = $state(0); // per i merge: con quale genitore confrontare

  const simbolo = {
    nuovo: "A", modificato: "M", cancellato: "D",
    rinominato: "R", tipocambiato: "T", conflitto: "!",
  };

  // Icona per tipo di riferimento (badge nel grafo).
  const iconaRef = { testa: "◆", locale: "⎇", remoto: "☁", tag: "🏷" };

  // Carica la cronologia (ricaricata anche quando cambia il limite).
  $effect(() => {
    stato.tic;
    const lim = limite;
    if (!stato.percorso) return;
    api.log(stato.percorso, lim).then((c) => {
      commit = c;
      if (c.length > 0 && !c.some((v) => v.id === scelto)) scelto = c[0].id;
    });
  });

  // --- Grafo a corsie con curve (best-effort, solo senza filtro) ---
  let zoom = $state(1);            // livello di zoom della cronologia
  let ROW = $derived(48 * zoom);  // altezza di una riga (px)
  const COLW = 15;  // larghezza di una corsia (px)
  const PAD = 12;   // margine sinistro (px)

  let filtroAutore = $state("");   // filtro per autore ("" = tutti)
  let soloMerge = $state(false);   // mostra solo i commit di merge
  let autori = $derived([...new Set(commit.map((c) => c.autore))].sort());

  let compareA = $state(null);     // { id, breve } marcato per il confronto
  let confronto = $state(null);    // { a, b, aBreve, bBreve } confronto attivo

  let grafo = $derived(filtriAttivi ? [] : calcolaGrafo(commit));
  let maxLarghezza = $derived(grafo.reduce((m, g) => Math.max(m, g.larghezza), 1));
  let larghezzaGrafo = $derived(maxLarghezza * COLW + PAD * 2);

  // Converte un segmento normalizzato (x = colonna, y = 0..1) in un path SVG:
  // linea dritta per le corsie verticali, curva morbida per le diagonali.
  function pathSegmento(s) {
    const x1 = PAD + s.x1 * COLW, x2 = PAD + s.x2 * COLW;
    const y1 = s.y1 * ROW, y2 = s.y2 * ROW;
    if (x1 === x2) return `M${x1},${y1} L${x2},${y2}`;
    const ym = (y1 + y2) / 2;
    return `M${x1},${y1} C${x1},${ym} ${x2},${ym} ${x2},${y2}`;
  }
  const cx = (col) => PAD + col * COLW;

  // Filtro combinato: testo (messaggio/autore/hash) + autore + solo merge.
  let commitFiltrati = $derived(
    commit.filter((c) => {
      if (filtroAutore && c.autore !== filtroAutore) return false;
      if (soloMerge && c.genitori.length < 2) return false;
      if (filtro.trim()) {
        const q = filtro.toLowerCase();
        if (!(c.titolo.toLowerCase().includes(q) || c.autore.toLowerCase().includes(q) || c.id_breve.includes(q)))
          return false;
      }
      return true;
    })
  );
  // Il grafo si disegna solo quando non ci sono filtri attivi (indici allineati).
  let filtriAttivi = $derived(!!filtro.trim() || !!filtroAutore || soloMerge);

  // Quando cambia il commit selezionato, ricarica i file e azzera lo stato.
  $effect(() => {
    if (!scelto || !stato.percorso) {
      file = [];
      return;
    }
    fileScelto = null;
    genitoreDiff = 0;
    api.listaFileCommit(stato.percorso, scelto).then((f) => (file = f));
    api.statCommit(stato.percorso, scelto)
      .then((v) => (statFile = Object.fromEntries(v.map((x) => [x.percorso, x]))))
      .catch(() => (statFile = {}));
  });

  // Carica il diff: del file scelto, oppure dell'intero commit (rispetto al
  // genitore selezionato, utile per i merge).
  $effect(() => {
    if (!scelto || !stato.percorso) {
      diffTesto = "";
      return;
    }
    stato.tic;
    const p = confronto
      ? api.diffTraCommit(stato.percorso, confronto.a, confronto.b, stato.ignoraSpazi)
      : fileScelto
        ? api.diffCommitFile(stato.percorso, scelto, fileScelto, stato.ignoraSpazi)
        : api.diffCommitGenitore(stato.percorso, scelto, genitoreDiff, stato.ignoraSpazi);
    p.then((t) => (diffTesto = t)).catch(() => (diffTesto = ""));
  });

  // --- Drag&drop: trascina un commit su un ramo per il cherry-pick ---
  function iniziaTrascina(e, c) {
    stato.trascina = { tipo: "commit", id: c.id, breve: c.id_breve };
    e.dataTransfer.effectAllowed = "copy";
    e.dataTransfer.setData("application/x-oops-commit", c.id);
  }

  // Salto a un commit richiesto dalla ricerca globale.
  $effect(() => {
    const id = stato.commitScelto;
    if (id && commit.some((c) => c.id === id)) {
      scelto = id;
      stato.commitScelto = null;
      // porta in vista la voce selezionata
      queueMicrotask(() => document.querySelector(".voce-commit.scelto")?.scrollIntoView({ block: "center" }));
    }
  });

  // --- Heat map: colora i nodi per quantità di modifiche (churn) ---
  let calore = $state({}); // id -> churn
  let maxCalore = $derived(Math.max(1, ...Object.values(calore)));
  $effect(() => {
    stato.tic;
    if (!stato.heatMap || !stato.percorso) {
      calore = {};
      return;
    }
    api.calore(stato.percorso, limite).then((v) => {
      const m = {};
      for (const c of v) m[c.id] = c.churn;
      calore = m;
    }).catch(() => (calore = {}));
  });

  // Colore "caldo" da verde (poco) a rosso (tanto), scala logaritmica.
  function coloreCalore(id) {
    const v = calore[id] ?? 0;
    const t = Math.min(1, Math.log2(1 + v) / Math.log2(1 + maxCalore));
    const h = 120 - Math.round(t * 120); // 120=verde → 0=rosso
    return `hsl(${h}, 70%, 55%)`;
  }

  // --- Menu contestuale (clic destro su un commit) ---
  let menu = $state(null); // { x, y, id }
  function apriMenu(e, c) {
    e.preventDefault();
    scelto = c.id;
    menu = { x: e.clientX, y: e.clientY, id: c.id };
  }
  function chiudiMenu() { menu = null; }
  function azioneMenu(fn) { menu = null; fn(); }

  // --- Hover card ricca sul commit ---
  let hover = $state(null); // { c, x, y }
  let timerHover;
  function entra(e, c) {
    clearTimeout(timerHover);
    const r = e.currentTarget.getBoundingClientRect();
    const x = r.right + 8, y = Math.max(8, r.top);
    timerHover = setTimeout(() => (hover = { c, x, y }), 350);
  }
  function esci() {
    clearTimeout(timerHover);
    hover = null;
  }

  function segnaConfronto() {
    compareA = { id: scelto, breve: datiScelto?.id_breve };
    stato.avvisa("Segnato " + datiScelto?.id_breve + " per il confronto");
  }
  function confrontaConMarcato() {
    if (!compareA) return;
    confronto = { a: compareA.id, b: scelto, aBreve: compareA.breve, bBreve: datiScelto?.id_breve };
    fileScelto = null;
  }

  async function reset(modo) {
    const msg =
      modo === "hard"
        ? "Reset HARD: le modifiche non salvate andranno PERSE. Procedere?"
        : "Spostare il ramo corrente a questo commit (" + modo + ")?";
    if (!(await confirm(msg))) return;
    try {
      await api.resetCommit(stato.percorso, scelto, modo);
      stato.avvisa("Reset " + modo + " eseguito", "ok");
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Reset fallito: " + e, "errore");
    }
  }

  async function cherry() {
    try {
      await api.cherryPick(stato.percorso, scelto);
      stato.avvisa("Cherry-pick eseguito 🍒", "ok");
      stato.ricarica();
    } catch (e) {
      stato.avvisa(String(e), "errore");
    }
  }

  async function revert() {
    if (!(await confirm("Creare un commit che annulla questo?"))) return;
    try {
      await api.revert(stato.percorso, scelto);
      stato.avvisa("Revert creato", "ok");
      stato.ricarica();
    } catch (e) {
      stato.avvisa(String(e), "errore");
    }
  }

  async function checkout() {
    if (!(await confirm("Spostarsi su questo commit (HEAD staccata)?"))) return;
    try {
      await api.ramoCheckoutCommit(stato.percorso, scelto);
      stato.avvisa("Ora su un commit (HEAD staccata)");
      stato.ricarica();
    } catch (e) {
      stato.avvisa(String(e), "errore");
    }
  }

  async function ripristina(f) {
    if (!(await confirm("Ripristinare «" + f + "» alla versione di questo commit?"))) return;
    try {
      await api.ripristinaFile(stato.percorso, scelto, f);
      stato.avvisa("File ripristinato (in stage)", "ok");
      stato.ricarica();
    } catch (e) {
      stato.avvisa(String(e), "errore");
    }
  }

  async function condensa() {
    if (!msgCondensa.trim()) return;
    try {
      await api.condensa(stato.percorso, scelto, msgCondensa);
      mostraCondensa = false;
      msgCondensa = "";
      stato.avvisa("Commit condensati 🗜", "ok");
      stato.ricarica();
    } catch (e) {
      stato.avvisa(String(e), "errore");
    }
  }

  async function copiaHash() {
    try {
      await navigator.clipboard.writeText(scelto);
      stato.avvisa("Hash copiato");
    } catch {
      stato.avvisa("Copia non riuscita", "errore");
    }
  }

  async function esportaPatch() {
    const f = await save({
      defaultPath: (datiScelto?.id_breve || "commit") + ".patch",
      title: "Esporta patch",
    });
    if (!f) return;
    try {
      await api.patchEsporta(stato.percorso, scelto, f);
      stato.avvisa("Patch esportata", "ok");
    } catch (e) {
      stato.avvisa(String(e), "errore");
    }
  }

  function apriRebaseInt() {
    const idx = commit.findIndex((c) => c.id === scelto);
    // I commit "più nuovi" del selezionato, ordinati dal più vecchio al più recente.
    const range = commit.slice(0, idx).reverse();
    if (range.length === 0) {
      stato.avvisa("Non ci sono commit dopo questo da riscrivere", "errore");
      return;
    }
    rebaseInt = { base: scelto, commits: range };
  }

  async function creaRamoDaQui() {
    if (!nomeRamo.trim()) return;
    try {
      await api.ramoCreaDa(stato.percorso, nomeRamo.trim(), scelto, true);
      mostraRamo = false;
      nomeRamo = "";
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Creazione ramo fallita: " + e, "errore");
    }
  }

  let datiScelto = $derived(commit.find((c) => c.id === scelto));
</script>

<div class="cronologia">
  <div class="lista-commit">
    <div class="cerca-commit">
      <input bind:value={filtro} placeholder="Cerca…" />
      <button class="fantasma heat-toggle" class:on={stato.heatMap}
        title="Heat map: colora i commit per quantità di modifiche"
        onclick={() => (stato.heatMap = !stato.heatMap)}>🔥</button>
    </div>
    <div class="cronologia-filtri">
      <select bind:value={filtroAutore} title="Filtra per autore">
        <option value="">Tutti gli autori</option>
        {#each autori as a}<option value={a}>{a}</option>{/each}
      </select>
      <button class="fantasma" class:on={soloMerge} title="Solo commit di merge" onclick={() => (soloMerge = !soloMerge)}>⑃ merge</button>
      <span class="spazio-flex"></span>
      <button class="fantasma" title="Riduci" onclick={() => (zoom = Math.max(0.7, +(zoom - 0.15).toFixed(2)))}>−</button>
      <span class="zoom-eti">{Math.round(zoom * 100)}%</span>
      <button class="fantasma" title="Ingrandisci" onclick={() => (zoom = Math.min(1.6, +(zoom + 0.15).toFixed(2)))}>+</button>
    </div>
    {#if commitFiltrati.length === 0}
      <div class="lista-vuota">
        {commit.length === 0 ? "Nessun commit ancora. Fanne uno! 🌱" : "Nessun commit corrisponde."}
      </div>
    {/if}
    {#each commitFiltrati as c, i (c.id)}
      <div
        class="voce-commit"
        class:scelto={scelto === c.id}
        class:con-grafo={grafo.length > 0}
        style={grafo.length > 0 ? `height:${ROW}px` : ""}
        in:fly={{ y: -10, duration: 220 }}
        draggable="true"
        ondragstart={(e) => iniziaTrascina(e, c)}
        ondragend={() => (stato.trascina = null)}
        onclick={() => (scelto = c.id)}
        oncontextmenu={(e) => apriMenu(e, c)}
        onmouseenter={(e) => entra(e, c)}
        onmouseleave={esci}
      >
        {#if grafo[i]}
          <svg class="grafo" width={larghezzaGrafo} height={ROW} style="flex:0 0 {larghezzaGrafo}px">
            {#each grafo[i].segmenti as s}
              <path d={pathSegmento(s)} stroke={s.colore} stroke-width="2" fill="none" opacity="0.85" />
            {/each}
            {#if stato.heatMap}
              <circle cx={cx(grafo[i].col)} cy={ROW / 2} r="6"
                fill={coloreCalore(c.id)} stroke="var(--sfondo)" stroke-width="1.5" />
            {:else}
              <circle cx={cx(grafo[i].col)} cy={ROW / 2} r={grafo[i].merge ? 5.5 : 4.5}
                fill={grafo[i].merge ? "var(--sfondo)" : grafo[i].colore}
                stroke={grafo[i].colore} stroke-width="2.5" />
            {/if}
          </svg>
        {/if}
        <div
          class="avatar"
          style="background:{coloreAvatar(c.email || c.autore)}"
          title="{c.autore} <{c.email}>"
        >{iniziali(c.autore)}</div>
        <div class="voce-corpo">
          <div class="titolo">
            {#each c.decori as r}
              <span class="badge-ref {r.tipo}" title={r.tipo}>{iconaRef[r.tipo] || ""} {r.nome}</span>
            {/each}
            {c.titolo}
          </div>
          <div class="meta">
            <span class="hash">{c.id_breve}</span>
            {#if c.genitori.length > 1}<span class="merge">merge</span>{/if}
            <span class="autore">{c.autore}</span>
            <span title={c.data}>{tempoRelativo(c.timestamp)}</span>
          </div>
        </div>
      </div>
    {/each}
    {#if commit.length >= limite}
      <button class="fantasma carica-altri" onclick={() => (limite += 200)}>Carica altri commit</button>
    {/if}
  </div>

  <div class="commit-dettaglio">
    {#if confronto}
      <div class="confronto-banner">
        <span>⇄ Confronto <b>{confronto.aBreve}</b> → <b>{confronto.bBreve}</b></span>
        <span class="spazio-flex"></span>
        <button class="fantasma" onclick={() => (confronto = null)}>✕ Chiudi confronto</button>
      </div>
    {/if}
    {#if datiScelto}
      <div class="azioni-commit">
        <span class="hash">{datiScelto.id_breve}</span>
        <span class="spazio"></span>
        <button onclick={() => (mostraRamo = true)} title="Crea un ramo da qui">⎇ Ramo</button>
        <button onclick={checkout} title="Spostati su questo commit">Checkout</button>
        <button onclick={cherry} title="Applica questo commit sul ramo corrente">🍒</button>
        <button onclick={revert} title="Annulla con un nuovo commit">↶ Revert</button>
        <button onclick={() => (mostraCondensa = true)} title="Condensa da qui fino a HEAD">🗜 Condensa</button>
        <span class="reset-label">Reset:</span>
        <button onclick={() => reset("soft")}>soft</button>
        <button onclick={() => reset("mixed")}>mixed</button>
        <button class="pericolo" onclick={() => reset("hard")}>hard</button>
        <button onclick={apriRebaseInt} title="Riscrivi i commit dopo questo">↻ Rebase i.</button>
        <button onclick={esportaPatch} title="Esporta come .patch">🗂</button>
        <button onclick={copiaHash} title="Copia l'hash">⧉</button>
      </div>

      {#if datiScelto.genitori.length > 1}
        <div class="merge-viewer">
          <span>Commit di merge — confronta con:</span>
          {#each datiScelto.genitori as g, gi}
            <button
              class:on={genitoreDiff === gi}
              onclick={() => { fileScelto = null; genitoreDiff = gi; }}
            >Genitore {gi + 1} ({g})</button>
          {/each}
        </div>
      {/if}

      <div class="file-commit">
        <div
          class="riga-file"
          class:scelto={fileScelto === null}
          onclick={() => (fileScelto = null)}
        >
          <span class="nome">Tutti i file ({file.length})</span>
        </div>
        {#each file as f}
          <div
            class="riga-file"
            class:scelto={fileScelto === f.percorso}
            onclick={() => (fileScelto = f.percorso)}
          >
            <span class="stato {f.stato}">{simbolo[f.stato]}</span>
            <span class="lang-dot" style="background:{coloreLingua(f.percorso)}" title={estensione(f.percorso) || "file"}></span>
            <span class="nome">{f.percorso}</span>
            <BarraStat stat={statFile[f.percorso]} />
            <span class="ops">
              <button title="Cronologia / blame del file" onclick={(e) => { e.stopPropagation(); stato.storiaFile = f.percorso; }}>📜</button>
              <button title="Ripristina questo file a questa versione" onclick={(e) => { e.stopPropagation(); ripristina(f.percorso); }}>↺</button>
            </span>
          </div>
        {/each}
      </div>

      <Diff testo={diffTesto} vuoto="Nessuna differenza." />
    {:else}
      <div class="diff-vuoto">Seleziona un commit.</div>
    {/if}
  </div>
</div>

{#if hover}
  <div class="hover-card" style="left:{hover.x}px; top:{hover.y}px">
    <div class="hc-titolo">{hover.c.titolo}</div>
    <div class="hc-riga"><span class="hc-k">SHA</span><span class="hash">{hover.c.id_breve}</span></div>
    <div class="hc-riga"><span class="hc-k">Autore</span>{hover.c.autore}{hover.c.email ? " <" + hover.c.email + ">" : ""}</div>
    <div class="hc-riga"><span class="hc-k">Data</span>{hover.c.data} · {tempoRelativo(hover.c.timestamp)}</div>
    <div class="hc-riga"><span class="hc-k">Genitori</span>{hover.c.genitori.join(", ") || "—"}{hover.c.genitori.length > 1 ? " (merge)" : ""}</div>
    {#if hover.c.decori.length > 0}
      <div class="hc-badges">
        {#each hover.c.decori as r}<span class="badge-ref {r.tipo}">{iconaRef[r.tipo] || ""} {r.nome}</span>{/each}
      </div>
    {/if}
  </div>
{/if}

{#if menu}
  <div class="menu-ctx-overlay" onclick={chiudiMenu} oncontextmenu={(e) => { e.preventDefault(); chiudiMenu(); }}></div>
  <div class="menu-ctx" style="left:{menu.x}px; top:{menu.y}px">
    <button onclick={() => azioneMenu(checkout)}>Checkout</button>
    <button onclick={() => azioneMenu(cherry)}>🍒 Cherry-pick</button>
    <button onclick={() => azioneMenu(revert)}>↶ Revert</button>
    <div class="mc-sep"></div>
    <button onclick={() => azioneMenu(() => reset("soft"))}>Reset soft</button>
    <button onclick={() => azioneMenu(() => reset("mixed"))}>Reset mixed</button>
    <button class="pericolo" onclick={() => azioneMenu(() => reset("hard"))}>Reset hard</button>
    <div class="mc-sep"></div>
    <button onclick={() => azioneMenu(() => (mostraRamo = true))}>⎇ Crea ramo da qui</button>
    <button onclick={() => azioneMenu(apriRebaseInt)}>↻ Rebase interattivo</button>
    <button onclick={() => azioneMenu(() => (mostraCondensa = true))}>🗜 Condensa</button>
    <div class="mc-sep"></div>
    <button onclick={() => azioneMenu(esportaPatch)}>🗂 Esporta patch</button>
    <button onclick={() => azioneMenu(copiaHash)}>⧉ Copia SHA</button>
    <div class="mc-sep"></div>
    <button onclick={() => azioneMenu(segnaConfronto)}>⇄ Segna per confronto</button>
    {#if compareA && compareA.id !== scelto}
      <button onclick={() => azioneMenu(confrontaConMarcato)}>⇄ Confronta con {compareA.breve}</button>
    {/if}
  </div>
{/if}

{#if mostraRamo}
  <div class="overlay" onclick={() => (mostraRamo = false)}>
    <div class="modale" onclick={(e) => e.stopPropagation()}>
      <h2>Nuovo ramo da questo commit</h2>
      <div class="campo">
        <label for="nrq">Nome del ramo</label>
        <input id="nrq" bind:value={nomeRamo} placeholder="es. correzione/bug" />
      </div>
      <div class="pulsanti">
        <button onclick={() => (mostraRamo = false)}>Annulla</button>
        <button class="primario" onclick={creaRamoDaQui}>Crea e passa</button>
      </div>
    </div>
  </div>
{/if}

{#if rebaseInt}
  <RebaseInterattivo
    base={rebaseInt.base}
    commits={rebaseInt.commits}
    chiudi={() => (rebaseInt = null)}
  />
{/if}

{#if mostraCondensa}
  <div class="overlay" onclick={() => (mostraCondensa = false)}>
    <div class="modale" onclick={(e) => e.stopPropagation()}>
      <h2>Condensa commit</h2>
      <p style="color:var(--testo2);font-size:12px;margin-top:0">
        Tutti i commit da quello selezionato fino all'ultimo verranno uniti in uno solo.
      </p>
      <div class="campo">
        <label for="mc">Messaggio del commit unico</label>
        <textarea id="mc" bind:value={msgCondensa} style="min-height:70px"></textarea>
      </div>
      <div class="pulsanti">
        <button onclick={() => (mostraCondensa = false)}>Annulla</button>
        <button class="primario" onclick={condensa}>Condensa</button>
      </div>
    </div>
  </div>
{/if}
