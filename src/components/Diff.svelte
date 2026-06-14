<script>
  // Mostra un diff in formato testo unificato, colorando le righe.
  // Se è passato `onHunk`, ogni blocco (@@) mostra dei pulsanti per agire
  // su quel singolo hunk (stage/unstage/scarta).
  let {
    testo = "",
    vuoto = "Seleziona un file per vedere le differenze.",
    inStage = false,
    onHunk = null,
  } = $props();

  // Assegna a ogni riga una classe in base al primo carattere.
  function classe(riga) {
    if (riga.startsWith("+++") || riga.startsWith("---")) return "info";
    if (riga.startsWith("+")) return "agg";
    if (riga.startsWith("-")) return "rim";
    if (riga.startsWith("@@")) return "testa";
    if (riga.startsWith("diff ") || riga.startsWith("index ")) return "info";
    return "";
  }

  // Divide il testo in: preambolo (intestazione del file) + lista di hunk.
  // Ogni hunk ha un indice progressivo, lo stesso usato dal backend.
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
</script>

{#if testo}
  <div class="diff">
    <pre>{#each blocchi.preambolo as riga}<span class="riga {classe(riga)}">{riga || " "}</span>
{/each}{#each blocchi.hunk as h}{#if onHunk}<span class="hunk-barra"><button onclick={() => onHunk(h.indice, inStage ? "unstage" : "stage")}>{inStage ? "− Togli hunk" : "+ Stage hunk"}</button>{#if !inStage}<button class="pericolo" onclick={() => onHunk(h.indice, "scarta")}>↩ Scarta hunk</button>{/if}</span>{/if}{#each h.righe as riga}<span class="riga {classe(riga)}">{riga || " "}</span>
{/each}{/each}</pre>
  </div>
{:else}
  <div class="diff-vuoto">{vuoto}</div>
{/if}
