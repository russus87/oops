<script>
  // Vista "Modifiche": elenco file in stage e non, riquadro commit, e a destra
  // il diff del file selezionato.
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";
  import Diff from "./Diff.svelte";

  let s = $state(null); // StatoRepo dal backend
  let messaggio = $state("");
  let scelto = $state(null); // { file, inStage }
  let diffTesto = $state("");

  // Simboli per lo stato di un file.
  const simbolo = {
    nuovo: "A",
    modificato: "M",
    cancellato: "D",
    rinominato: "R",
    tipocambiato: "T",
    conflitto: "!",
  };

  // Ricarica lo stato quando cambia il repo o quando qualcosa lo richiede (tic).
  $effect(() => {
    stato.tic;
    if (!stato.percorso) return;
    api.stato(stato.percorso).then((nuovo) => {
      s = nuovo;
      // Se il file selezionato non esiste più, mostra il primo disponibile.
      aggiornaSelezione(nuovo);
    });
  });

  // Carica il diff del file selezionato.
  $effect(() => {
    if (!scelto || !stato.percorso) {
      diffTesto = "";
      return;
    }
    stato.tic;
    api
      .diffFile(stato.percorso, scelto.file, scelto.inStage)
      .then((t) => (diffTesto = t))
      .catch(() => (diffTesto = ""));
  });

  function aggiornaSelezione(nuovo) {
    const tutti = [...nuovo.in_stage, ...nuovo.non_in_stage];
    if (scelto && tutti.some((f) => f.percorso === scelto.file)) return;
    if (tutti.length > 0) {
      const primo = nuovo.non_in_stage[0] || nuovo.in_stage[0];
      scelto = { file: primo.percorso, inStage: primo.in_stage };
    } else {
      scelto = null;
    }
  }

  function seleziona(file, inStage) {
    scelto = { file, inStage };
  }

  async function add(file, ev) {
    ev.stopPropagation();
    await api.stageAggiungi(stato.percorso, file);
    stato.ricarica();
  }
  async function rimuovi(file, ev) {
    ev.stopPropagation();
    await api.stageTogli(stato.percorso, file);
    stato.ricarica();
  }
  async function scarta(file, ev) {
    ev.stopPropagation();
    await api.scarta(stato.percorso, file);
    stato.avvisa("Modifiche di " + file + " scartate.");
    stato.ricarica();
  }
  async function addTutto() {
    await api.stageAggiungiTutto(stato.percorso);
    stato.ricarica();
  }
  async function rimuoviTutto() {
    await api.stageTogliTutto(stato.percorso);
    stato.ricarica();
  }

  async function commit() {
    if (!messaggio.trim()) return;
    try {
      await api.creaCommit(stato.percorso, messaggio);
      messaggio = "";
      stato.avvisa("Commit creato 🎉", "ok");
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Commit fallito: " + e, "errore");
    }
  }
</script>

<div class="modifiche">
  <div class="pannello-modifiche">
    {#if s}
      <!-- File non in stage -->
      <div class="gruppo-file">
        <div class="intestazione">
          <span>Modifiche ({s.non_in_stage.length})</span>
          {#if s.non_in_stage.length > 0}
            <button class="fantasma" onclick={addTutto}>Aggiungi tutto</button>
          {/if}
        </div>
        <div class="lista">
          {#each s.non_in_stage as f}
            <div
              class="riga-file"
              class:scelto={scelto && scelto.file === f.percorso && !scelto.inStage}
              onclick={() => seleziona(f.percorso, false)}
            >
              <span class="stato {f.stato}">{simbolo[f.stato]}</span>
              <span class="nome">{f.percorso}</span>
              <span class="ops">
                <button title="Scarta" class="pericolo" onclick={(e) => scarta(f.percorso, e)}>↩</button>
                <button title="Aggiungi" onclick={(e) => add(f.percorso, e)}>+</button>
              </span>
            </div>
          {/each}
        </div>
      </div>

      <!-- File in stage -->
      <div class="gruppo-file">
        <div class="intestazione">
          <span>In stage ({s.in_stage.length})</span>
          {#if s.in_stage.length > 0}
            <button class="fantasma" onclick={rimuoviTutto}>Togli tutto</button>
          {/if}
        </div>
        <div class="lista">
          {#each s.in_stage as f}
            <div
              class="riga-file"
              class:scelto={scelto && scelto.file === f.percorso && scelto.inStage}
              onclick={() => seleziona(f.percorso, true)}
            >
              <span class="stato {f.stato}">{simbolo[f.stato]}</span>
              <span class="nome">{f.percorso}</span>
              <span class="ops">
                <button title="Togli dallo stage" onclick={(e) => rimuovi(f.percorso, e)}>−</button>
              </span>
            </div>
          {/each}
        </div>
      </div>

      <!-- Riquadro del commit -->
      <div class="commit-box">
        <textarea
          bind:value={messaggio}
          placeholder={s.in_stage.length ? "Messaggio del commit…" : "Aggiungi dei file per fare un commit"}
        ></textarea>
        <button
          class="primario"
          style="width:100%"
          disabled={!messaggio.trim() || s.in_stage.length === 0}
          onclick={commit}
        >
          Commit su {s.ramo}
        </button>
      </div>
    {/if}
  </div>

  <Diff testo={diffTesto} />
</div>
