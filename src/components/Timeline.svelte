<script>
  // Timeline: cosa hai fatto in questa sessione (fetch, commit, push, checkout…),
  // in ordine cronologico inverso. Serve a capire "cosa è successo". È in memoria:
  // si azzera alla chiusura del repository.
  import { stato } from "../lib/stato.svelte.js";
</script>

<div class="timeline-vista">
  <div class="tl-testa">
    <span>Timeline della sessione</span>
    {#if stato.azioni.length > 0}
      <button class="fantasma" onclick={() => (stato.azioni = [])}>Svuota</button>
    {/if}
  </div>
  {#if stato.azioni.length === 0}
    <div class="diff-vuoto">Nessuna azione ancora. Le operazioni compariranno qui. ⏱</div>
  {:else}
    <div class="tl-lista">
      {#each stato.azioni as a}
        <div class="tl-riga {a.tipo}">
          <span class="tl-ora">{a.ora}</span>
          <span class="tl-punto"></span>
          <span class="tl-testo">{a.testo}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>
