<script>
  // Vista "Cronologia": elenco commit a sinistra; a destra dettaglio del commit
  // (azioni + file toccati + diff del file scelto).
  import { confirm } from "@tauri-apps/plugin-dialog";
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";
  import Diff from "./Diff.svelte";

  let mostraRamo = $state(false);
  let nomeRamo = $state("");

  let commit = $state([]);
  let scelto = $state(null); // id del commit selezionato
  let file = $state([]); // file toccati dal commit
  let fileScelto = $state(null); // percorso del file mostrato (null = tutto il commit)
  let diffTesto = $state("");

  const simbolo = {
    nuovo: "A", modificato: "M", cancellato: "D",
    rinominato: "R", tipocambiato: "T", conflitto: "!",
  };

  // Carica la cronologia.
  $effect(() => {
    stato.tic;
    if (!stato.percorso) return;
    api.log(stato.percorso, 200).then((c) => {
      commit = c;
      if (c.length > 0 && !c.some((v) => v.id === scelto)) scelto = c[0].id;
    });
  });

  // Quando cambia il commit selezionato, ricarica i file e mostra tutto il diff.
  $effect(() => {
    if (!scelto || !stato.percorso) {
      file = [];
      return;
    }
    fileScelto = null;
    api.listaFileCommit(stato.percorso, scelto).then((f) => (file = f));
  });

  // Carica il diff: del file scelto, oppure dell'intero commit.
  $effect(() => {
    if (!scelto || !stato.percorso) {
      diffTesto = "";
      return;
    }
    const p = fileScelto
      ? api.diffCommitFile(stato.percorso, scelto, fileScelto)
      : api.diffCommit(stato.percorso, scelto);
    p.then((t) => (diffTesto = t)).catch(() => (diffTesto = ""));
  });

  async function reset(modo) {
    const msg =
      modo === "hard"
        ? "Reset HARD: le modifiche non salvate andranno PERSE. Procedere?"
        : "Spostare il ramo corrente a questo commit (" + modo + ")?";
    if (!(await confirm(msg))) return;
    try {
      await api.resetCommit(stato.percorso, scelto, modo);
      stato.avvisa("Reset " + modo + " eseguito", "ok");
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Reset fallito: " + e, "errore");
    }
  }

  async function cherry() {
    try {
      await api.cherryPick(stato.percorso, scelto);
      stato.avvisa("Cherry-pick eseguito 🍒", "ok");
      stato.ricarica();
    } catch (e) {
      stato.avvisa(String(e), "errore");
    }
  }

  async function revert() {
    if (!(await confirm("Creare un commit che annulla questo?"))) return;
    try {
      await api.revert(stato.percorso, scelto);
      stato.avvisa("Revert creato", "ok");
      stato.ricarica();
    } catch (e) {
      stato.avvisa(String(e), "errore");
    }
  }

  async function checkout() {
    if (!(await confirm("Spostarsi su questo commit (HEAD staccata)?"))) return;
    try {
      await api.ramoCheckoutCommit(stato.percorso, scelto);
      stato.avvisa("Ora su un commit (HEAD staccata)");
      stato.ricarica();
    } catch (e) {
      stato.avvisa(String(e), "errore");
    }
  }

  async function creaRamoDaQui() {
    if (!nomeRamo.trim()) return;
    try {
      await api.ramoCreaDa(stato.percorso, nomeRamo.trim(), scelto, true);
      mostraRamo = false;
      nomeRamo = "";
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Creazione ramo fallita: " + e, "errore");
    }
  }

  let datiScelto = $derived(commit.find((c) => c.id === scelto));
</script>

<div class="cronologia">
  <div class="lista-commit">
    {#if commit.length === 0}
      <div class="lista-vuota">Nessun commit ancora. Fanne uno! 🌱</div>
    {/if}
    {#each commit as c}
      <div class="voce-commit" class:scelto={scelto === c.id} onclick={() => (scelto = c.id)}>
        <div class="titolo">
          {#each c.riferimenti as r}<span class="deco">{r}</span>{/each}{c.titolo}
        </div>
        <div class="meta">
          <span class="hash">{c.id_breve}</span>
          {#if c.genitori.length > 1}<span class="merge">merge</span>{/if}
          <span>{c.autore}</span>
          <span>{c.data}</span>
        </div>
      </div>
    {/each}
  </div>

  <div class="commit-dettaglio">
    {#if datiScelto}
      <div class="azioni-commit">
        <span class="hash">{datiScelto.id_breve}</span>
        <span class="spazio"></span>
        <button onclick={() => (mostraRamo = true)} title="Crea un ramo da qui">⎇ Ramo</button>
        <button onclick={checkout} title="Spostati su questo commit">Checkout</button>
        <button onclick={cherry} title="Applica questo commit sul ramo corrente">🍒</button>
        <button onclick={revert} title="Annulla con un nuovo commit">↶ Revert</button>
        <span class="reset-label">Reset:</span>
        <button onclick={() => reset("soft")}>soft</button>
        <button onclick={() => reset("mixed")}>mixed</button>
        <button class="pericolo" onclick={() => reset("hard")}>hard</button>
      </div>

      <div class="file-commit">
        <div
          class="riga-file"
          class:scelto={fileScelto === null}
          onclick={() => (fileScelto = null)}
        >
          <span class="nome">Tutti i file ({file.length})</span>
        </div>
        {#each file as f}
          <div
            class="riga-file"
            class:scelto={fileScelto === f.percorso}
            onclick={() => (fileScelto = f.percorso)}
          >
            <span class="stato {f.stato}">{simbolo[f.stato]}</span>
            <span class="nome">{f.percorso}</span>
          </div>
        {/each}
      </div>

      <Diff testo={diffTesto} vuoto="Nessuna differenza." />
    {:else}
      <div class="diff-vuoto">Seleziona un commit.</div>
    {/if}
  </div>
</div>

{#if mostraRamo}
  <div class="overlay" onclick={() => (mostraRamo = false)}>
    <div class="modale" onclick={(e) => e.stopPropagation()}>
      <h2>Nuovo ramo da questo commit</h2>
      <div class="campo">
        <label for="nrq">Nome del ramo</label>
        <input id="nrq" bind:value={nomeRamo} placeholder="es. correzione/bug" />
      </div>
      <div class="pulsanti">
        <button onclick={() => (mostraRamo = false)}>Annulla</button>
        <button class="primario" onclick={creaRamoDaQui}>Crea e passa</button>
      </div>
    </div>
  </div>
{/if}
