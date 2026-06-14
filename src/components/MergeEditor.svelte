<script>
  // Editor di merge a 3 vie per un file in conflitto: mostra base/nostra/loro
  // (sola lettura) e un'area modificabile col risultato, poi salva e risolve.
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";

  let { file, chiudi } = $props();

  let v = $state(null);
  let risultato = $state("");

  $effect(() => {
    api.conflittoVersioni(stato.percorso, file).then((dati) => {
      v = dati;
      risultato = dati.corrente;
    });
  });

  async function salva() {
    try {
      await api.conflittoSalva(stato.percorso, file, risultato);
      stato.avvisa("Conflitto risolto: " + file, "ok");
      chiudi();
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Salvataggio fallito: " + e, "errore");
    }
  }

  // Copia una delle versioni nell'area del risultato.
  const usa = (testo) => (risultato = testo);
</script>

<div class="overlay" onclick={chiudi}>
  <div class="modale grande merge" onclick={(e) => e.stopPropagation()}>
    <div class="modale-testa">
      <h2>Merge: {file}</h2>
      <button class="fantasma" onclick={chiudi}>✕</button>
    </div>

    {#if v}
      <div class="merge-corpo">
        <div class="merge-tre">
          <div class="merge-col">
            <div class="merge-tit">
              <span>Nostra</span><button onclick={() => usa(v.nostra)}>Usa</button>
            </div>
            <pre>{v.nostra}</pre>
          </div>
          <div class="merge-col">
            <div class="merge-tit">
              <span>Base</span><button onclick={() => usa(v.base)}>Usa</button>
            </div>
            <pre>{v.base}</pre>
          </div>
          <div class="merge-col">
            <div class="merge-tit">
              <span>Loro</span><button onclick={() => usa(v.loro)}>Usa</button>
            </div>
            <pre>{v.loro}</pre>
          </div>
        </div>
        <div class="merge-risultato">
          <div class="merge-tit"><span>Risultato (modificabile)</span></div>
          <textarea bind:value={risultato}></textarea>
        </div>
      </div>
      <div class="pulsanti" style="padding:12px 18px">
        <button onclick={chiudi}>Annulla</button>
        <button class="primario" onclick={salva}>Salva e risolvi</button>
      </div>
    {/if}
  </div>
</div>
