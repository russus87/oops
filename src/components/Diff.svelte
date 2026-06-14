<script>
  // Mostra un diff: vista unificata (default) oppure affiancata (side-by-side).
  // Se è passato `onHunk`, ogni blocco (@@) mostra i pulsanti per agire su quel
  // singolo hunk (stage/unstage/scarta). Lo split è disattivato con onHunk.
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

  // Divide il testo in preambolo + hunk (con indice progressivo, come il backend).
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

  // Per la vista affiancata: accoppia righe rimosse (sinistra) e aggiunte (destra).
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
        // Se c'è una riga rimossa in attesa, le mettiamo sulla stessa riga.
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
      <pre>{#each blocchi.preambolo as riga}<span class="riga {classe(riga)}">{riga || " "}</span>
{/each}{#each blocchi.hunk as h}{#if onHunk}<span class="hunk-barra"><button onclick={() => onHunk(h.indice, inStage ? "unstage" : "stage")}>{inStage ? "− Togli hunk" : "+ Stage hunk"}</button>{#if !inStage}<button class="pericolo" onclick={() => onHunk(h.indice, "scarta")}>↩ Scarta hunk</button>{/if}</span>{/if}{#each h.righe as riga}<span class="riga {classe(riga)}">{riga || " "}</span>
{/each}{/each}</pre>
    </div>
  {/if}
{:else}
  <div class="diff-vuoto">{vuoto}</div>
{/if}
</div>
