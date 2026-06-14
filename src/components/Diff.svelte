<script>
  // Mostra un diff: vista unificata (con evidenziazione delle parti cambiate
  // all'interno della riga) oppure affiancata (side-by-side). Con `onHunk` ogni
  // blocco mostra i pulsanti per agire su quel singolo hunk.
  import { stato } from "../lib/stato.svelte.js";

  let {
    testo = "",
    vuoto = "Seleziona un file per vedere le differenze.",
    inStage = false,
    onHunk = null,
  } = $props();

  let affiancato = $state(false);

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
    <button class="fantasma" class:on={stato.ignoraSpazi} onclick={() => stato.cambiaIgnoraSpazi()}>
      Ignora spazi
    </button>
    <button class="fantasma" onclick={() => (affiancato = !affiancato)}>
      {affiancato ? "Vista unificata" : "Vista affiancata"}
    </button>
  </div>
  {#if affiancato}
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
  {:else}
    <div class="diff">
      <pre>{#each blocchi.preambolo as riga}<span class="riga {classe(riga)}">{riga || " "}</span>{/each}{#each hunkResi as h}{#if onHunk}<span class="hunk-barra"><button onclick={() => onHunk(h.indice, inStage ? "unstage" : "stage")}>{inStage ? "− Togli hunk" : "+ Stage hunk"}</button>{#if !inStage}<button class="pericolo" onclick={() => onHunk(h.indice, "scarta")}>↩ Scarta hunk</button>{/if}</span>{/if}{#each h.linee as l}<span class="riga {l.classe}">{#each l.segmenti as s}{#if s.m}<mark>{s.t}</mark>{:else}{s.t}{/if}{/each}</span>{/each}{/each}</pre>
    </div>
  {/if}
{:else}
  <div class="diff-vuoto">{vuoto}</div>
{/if}
</div>
