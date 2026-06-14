<script>
  // Mostra un diff in formato testo unificato, colorando le righe.
  // Riceve già il testo pronto dal backend (vedi api.diffFile / api.diffCommit).
  let { testo = "", vuoto = "Seleziona un file per vedere le differenze." } = $props();

  // Assegna a ogni riga una classe in base al primo carattere (+/-/@@/diff).
  function classe(riga) {
    if (riga.startsWith("+++") || riga.startsWith("---")) return "info";
    if (riga.startsWith("+")) return "agg";
    if (riga.startsWith("-")) return "rim";
    if (riga.startsWith("@@")) return "testa";
    if (riga.startsWith("diff ") || riga.startsWith("index ")) return "info";
    return "";
  }

  let righe = $derived(testo ? testo.split("\n") : []);
</script>

{#if testo}
  <div class="diff">
    <pre>{#each righe as riga}<span class="riga {classe(riga)}">{riga || " "}</span>
{/each}</pre>
  </div>
{:else}
  <div class="diff-vuoto">{vuoto}</div>
{/if}
