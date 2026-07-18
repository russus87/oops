<script>
  // Mostra un diff: vista unificata (con evidenziazione delle parti cambiate
  // all'interno della riga) oppure affiancata (side-by-side). Con `onHunk` ogni
  // blocco mostra i pulsanti per agire su quel singolo hunk.
  import { stato } from "../lib/stato.svelte.js";
  import * as api from "../lib/api.js";
  import { estensione, ESTENSIONI_IMG, mimeDa, markdownToHtml } from "../lib/util.js";

  let {
    testo = "",
    vuoto = "Seleziona un file per vedere le differenze.",
    inStage = false,
    onHunk = null,
    onRighe = null, // callback(patch) per lo stage di righe selezionate
    percorsoFile = null, // percorso del file (abilita l'anteprima markdown/immagini)
  } = $props();

  let affiancato = $state(false);
  let anteprima = $state(false);

  // Tipo di file per l'anteprima.
  let ext = $derived(estensione(percorsoFile || ""));
  let isMd = $derived(ext === "md" || ext === "markdown");
  let isImg = $derived(ESTENSIONI_IMG.includes(ext));
  let anteprimaDisponibile = $derived(!!percorsoFile && (isMd || isImg));

  // Contenuti caricati per l'anteprima.
  let mdHtml = $state("");
  let imgPrima = $state("");
  let imgDopo = $state("");
  $effect(() => {
    if (!anteprima || !percorsoFile || !stato.percorso) return;
    if (isMd) {
      api.leggiTestoLavoro(stato.percorso, percorsoFile)
        .then((t) => (mdHtml = markdownToHtml(t)))
        .catch(() => (mdHtml = "<p>Impossibile leggere il file.</p>"));
    } else if (isImg) {
      const mime = mimeDa(percorsoFile);
      api.leggiB64Head(stato.percorso, percorsoFile).then((b) => (imgPrima = b ? `data:${mime};base64,${b}` : "")).catch(() => (imgPrima = ""));
      api.leggiB64Lavoro(stato.percorso, percorsoFile).then((b) => (imgDopo = b ? `data:${mime};base64,${b}` : "")).catch(() => (imgDopo = ""));
    }
  });
  // Se cambio file, esco dall'anteprima se non più disponibile.
  $effect(() => { if (!anteprimaDisponibile) anteprima = false; });
  let modoRighe = $state(false); // selezione per singola riga
  let selezione = $state(new Set()); // chiavi "h:i" delle righe scelte

  function toggleRiga(h, i) {
    const k = h + ":" + i;
    const s = new Set(selezione);
    s.has(k) ? s.delete(k) : s.add(k);
    selezione = s;
  }

  // Costruisce una patch parziale per un hunk, tenendo solo le righe scelte:
  // '+' non scelta = scartata; '-' non scelta = diventa contesto.
  function costruisciPatch(h) {
    const hunk = blocchi.hunk[h].righe;
    const m = hunk[0].match(/@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/);
    if (!m) return null;
    const oldStart = m[1], newStart = m[2];
    const body = [];
    let oldC = 0, newC = 0, cambi = 0;
    for (let i = 1; i < hunk.length; i++) {
      const l = hunk[i];
      if (l === undefined) continue;
      const c = l[0];
      const sel = selezione.has(h + ":" + i);
      if (c === "\\") { body.push(l); continue; }
      if (c === "+") {
        if (sel) { body.push(l); newC++; cambi++; }
      } else if (c === "-") {
        if (sel) { body.push(l); oldC++; cambi++; }
        else { body.push(" " + l.slice(1)); oldC++; newC++; }
      } else {
        body.push(l.length ? l : " "); oldC++; newC++;
      }
    }
    if (cambi === 0) return null;
    const testa = blocchi.preambolo.filter((r) => r.length > 0);
    const header = `@@ -${oldStart},${oldC} +${newStart},${newC} @@`;
    return [...testa, header, ...body, ""].join("\n");
  }

  function stageHunkRighe(h) {
    const patch = costruisciPatch(h);
    if (!patch) { return; }
    onRighe(patch);
    selezione = new Set();
  }

  function classe(riga) {
    if (riga.startsWith("+++") || riga.startsWith("---")) return "info";
    if (riga.startsWith("+")) return "agg";
    if (riga.startsWith("-")) return "rim";
    if (riga.startsWith("@@")) return "testa";
    if (riga.startsWith("diff ") || riga.startsWith("index ")) return "info";
    return "";
  }

  let blocchi = $derived(dividi(testo));

  function dividi(t) {
    const righe = t ? t.split("\n") : [];
    const preambolo = [];
    const hunk = [];
    let corrente = null;
    for (const r of righe) {
      if (r.startsWith("@@")) {
        corrente = { indice: hunk.length, righe: [r] };
        hunk.push(corrente);
      } else if (corrente) {
        corrente.righe.push(r);
      } else {
        preambolo.push(r);
      }
    }
    return { preambolo, hunk };
  }

  // Per ogni hunk produce le righe già pronte con i segmenti evidenziati.
  let hunkResi = $derived(blocchi.hunk.map((h) => ({ indice: h.indice, linee: evidenzia(h.righe) })));

  // Calcola, per ogni riga, la classe e i segmenti (parti cambiate marcate).
  function evidenzia(righe) {
    const out = [];
    let rim = [];
    let agg = [];
    const scarica = () => {
      // Accoppia le righe rimosse con quelle aggiunte e marca le differenze.
      const n = Math.max(rim.length, agg.length);
      for (let i = 0; i < n; i++) {
        const a = rim[i];
        const b = agg[i];
        if (a !== undefined && b !== undefined) {
          const [sa, sb] = diffParole(a.slice(1), b.slice(1));
          out.push({ classe: "rim", segmenti: [{ t: "-", m: false }, ...sa] });
          out.push({ classe: "agg", segmenti: [{ t: "+", m: false }, ...sb] });
        } else if (a !== undefined) {
          out.push({ classe: "rim", segmenti: [{ t: a, m: false }] });
        } else {
          out.push({ classe: "agg", segmenti: [{ t: b, m: false }] });
        }
      }
      rim = [];
      agg = [];
    };
    for (const r of righe) {
      if (r.startsWith("-")) rim.push(r);
      else if (r.startsWith("+")) agg.push(r);
      else {
        scarica();
        out.push({ classe: classe(r), segmenti: [{ t: r || " ", m: false }] });
      }
    }
    scarica();
    return out;
  }

  // Diff a parole tra due testi: ritorna i segmenti dei due lati, marcando le
  // parti diverse. Trova prefisso e suffisso comuni, marca il centro.
  function diffParole(a, b) {
    const ta = a.split(/(\s+)/);
    const tb = b.split(/(\s+)/);
    let i = 0;
    while (i < ta.length && i < tb.length && ta[i] === tb[i]) i++;
    let ja = ta.length, jb = tb.length;
    while (ja > i && jb > i && ta[ja - 1] === tb[jb - 1]) {
      ja--;
      jb--;
    }
    const seg = (tok, da, aa) => {
      const pre = tok.slice(0, da).join("");
      const mid = tok.slice(da, aa).join("");
      const post = tok.slice(aa).join("");
      const s = [];
      if (pre) s.push({ t: pre, m: false });
      if (mid) s.push({ t: mid, m: true });
      if (post) s.push({ t: post, m: false });
      return s.length ? s : [{ t: "", m: false }];
    };
    return [seg(ta, i, ja), seg(tb, i, jb)];
  }

  // Vista affiancata: accoppia righe rimosse (sinistra) e aggiunte (destra).
  let coppie = $derived(affiancato ? affianca(testo) : []);

  function affianca(t) {
    const righe = t ? t.split("\n") : [];
    const out = [];
    let rim = [];
    const scarica = () => {
      for (const r of rim) out.push({ s: r, d: null });
      rim = [];
    };
    for (const r of righe) {
      if (r.startsWith("diff ") || r.startsWith("index ") || r.startsWith("+++") || r.startsWith("---")) {
        scarica();
        out.push({ info: r });
      } else if (r.startsWith("@@")) {
        scarica();
        out.push({ testa: r });
      } else if (r.startsWith("-")) {
        rim.push(r);
      } else if (r.startsWith("+")) {
        if (rim.length > 0) out.push({ s: rim.shift(), d: r });
        else out.push({ s: null, d: r });
      } else {
        scarica();
        out.push({ s: r, d: r, ctx: true });
      }
    }
    scarica();
    return out;
  }
</script>

<div class="diff-contenitore">
{#if testo}
  <div class="diff-intestazione">
    {#if anteprimaDisponibile}
      <button class="fantasma" class:on={anteprima} onclick={() => (anteprima = !anteprima)}>
        {isImg ? "Anteprima immagine" : "Anteprima"}
      </button>
    {/if}
    {#if onRighe}
      <button class="fantasma" class:on={modoRighe} onclick={() => { modoRighe = !modoRighe; affiancato = false; }}>
        Per riga
      </button>
    {/if}
    <button class="fantasma" class:on={stato.ignoraSpazi} onclick={() => stato.cambiaIgnoraSpazi()}>
      Ignora spazi
    </button>
    <button class="fantasma" onclick={() => (affiancato = !affiancato)}>
      {affiancato ? "Vista unificata" : "Vista affiancata"}
    </button>
  </div>
  {#if anteprima && isMd}
    <div class="diff anteprima-md">{@html mdHtml}</div>
  {:else if anteprima && isImg}
    <div class="diff anteprima-img">
      <div class="ai-lato">
        <div class="ai-tit">Prima (HEAD)</div>
        {#if imgPrima}<img src={imgPrima} alt="prima" />{:else}<div class="ai-vuoto">assente</div>{/if}
      </div>
      <div class="ai-lato">
        <div class="ai-tit">Dopo (working)</div>
        {#if imgDopo}<img src={imgDopo} alt="dopo" />{:else}<div class="ai-vuoto">assente</div>{/if}
      </div>
    </div>
  {:else if affiancato}
    <div class="diff diff-split">
      <table>
        <tbody>
          {#each coppie as c}
            {#if c.info || c.testa}
              <tr><td class="riga {c.info ? 'info' : 'testa'}" colspan="2">{c.info || c.testa}</td></tr>
            {:else}
              <tr>
                <td class="riga {c.s && !c.ctx ? 'rim' : ''}">{c.s ?? ""}</td>
                <td class="riga {c.d && !c.ctx ? 'agg' : ''}">{c.d ?? ""}</td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
    </div>
  {:else if modoRighe && onRighe}
    <div class="diff diff-righe">
      <pre>{#each blocchi.preambolo as riga}<span class="riga {classe(riga)}">{riga || " "}</span>{/each}{#each blocchi.hunk as h, hi}<span class="hunk-barra"><span class="hb-tit">{h.righe[0]}</span><button onclick={() => stageHunkRighe(hi)}>+ Stage righe selezionate</button></span>{#each h.righe.slice(1) as l, idx}{@const i = idx + 1}{@const c = l ? l[0] : " "}<span class="riga {classe(l)}" class:selezionabile={c === "+" || c === "-"} class:selezionata={selezione.has(hi + ":" + i)} onclick={() => (c === "+" || c === "-") && toggleRiga(hi, i)}>{#if c === "+" || c === "-"}<span class="sel-box">{selezione.has(hi + ":" + i) ? "☑" : "☐"}</span>{/if}{l || " "}</span>{/each}{/each}</pre>
    </div>
  {:else}
    <div class="diff">
      <pre>{#each blocchi.preambolo as riga}<span class="riga {classe(riga)}">{riga || " "}</span>{/each}{#each hunkResi as h}{#if onHunk}<span class="hunk-barra"><button onclick={() => onHunk(h.indice, inStage ? "unstage" : "stage")}>{inStage ? "− Togli hunk" : "+ Stage hunk"}</button>{#if !inStage}<button class="pericolo" onclick={() => onHunk(h.indice, "scarta")}>↩ Scarta hunk</button>{/if}</span>{/if}{#each h.linee as l}<span class="riga {l.classe}">{#each l.segmenti as s}{#if s.m}<mark>{s.t}</mark>{:else}{s.t}{/if}{/each}</span>{/each}{/each}</pre>
    </div>
  {/if}
{:else}
  <div class="diff-vuoto">{vuoto}</div>
{/if}
</div>
