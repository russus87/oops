<script>
  // Modale con due viste per un file: Blame (chi ha toccato ogni riga) e
  // Cronologia (i commit che hanno modificato quel file).
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";

  let { file, chiudi } = $props();

  let vista = $state("blame"); // "blame" | "storia"
  let righe = $state([]);
  let commit = $state([]);

  $effect(() => {
    if (vista === "blame") {
      api.blame(stato.percorso, file).then((r) => (righe = r)).catch(() => (righe = []));
    } else {
      api.logFile(stato.percorso, file, 100).then((c) => (commit = c)).catch(() => (commit = []));
    }
  });
</script>

<div class="overlay" onclick={chiudi}>
  <div class="modale grande" onclick={(e) => e.stopPropagation()}>
    <div class="modale-testa">
      <h2>{file}</h2>
      <div class="tabs-mini">
        <button class:attivo={vista === "blame"} onclick={() => (vista = "blame")}>Blame</button>
        <button class:attivo={vista === "storia"} onclick={() => (vista = "storia")}>Cronologia</button>
      </div>
      <button class="fantasma" onclick={chiudi}>✕</button>
    </div>

    <div class="modale-corpo">
      {#if vista === "blame"}
        <table class="blame">
          <tbody>
            {#each righe as r}
              <tr>
                <td class="b-hash" title={r.autore}>{r.id_breve}</td>
                <td class="b-aut">{r.autore}</td>
                <td class="b-num">{r.numero}</td>
                <td class="b-testo">{r.testo || " "}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {:else}
        {#if commit.length === 0}
          <div class="lista-vuota">Nessun commit per questo file.</div>
        {/if}
        {#each commit as c}
          <div class="voce-commit">
            <div class="titolo">{c.titolo}</div>
            <div class="meta">
              <span class="hash">{c.id_breve}</span>
              <span>{c.autore}</span>
              <span>{c.data}</span>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>
