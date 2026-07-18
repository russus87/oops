<script>
  // Sidebar: logo, elenco dei rami (locali e remoti) con le relative azioni.
  import { confirm } from "@tauri-apps/plugin-dialog";
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";
  import Diff from "./Diff.svelte";
  import { tempoRelativo } from "../lib/util.js";

  let { info = null } = $props();

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

  // --- Drag&drop sui rami ---
  //  · commit → ramo  = cherry-pick su quel ramo
  //  · ramo   → ramo  = menu Merge / Rebase
  let ramoSopra = $state(null); // nome del ramo attualmente "sotto" al drag
  let menuDrop = $state(null); // { sorgente, destinazione } — merge/rebase (ramo→ramo)
  let menuCommit = $state(null); // { commit:{id,breve}, ramo } — copy/move/squash (commit→ramo)

  const nomeTrascinato = $derived(stato.trascina?.tipo === "ramo" ? stato.trascina.nome : null);

  function iniziaTrascinaRamo(e, nome) {
    stato.trascina = { tipo: "ramo", nome };
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("application/x-oops-ramo", nome);
  }

  function trascinaSopra(e, nome) {
    const t = stato.trascina;
    if (!t) return;
    if (t.tipo === "ramo" && t.nome === nome) return; // non su se stesso
    e.preventDefault();
    e.dataTransfer.dropEffect = t.tipo === "ramo" ? "move" : "copy";
    ramoSopra = nome;
  }

  async function rilascia(e, nome) {
    e.preventDefault();
    ramoSopra = null;
    const t = stato.trascina;
    stato.trascina = null;
    if (!t) return;
    if (t.tipo === "ramo") {
      if (t.nome === nome) return;
      menuDrop = { sorgente: t.nome, destinazione: nome }; // scegli Merge o Rebase
      return;
    }
    // Commit → menu Copy / Move / Squash sul ramo.
    menuCommit = { commit: { id: t.id, breve: t.breve }, ramo: nome };
  }

  async function eseguiCommit(azione) {
    const { commit, ramo } = menuCommit;
    menuCommit = null;
    try {
      if (azione === "copy") {
        await api.cherryPickSu(stato.percorso, commit.id, ramo);
        stato.avvisa("Copiato " + commit.breve + " su " + ramo + " 🍒", "ok");
      } else if (azione === "move") {
        await api.cherryPickMuovi(stato.percorso, commit.id, ramo);
        stato.avvisa("Spostato " + commit.breve + " su " + ramo, "ok");
      } else {
        await api.cherryPickSquash(stato.percorso, commit.id, ramo);
        stato.avvisa("Squash di " + commit.breve + " in " + ramo, "ok");
      }
      stato.ricarica();
    } catch (e) {
      stato.avvisa(String(e), "errore");
    }
  }

  async function eseguiDrop(azione) {
    const { sorgente, destinazione } = menuDrop;
    menuDrop = null;
    try {
      if (azione === "merge") {
        const esito = await api.mergeRami(stato.percorso, sorgente, destinazione);
        stato.avvisa("Merge di " + sorgente + " in " + destinazione + ": " + esito, "ok");
      } else {
        const esito = await api.rebaseRami(stato.percorso, sorgente, destinazione);
        stato.avvisa("Rebase di " + sorgente + " su " + destinazione + ": " + esito, "ok");
      }
      stato.ricarica();
    } catch (e) {
      stato.avvisa(String(e), "errore");
    }
  }

  // Voci di navigazione (stile mockup DevStudio).
  const nav = [
    { id: "panoramica", eti: "Panoramica", ico: "◉" },
    { id: "modifiche", eti: "Modifiche", ico: "✎" },
    { id: "cronologia", eti: "Cronologia", ico: "❋" },
    { id: "insights", eti: "Insights", ico: "▤" },
    { id: "timeline", eti: "Timeline", ico: "⏱" },
    { id: "terminale", eti: "Terminale", ico: "▸" },
  ];
  let nCambi = $derived(info ? info.in_stage.length + info.non_in_stage.length : 0);
</script>

<div class="sidebar">
  <div class="logo">
    <span>Oops<span class="punto">.</span></span>
    <button class="fantasma" title="Chiudi repository" onclick={() => stato.chiudi()}>⌂</button>
  </div>

  <div class="nav">
    {#each nav as v}
      <div class="nav-voce" class:attivo={stato.vista === v.id} onclick={() => (stato.vista = v.id)}>
        <span class="nv-ico">{v.ico}</span>
        <span class="nv-eti">{v.eti}</span>
        {#if v.id === "modifiche" && nCambi > 0}<span class="nv-badge">{nCambi}</span>{/if}
        {#if v.id === "timeline" && stato.azioni.length > 0}<span class="nv-badge">{stato.azioni.length}</span>{/if}
      </div>
    {/each}
  </div>

  <div class="sezione">
    <div class="titolo">
      <span>Rami</span>
      <button class="fantasma" title="Nuovo ramo" onclick={() => (mostraNuovo = true)}>＋</button>
    </div>
  </div>

  <div class="lista-rami">
    {#each locali as r}
      <div
        class="ramo"
        class:corrente={r.corrente}
        class:bersaglio={stato.trascina && ramoSopra !== r.nome && nomeTrascinato !== r.nome}
        class:sopra={ramoSopra === r.nome}
        class:in-trascina={nomeTrascinato === r.nome}
        draggable="true"
        ondragstart={(e) => iniziaTrascinaRamo(e, r.nome)}
        ondragend={() => (stato.trascina = null)}
        ondragover={(e) => trascinaSopra(e, r.nome)}
        ondragleave={() => (ramoSopra = null)}
        ondrop={(e) => rilascia(e, r.nome)}
        onclick={() => !r.corrente && cambia(r.nome)}
        title={r.ultimo_titolo ? "Ultimo: " + r.ultimo_titolo + " · " + tempoRelativo(r.ultimo_quando) : "Trascina su un altro ramo per Merge/Rebase"}
      >
        <span class="icona">{r.corrente ? "●" : "○"}</span>
        <span class="nome">{r.nome}</span>
        {#if r.avanti > 0}<span class="ramo-badge up">↑{r.avanti}</span>{/if}
        {#if r.indietro > 0}<span class="ramo-badge down">↓{r.indietro}</span>{/if}
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

{#if menuCommit}
  <div class="overlay" onclick={() => (menuCommit = null)}>
    <div class="modale" onclick={(e) => e.stopPropagation()} style="width:380px">
      <h2>Commit → ramo</h2>
      <p style="color:var(--testo2);font-size:13px;margin-top:0">
        Commit <code>{menuCommit.commit.breve}</code> sul ramo <code>{menuCommit.ramo}</code>.
      </p>
      <div class="drop-scelte">
        <button onclick={() => eseguiCommit("copy")}>
          <b>🍒 Copy</b><span>Cherry-pick: copia il commit sul ramo</span>
        </button>
        <button onclick={() => eseguiCommit("move")}>
          <b>➔ Move</b><span>Copia sul ramo e rimuovi dal ramo attuale</span>
        </button>
        <button onclick={() => eseguiCommit("squash")}>
          <b>🗜 Squash</b><span>Fondi le modifiche nell'ultimo commit del ramo</span>
        </button>
      </div>
      <div class="pulsanti"><button onclick={() => (menuCommit = null)}>Annulla</button></div>
    </div>
  </div>
{/if}

{#if menuDrop}
  <div class="overlay" onclick={() => (menuDrop = null)}>
    <div class="modale" onclick={(e) => e.stopPropagation()} style="width:380px">
      <h2>Integra i rami</h2>
      <p style="color:var(--testo2);font-size:13px;margin-top:0">
        Trascinato <code>{menuDrop.sorgente}</code> su <code>{menuDrop.destinazione}</code>.
        Cosa vuoi fare?
      </p>
      <div class="drop-scelte">
        <button onclick={() => eseguiDrop("merge")}>
          <b>⇄ Merge</b><span>Unisci «{menuDrop.sorgente}» dentro «{menuDrop.destinazione}»</span>
        </button>
        <button onclick={() => eseguiDrop("rebase")}>
          <b>⤴ Rebase</b><span>Riposiziona «{menuDrop.sorgente}» su «{menuDrop.destinazione}»</span>
        </button>
      </div>
      <div class="pulsanti">
        <button onclick={() => (menuDrop = null)}>Annulla</button>
      </div>
    </div>
  </div>
{/if}

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
