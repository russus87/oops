<script>
  // Schermata iniziale: apri/inizializza/clona un repository + elenco recenti.
  import { open } from "@tauri-apps/plugin-dialog";
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";

  let recenti = $state([]);
  let mostraClona = $state(false);
  let urlClona = $state("");

  // Carica i recenti all'avvio del componente.
  $effect(() => {
    api.recentiLista().then((r) => (recenti = r));
  });

  // Apre una cartella esistente come repository.
  async function apriCartella() {
    const dir = await open({ directory: true, title: "Apri un repository Git" });
    if (!dir) return;
    try {
      const radice = await api.apriRepo(dir);
      recenti = await api.recentiAggiungi(radice);
      stato.apri(radice);
    } catch (e) {
      stato.avvisa("Qui non c'è un repository Git: " + e, "errore");
    }
  }

  // Inizializza un nuovo repository in una cartella.
  async function inizializza() {
    const dir = await open({ directory: true, title: "Cartella del nuovo repository" });
    if (!dir) return;
    try {
      const radice = await api.initRepo(dir);
      recenti = await api.recentiAggiungi(radice);
      stato.apri(radice);
    } catch (e) {
      stato.avvisa("Impossibile inizializzare: " + e, "errore");
    }
  }

  // Clona un repository remoto: chiede URL e cartella di destinazione.
  async function clona() {
    if (!urlClona.trim()) return;
    const dir = await open({ directory: true, title: "Dove clonare il repository" });
    if (!dir) return;
    stato.occupato = true;
    try {
      const radice = await api.clona(urlClona.trim(), dir);
      recenti = await api.recentiAggiungi(radice);
      mostraClona = false;
      urlClona = "";
      stato.apri(radice);
    } catch (e) {
      stato.avvisa("Clonazione fallita: " + e, "errore");
    } finally {
      stato.occupato = false;
    }
  }

  // Apre un repository dalla lista dei recenti.
  async function apriRecente(percorso) {
    try {
      const radice = await api.apriRepo(percorso);
      recenti = await api.recentiAggiungi(radice);
      stato.apri(radice);
    } catch (e) {
      stato.avvisa("Repository non più disponibile: " + e, "errore");
    }
  }

  async function togliRecente(percorso, ev) {
    ev.stopPropagation();
    recenti = await api.recentiRimuovi(percorso);
  }
</script>

<div class="avvio">
  <div class="marchio">
    <h1>Oops<span class="punto">.</span></h1>
    <p>Git senza panico — quando sbagli, niente paura.</p>
  </div>

  <div class="azioni">
    <button class="primario" onclick={apriCartella}>Apri repository</button>
    <button onclick={inizializza}>Nuovo repository</button>
    <button onclick={() => (mostraClona = true)}>Clona…</button>
  </div>

  {#if recenti.length > 0}
    <div class="recenti">
      <h3>Aperti di recente</h3>
      {#each recenti as r}
        <div class="recente" onclick={() => apriRecente(r.percorso)}>
          <span class="nome">{r.nome}</span>
          <span class="perc">{r.percorso}</span>
          <button class="x fantasma" onclick={(e) => togliRecente(r.percorso, e)}>✕</button>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if mostraClona}
  <div class="overlay" onclick={() => (mostraClona = false)}>
    <div class="modale" onclick={(e) => e.stopPropagation()}>
      <h2>Clona un repository</h2>
      <div class="campo">
        <label for="url">URL (https o ssh)</label>
        <input id="url" bind:value={urlClona} placeholder="https://github.com/utente/repo.git" />
      </div>
      <div class="pulsanti">
        <button onclick={() => (mostraClona = false)}>Annulla</button>
        <button class="primario" onclick={clona} disabled={stato.occupato}>
          {stato.occupato ? "Clono…" : "Clona"}
        </button>
      </div>
    </div>
  </div>
{/if}
