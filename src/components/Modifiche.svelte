<script>
  // Vista "Modifiche": elenco file in stage e non, riquadro commit, e a destra
  // il diff del file selezionato.
  import { confirm } from "@tauri-apps/plugin-dialog";
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";
  import Diff from "./Diff.svelte";
  import Conflitti from "./Conflitti.svelte";
  import Blame from "./Blame.svelte";

  let s = $state(null); // StatoRepo dal backend
  let conflitti = $state([]); // file in conflitto
  let messaggio = $state("");
  let scelto = $state(null); // { file, inStage }
  let diffTesto = $state("");
  let amendOn = $state(false); // se true, il commit corregge l'ultimo
  let blameFile = $state(null); // file aperto nel pannello Blame

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
    api.conflittiLista(stato.percorso).then((c) => (conflitti = c)).catch(() => (conflitti = []));
  });

  // Carica il diff del file selezionato.
  $effect(() => {
    if (!scelto || !stato.percorso) {
      diffTesto = "";
      return;
    }
    stato.tic;
    api
      .diffFile(stato.percorso, scelto.file, scelto.inStage, stato.ignoraSpazi)
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
      if (amendOn) {
        await api.amend(stato.percorso, messaggio);
        stato.avvisa("Ultimo commit corretto ✏️", "ok");
      } else {
        await api.creaCommit(stato.percorso, messaggio);
        stato.avvisa("Commit creato 🎉", "ok");
      }
      messaggio = "";
      amendOn = false;
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Commit fallito: " + e, "errore");
    }
  }

  // Attiva/disattiva amend: quando si attiva, precompila col vecchio messaggio.
  async function toggleAmend() {
    amendOn = !amendOn;
    if (amendOn && !messaggio.trim()) {
      messaggio = await api.ultimoMessaggio(stato.percorso).catch(() => "");
    }
  }

  // Mette da parte tutte le modifiche correnti (stash).
  async function stash() {
    try {
      await api.stashSalva(stato.percorso, "", true);
      stato.avvisa("Modifiche messe da parte 📦", "ok");
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Stash fallito: " + e, "errore");
    }
  }

  async function scartaTutto() {
    if (!(await confirm("Scartare TUTTE le modifiche non salvate? Operazione irreversibile."))) return;
    await api.scartaTutto(stato.percorso);
    stato.avvisa("Modifiche scartate");
    stato.ricarica();
  }

  async function pulisci() {
    if (!(await confirm("Eliminare dal disco tutti i file non tracciati?"))) return;
    await api.pulisciNonTracciati(stato.percorso);
    stato.avvisa("File non tracciati eliminati");
    stato.ricarica();
  }

  // Azione su un singolo hunk dal pannello Diff.
  async function suHunk(indice, tipo) {
    try {
      if (tipo === "scarta") {
        await api.hunkScarta(stato.percorso, scelto.file, indice);
      } else {
        await api.hunkStage(stato.percorso, scelto.file, indice, tipo === "unstage");
      }
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Operazione sul hunk fallita: " + e, "errore");
    }
  }
</script>

<div class="modifiche">
  <div class="pannello-modifiche">
    {#if conflitti.length > 0}
      <Conflitti file={conflitti} />
    {/if}
    {#if s}
      <!-- File non in stage -->
      <div class="gruppo-file">
        <div class="intestazione">
          <span>Modifiche ({s.non_in_stage.length})</span>
          {#if s.non_in_stage.length > 0}
            <span class="int-ops">
              <button class="fantasma" title="Scarta tutto" onclick={scartaTutto}>↩</button>
              <button class="fantasma" title="Elimina i non tracciati" onclick={pulisci}>🧹</button>
              <button class="fantasma" onclick={addTutto}>Aggiungi tutto</button>
            </span>
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
                <button title="Blame / cronologia" onclick={(e) => { e.stopPropagation(); blameFile = f.percorso; }}>📜</button>
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
          placeholder={amendOn ? "Nuovo messaggio (vuoto = lascia invariato)" : s.in_stage.length ? "Messaggio del commit…" : "Aggiungi dei file per fare un commit"}
        ></textarea>
        <div class="commit-azioni">
          <label class="amend" title="Correggi l'ultimo commit invece di crearne uno nuovo">
            <input type="checkbox" checked={amendOn} onchange={toggleAmend} />
            Amend
          </label>
          <button class="fantasma" onclick={stash} title="Metti da parte le modifiche">📦 Stash</button>
        </div>
        <button
          class="primario"
          style="width:100%"
          disabled={amendOn ? !messaggio.trim() : !messaggio.trim() || s.in_stage.length === 0}
          onclick={commit}
        >
          {amendOn ? "Correggi ultimo commit" : "Commit su " + s.ramo}
        </button>
      </div>
    {/if}
  </div>

  <Diff testo={diffTesto} inStage={scelto?.inStage} onHunk={suHunk} />
</div>

{#if blameFile}
  <Blame file={blameFile} chiudi={() => (blameFile = null)} />
{/if}
