<script>
  // Componente principale: decide tra schermata di avvio e area di lavoro,
  // e contiene la toolbar (Fetch/Pull/Push) e le schede Modifiche/Cronologia.
  import * as api from "./lib/api.js";
  import { stato } from "./lib/stato.svelte.js";
  import Avvio from "./components/Avvio.svelte";
  import BarraLaterale from "./components/BarraLaterale.svelte";
  import Modifiche from "./components/Modifiche.svelte";
  import Cronologia from "./components/Cronologia.svelte";

  let vista = $state("modifiche"); // "modifiche" | "cronologia"
  let info = $state(null); // StatoRepo, serve alla toolbar per avanti/indietro
  let mostraConfig = $state(false);
  let cfgNome = $state("");
  let cfgEmail = $state("");

  async function apriConfig() {
    const c = await api.configUtente(stato.percorso).catch(() => ({ nome: "", email: "" }));
    cfgNome = c.nome;
    cfgEmail = c.email;
    mostraConfig = true;
  }

  async function salvaConfig() {
    try {
      await api.impostaConfigUtente(stato.percorso, cfgNome, cfgEmail);
      mostraConfig = false;
      stato.avvisa("Autore impostato", "ok");
    } catch (e) {
      stato.avvisa("Salvataggio fallito: " + e, "errore");
    }
  }

  $effect(() => {
    stato.tic;
    if (!stato.percorso) {
      info = null;
      return;
    }
    api.stato(stato.percorso).then((s) => (info = s)).catch(() => (info = null));
  });

  async function azioneRete(fn, nome) {
    stato.occupato = true;
    try {
      const esito = await fn(stato.percorso);
      stato.avvisa(nome + ": " + (esito || "fatto"), "ok");
      stato.ricarica();
    } catch (e) {
      stato.avvisa(nome + " fallito: " + e, "errore");
    } finally {
      stato.occupato = false;
    }
  }

  const fetch = () => azioneRete((p) => api.fetch(p), "Fetch");
  const pull = () => azioneRete((p) => api.pull(p), "Pull");
  const push = () => azioneRete((p) => api.push(p), "Push");
</script>

{#if !stato.percorso}
  <Avvio />
{:else}
  <div class="app">
    <BarraLaterale />

    <div class="principale">
      <div class="toolbar">
        <span class="repo">{stato.nome}</span>
        {#if info}
          <span class="ramo-attivo">⎇ {info.ramo}</span>
          {#if info.avanti > 0}<span class="badge">↑{info.avanti}</span>{/if}
          {#if info.indietro > 0}<span class="badge">↓{info.indietro}</span>{/if}
        {/if}
        <span class="spazio"></span>
        <div class="sincro">
          <button onclick={fetch} disabled={stato.occupato}>Fetch</button>
          <button onclick={pull} disabled={stato.occupato}>Pull</button>
          <button class="primario" onclick={push} disabled={stato.occupato}>Push</button>
          <button class="fantasma" title="Imposta autore (nome/email)" onclick={apriConfig}>⚙</button>
        </div>
      </div>

      <div class="tabs">
        <div class="tab" class:attivo={vista === "modifiche"} onclick={() => (vista = "modifiche")}>
          Modifiche
        </div>
        <div class="tab" class:attivo={vista === "cronologia"} onclick={() => (vista = "cronologia")}>
          Cronologia
        </div>
      </div>

      <div class="contenuto">
        {#if vista === "modifiche"}
          <Modifiche />
        {:else}
          <Cronologia />
        {/if}
      </div>
    </div>
  </div>
{/if}

{#if mostraConfig}
  <div class="overlay" onclick={() => (mostraConfig = false)}>
    <div class="modale" onclick={(e) => e.stopPropagation()}>
      <h2>Autore dei commit</h2>
      <div class="campo">
        <label for="cn">Nome</label>
        <input id="cn" bind:value={cfgNome} placeholder="Mario Rossi" />
      </div>
      <div class="campo">
        <label for="ce">Email</label>
        <input id="ce" bind:value={cfgEmail} placeholder="mario@esempio.it" />
      </div>
      <div class="pulsanti">
        <button onclick={() => (mostraConfig = false)}>Annulla</button>
        <button class="primario" onclick={salvaConfig}>Salva</button>
      </div>
    </div>
  </div>
{/if}

{#if stato.nota}
  <div class="toast {stato.tipoNota}">{stato.nota}</div>
{/if}
