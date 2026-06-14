<script>
  // Sidebar: logo, elenco dei rami (locali e remoti) con le relative azioni.
  import { confirm } from "@tauri-apps/plugin-dialog";
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";
  import Diff from "./Diff.svelte";

  let rami = $state([]);
  let tag = $state([]);
  let stash = $state([]);
  let mostraNuovo = $state(false);
  let nomeNuovo = $state("");
  let mostraTag = $state(false);
  let nomeTag = $state("");
  let msgTag = $state("");
  let stashSel = $state(null); // indice dello stash aperto nel dettaglio
  let stashTesto = $state("");

  $effect(() => {
    stato.tic;
    if (!stato.percorso) return;
    api.ramiLista(stato.percorso).then((r) => (rami = r));
    api.tagLista(stato.percorso).then((t) => (tag = t));
    api.stashLista(stato.percorso).then((s) => (stash = s));
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

  async function rebase(nome, ev) {
    ev.stopPropagation();
    if (!(await confirm("Riposizionare (rebase) il ramo corrente su «" + nome + "»?"))) return;
    try {
      const esito = await api.ramoRebase(stato.percorso, nome);
      stato.avvisa("Rebase: " + esito, "ok");
      stato.ricarica();
    } catch (e) {
      stato.avvisa(String(e), "errore");
    }
  }

  async function eliminaRemoto(nomeCompleto, ev) {
    ev.stopPropagation();
    // "origin/feature" -> remoto "origin", ramo "feature".
    const i = nomeCompleto.indexOf("/");
    if (i < 0) return;
    const remoto = nomeCompleto.slice(0, i);
    const ramo = nomeCompleto.slice(i + 1);
    if (!(await confirm("Eliminare «" + ramo + "» sul remoto «" + remoto + "»?"))) return;
    try {
      await api.eliminaRamoRemoto(stato.percorso, remoto, ramo);
      stato.avvisa("Ramo remoto eliminato");
      stato.ricarica();
    } catch (e) {
      stato.avvisa(String(e), "errore");
    }
  }

  async function creaTag() {
    if (!nomeTag.trim()) return;
    try {
      await api.tagCrea(stato.percorso, nomeTag.trim(), msgTag);
      mostraTag = false;
      nomeTag = "";
      msgTag = "";
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Creazione tag fallita: " + e, "errore");
    }
  }

  async function eliminaTag(nome, ev) {
    ev.stopPropagation();
    if (!(await confirm("Eliminare la tag «" + nome + "»?"))) return;
    await api.tagElimina(stato.percorso, nome);
    stato.ricarica();
  }

  async function apriStash(indice) {
    stashSel = indice;
    stashTesto = await api.stashDiff(stato.percorso, indice).catch(() => "");
  }

  async function applicaStash(indice, pop) {
    try {
      if (pop) await api.stashPop(stato.percorso, indice);
      else await api.stashApplica(stato.percorso, indice);
      stashSel = null;
      stato.avvisa("Stash ripristinato", "ok");
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Ripristino dello stash fallito: " + e, "errore");
    }
  }

  async function eliminaStash(indice, ev) {
    if (ev) ev.stopPropagation();
    if (!(await confirm("Eliminare questo stash?"))) return;
    await api.stashElimina(stato.percorso, indice);
    stashSel = null;
    stato.ricarica();
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
            <button title="Rebase del ramo corrente su questo" onclick={(e) => rebase(r.nome, e)}>⤴</button>
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
          <span class="ops">
            <button class="pericolo" title="Elimina sul remoto" onclick={(e) => eliminaRemoto(r.nome, e)}>✕</button>
          </span>
        </div>
      {/each}
    {/if}

    <!-- Tag -->
    <div class="sezione">
      <div class="titolo">
        <span>Tag</span>
        <button class="fantasma" title="Nuova tag" onclick={() => (mostraTag = true)}>＋</button>
      </div>
    </div>
    {#each tag as t}
      <div class="ramo" title={t.messaggio}>
        <span class="icona">🏷</span>
        <span class="nome">{t.nome}</span>
        <span class="ops">
          <button class="pericolo" title="Elimina" onclick={(e) => eliminaTag(t.nome, e)}>✕</button>
        </span>
      </div>
    {/each}

    <!-- Stash -->
    {#if stash.length > 0}
      <div class="sezione">
        <div class="titolo"><span>Stash</span></div>
      </div>
      {#each stash as st}
        <div class="ramo" title="Clic per vedere il contenuto" onclick={() => apriStash(st.indice)}>
          <span class="icona">📦</span>
          <span class="nome">{st.messaggio}</span>
          <span class="ops">
            <button class="pericolo" title="Elimina" onclick={(e) => eliminaStash(st.indice, e)}>✕</button>
          </span>
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

{#if mostraTag}
  <div class="overlay" onclick={() => (mostraTag = false)}>
    <div class="modale" onclick={(e) => e.stopPropagation()}>
      <h2>Nuova tag</h2>
      <div class="campo">
        <label for="nt">Nome</label>
        <input id="nt" bind:value={nomeTag} placeholder="es. v1.0" />
      </div>
      <div class="campo">
        <label for="mt">Messaggio (vuoto = tag leggera)</label>
        <input id="mt" bind:value={msgTag} placeholder="Descrizione della release" />
      </div>
      <div class="pulsanti">
        <button onclick={() => (mostraTag = false)}>Annulla</button>
        <button class="primario" onclick={creaTag}>Crea tag</button>
      </div>
    </div>
  </div>
{/if}

{#if stashSel !== null}
  <div class="overlay" onclick={() => (stashSel = null)}>
    <div class="modale grande" onclick={(e) => e.stopPropagation()}>
      <div class="modale-testa">
        <h2>Stash</h2>
        <div class="tabs-mini">
          <button onclick={() => applicaStash(stashSel, false)}>Applica</button>
          <button class="primario" onclick={() => applicaStash(stashSel, true)}>Pop</button>
          <button class="pericolo" onclick={() => eliminaStash(stashSel, null)}>Elimina</button>
        </div>
        <button class="fantasma" onclick={() => (stashSel = null)}>✕</button>
      </div>
      <div class="modale-corpo" style="padding:0">
        <Diff testo={stashTesto} vuoto="Stash vuoto." />
      </div>
    </div>
  </div>
{/if}
