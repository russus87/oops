<script>
  // Vista "Cronologia": elenco dei commit a sinistra, diff del commit a destra.
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";
  import Diff from "./Diff.svelte";

  let commit = $state([]);
  let scelto = $state(null); // id del commit selezionato
  let diffTesto = $state("");

  // Carica la cronologia quando cambia il repo o si richiede un ricarico.
  $effect(() => {
    stato.tic;
    if (!stato.percorso) return;
    api.log(stato.percorso, 200).then((c) => {
      commit = c;
      if (c.length > 0 && !c.some((v) => v.id === scelto)) scelto = c[0].id;
    });
  });

  // Carica il diff del commit selezionato.
  $effect(() => {
    if (!scelto || !stato.percorso) {
      diffTesto = "";
      return;
    }
    api
      .diffCommit(stato.percorso, scelto)
      .then((t) => (diffTesto = t))
      .catch(() => (diffTesto = ""));
  });
</script>

<div class="cronologia">
  <div class="lista-commit">
    {#if commit.length === 0}
      <div class="lista-vuota">Nessun commit ancora. Fanne uno! 🌱</div>
    {/if}
    {#each commit as c}
      <div class="voce-commit" class:scelto={scelto === c.id} onclick={() => (scelto = c.id)}>
        <div class="titolo">{c.titolo}</div>
        <div class="meta">
          <span class="hash">{c.id_breve}</span>
          {#if c.genitori.length > 1}<span class="merge">merge</span>{/if}
          <span>{c.autore}</span>
          <span>{c.data}</span>
        </div>
      </div>
    {/each}
  </div>

  <Diff testo={diffTesto} vuoto="Seleziona un commit per vederne le modifiche." />
</div>
