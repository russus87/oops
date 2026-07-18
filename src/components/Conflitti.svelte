<script>
  // Pannello mostrato quando ci sono conflitti: per ogni file si sceglie quale
  // versione tenere, o lo si segna come risolto a mano. In alto si può annullare
  // l'intera operazione (merge/cherry-pick/rebase).
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";
  import MergeEditor from "./MergeEditor.svelte";

  let { file = [] } = $props();
  let editorFile = $state(null);

  async function risolvi(f, lato) {
    try {
      await api.conflittoRisolvi(stato.percorso, f, lato);
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Risoluzione fallita: " + e, "errore");
    }
  }

  async function risolto(f) {
    await api.conflittoSegnaRisolto(stato.percorso, f);
    stato.ricarica();
  }

  async function annulla() {
    try {
      await api.operazioneAnnulla(stato.percorso);
      stato.avvisa("Operazione annullata");
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Annullamento fallito: " + e, "errore");
    }
  }
</script>

<div class="conflitti">
  <div class="conflitti-testa">
    <span>⚠ {file.length} file in conflitto</span>
    <button class="pericolo" onclick={annulla}>Annulla operazione</button>
  </div>
  {#each file as f}
    <div class="conflitto-riga">
      <span class="nome">{f}</span>
      <div class="conflitto-ops">
        <button onclick={() => risolvi(f, "nostra")} title="Tieni la tua versione">Nostra</button>
        <button onclick={() => risolvi(f, "loro")} title="Tieni la versione in arrivo">Loro</button>
        <button onclick={() => (editorFile = f)} title="Editor a 3 vie">Editor</button>
        <button class="primario" onclick={() => risolto(f)} title="Ho risolto a mano">Risolto</button>
      </div>
    </div>
  {/each}
</div>

{#if editorFile}
  <MergeEditor file={editorFile} chiudi={() => (editorFile = null)} />
{/if}
