<script>
  // Sidebar: logo, elenco dei rami (locali e remoti) con le relative azioni.
  import { confirm } from "@tauri-apps/plugin-dialog";
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";

  let rami = $state([]);
  let mostraNuovo = $state(false);
  let nomeNuovo = $state("");

  $effect(() => {
    stato.tic;
    if (!stato.percorso) return;
    api.ramiLista(stato.percorso).then((r) => (rami = r));
  });

  let locali = $derived(rami.filter((r) => !r.remoto));
  let remoti = $derived(rami.filter((r) => r.remoto));

  async function cambia(nome) {
    try {
      await api.ramoCheckout(stato.percorso, nome);
      stato.avvisa("Sei su " + nome);
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Cambio ramo fallito: " + e, "errore");
    }
  }

  async function crea() {
    if (!nomeNuovo.trim()) return;
    try {
      await api.ramoCrea(stato.percorso, nomeNuovo.trim(), true);
      mostraNuovo = false;
      nomeNuovo = "";
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Creazione ramo fallita: " + e, "errore");
    }
  }

  async function elimina(nome, ev) {
    ev.stopPropagation();
    if (!(await confirm("Eliminare il ramo «" + nome + "»?"))) return;
    try {
      await api.ramoElimina(stato.percorso, nome);
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Eliminazione fallita: " + e, "errore");
    }
  }

  async function merge(nome, ev) {
    ev.stopPropagation();
    try {
      const esito = await api.ramoMerge(stato.percorso, nome);
      stato.avvisa("Merge: " + esito, "ok");
      stato.ricarica();
    } catch (e) {
      stato.avvisa(String(e), "errore");
    }
  }
</script>

<div class="sidebar">
  <div class="logo">
    <span>Oops<span class="punto">.</span></span>
    <button class="fantasma" title="Chiudi repository" onclick={() => stato.chiudi()}>⌂</button>
  </div>

  <div class="sezione">
    <div class="titolo">
      <span>Rami</span>
      <button class="fantasma" title="Nuovo ramo" onclick={() => (mostraNuovo = true)}>＋</button>
    </div>
  </div>

  <div class="lista-rami">
    {#each locali as r}
      <div class="ramo" class:corrente={r.corrente} onclick={() => !r.corrente && cambia(r.nome)}>
        <span class="icona">{r.corrente ? "●" : "○"}</span>
        <span class="nome">{r.nome}</span>
        {#if !r.corrente}
          <span class="ops">
            <button title="Unisci nel ramo corrente" onclick={(e) => merge(r.nome, e)}>⇄</button>
            <button class="pericolo" title="Elimina" onclick={(e) => elimina(r.nome, e)}>✕</button>
          </span>
        {/if}
      </div>
    {/each}

    {#if remoti.length > 0}
      <div class="sezione">
        <div class="titolo"><span>Remoti</span></div>
      </div>
      {#each remoti as r}
        <div class="ramo" onclick={() => cambia(r.nome)}>
          <span class="icona">☁</span>
          <span class="nome">{r.nome}</span>
        </div>
      {/each}
    {/if}
  </div>
</div>

{#if mostraNuovo}
  <div class="overlay" onclick={() => (mostraNuovo = false)}>
    <div class="modale" onclick={(e) => e.stopPropagation()}>
      <h2>Nuovo ramo</h2>
      <div class="campo">
        <label for="nr">Nome del ramo</label>
        <input id="nr" bind:value={nomeNuovo} placeholder="es. funzionalita/login" />
      </div>
      <div class="pulsanti">
        <button onclick={() => (mostraNuovo = false)}>Annulla</button>
        <button class="primario" onclick={crea}>Crea e passa</button>
      </div>
    </div>
  </div>
{/if}
