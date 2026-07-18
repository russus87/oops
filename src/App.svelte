<script>
  // Componente principale: schermata di avvio oppure area di lavoro con toolbar,
  // barra laterale (navigazione + rami), vista corrente e status bar.
  import { listen } from "@tauri-apps/api/event";
  import { openPath } from "@tauri-apps/plugin-opener";
  import { confirm } from "@tauri-apps/plugin-dialog";
  import {
    isPermissionGranted,
    requestPermission,
    sendNotification,
  } from "@tauri-apps/plugin-notification";
  import * as api from "./lib/api.js";
  import { stato } from "./lib/stato.svelte.js";
  import Avvio from "./components/Avvio.svelte";
  import BarraLaterale from "./components/BarraLaterale.svelte";
  import Dashboard from "./components/Dashboard.svelte";
  import Modifiche from "./components/Modifiche.svelte";
  import Cronologia from "./components/Cronologia.svelte";
  import Insights from "./components/Insights.svelte";
  import Timeline from "./components/Timeline.svelte";
  import Terminale from "./components/Terminale.svelte";
  import Impostazioni from "./components/Impostazioni.svelte";
  import Credenziali from "./components/Credenziali.svelte";
  import Ricerca from "./components/Ricerca.svelte";
  import GitFlow from "./components/GitFlow.svelte";
  import Blame from "./components/Blame.svelte";

  let info = $state(null); // StatoRepo, serve a toolbar e status bar
  let mostraImpostazioni = $state(false);
  let mostraGitFlow = $state(false);
  let menuPush = $state(false);
  let menuPull = $state(false);

  const titoli = {
    panoramica: "Panoramica",
    modifiche: "Modifiche",
    cronologia: "Cronologia",
    insights: "Insights",
    timeline: "Timeline",
    terminale: "Terminale",
  };

  $effect(() => {
    stato.tic;
    if (!stato.percorso) {
      info = null;
      return;
    }
    api.stato(stato.percorso).then((s) => (info = s)).catch(() => (info = null));
  });

  // Quando si apre un repository, avvia l'osservatore del filesystem.
  $effect(() => {
    if (stato.percorso) api.avviaOsservatore(stato.percorso).catch(() => {});
  });

  // Ascolta una sola volta gli eventi del filesystem e ricarica (con debounce).
  let timerFs;
  $effect(() => {
    const promessa = listen("oops-fs", () => {
      clearTimeout(timerFs);
      timerFs = setTimeout(() => stato.percorso && stato.ricarica(), 500);
    });
    return () => promessa.then((un) => un());
  });

  async function notifica(testo) {
    try {
      let ok = await isPermissionGranted();
      if (!ok) ok = (await requestPermission()) === "granted";
      if (ok) sendNotification({ title: "Oops", body: testo });
    } catch {}
  }

  // Scorciatoie da tastiera globali.
  function tasti(e) {
    if (!stato.percorso) return;
    const cmd = e.ctrlKey || e.metaKey;
    if (cmd && e.key === "k") {
      e.preventDefault();
      stato.ricercaAperta = !stato.ricercaAperta;
    } else if (e.key === "F5" || (cmd && e.key === "r")) {
      e.preventDefault();
      stato.ricarica();
    } else if (cmd && e.key === "1") stato.vista = "panoramica";
    else if (cmd && e.key === "2") stato.vista = "modifiche";
    else if (cmd && e.key === "3") stato.vista = "cronologia";
    else if (cmd && e.shiftKey && (e.key === "z" || e.key === "Z")) {
      e.preventDefault();
      rifai();
    } else if (cmd && e.key === "z") {
      e.preventDefault();
      annulla();
    }
  }

  const eAuth = (e) => /authenticat|401|403|credential|auth/i.test(String(e));

  async function azioneRete(fn, nome) {
    stato.occupato = true;
    try {
      const esito = await fn(null);
      stato.avvisa(nome + ": " + (esito || "fatto"), "ok");
      notifica(nome + ": " + (esito || "completato"));
      stato.ricarica();
    } catch (e) {
      if (eAuth(e)) {
        const cred = await stato.chiediCredenziali();
        if (!cred) {
          stato.avvisa(nome + " annullato", "errore");
        } else {
          try {
            const esito = await fn(cred);
            stato.avvisa(nome + ": " + (esito || "fatto"), "ok");
            notifica(nome + ": " + (esito || "completato"));
            stato.ricarica();
          } catch (e2) {
            stato.avvisa(nome + " fallito: " + e2, "errore");
            notifica(nome + " fallito");
          }
        }
      } else {
        stato.avvisa(nome + " fallito: " + e, "errore");
        notifica(nome + " fallito");
      }
    } finally {
      stato.occupato = false;
    }
  }

  async function annulla() {
    if (!(await confirm("Annullare l'ultima operazione? (reset allo stato precedente, le modifiche non salvate possono andare perse)"))) return;
    try {
      // Ricorda lo stato attuale per un eventuale "Rifai".
      const oid = (await api.eseguiGit(stato.percorso, ["rev-parse", "HEAD"])).trim();
      const msg = await api.annullaUltima(stato.percorso);
      if (/^[0-9a-f]{7,40}$/.test(oid)) stato.redoOid = oid;
      stato.avvisa(msg, "ok");
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Annulla fallito: " + e, "errore");
    }
  }

  async function rifai() {
    if (!stato.redoOid) {
      stato.avvisa("Niente da rifare", "errore");
      return;
    }
    const oid = stato.redoOid;
    try {
      await api.eseguiGit(stato.percorso, ["reset", "--hard", oid]);
      stato.redoOid = null;
      stato.avvisa("Operazione ripristinata (redo)", "ok");
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Rifai fallito: " + e, "errore");
    }
  }

  const apriCartella = () => openPath(stato.percorso).catch(() => {});

  const fetch = () => azioneRete((c) => api.fetch(stato.percorso, "origin", c), "Fetch");
  const pull = (strategia = "ff") => { menuPull = false; azioneRete((c) => api.pull(stato.percorso, "origin", strategia, c), "Pull"); };
  const push = () => azioneRete((c) => api.push(stato.percorso, "origin", false, c), "Push");
  const pushForza = () => { menuPush = false; azioneRete((c) => api.push(stato.percorso, "origin", true, c), "Push forzato"); };
  const pushTags = () => { menuPush = false; azioneRete((c) => api.pushTags(stato.percorso, "origin", c), "Push tag"); };

  let nCambi = $derived(info ? info.in_stage.length + info.non_in_stage.length : 0);
</script>

<svelte:window onkeydown={tasti} onfocus={() => stato.percorso && stato.ricarica()} />

{#if !stato.percorso}
  <Avvio />
{:else}
  <div class="app">
    <BarraLaterale {info} />

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
          <button class="fantasma" title="Cerca (Ctrl+K)" onclick={() => (stato.ricercaAperta = true)}>⌕</button>
          <button class="fantasma" title="Annulla ultima operazione (Ctrl+Z)" onclick={annulla}>↶</button>
          <button class="fantasma" title="Rifai (Ctrl+Shift+Z)" onclick={rifai} disabled={!stato.redoOid}>↷</button>
          <button class="fantasma" title="Git Flow: nuova feature" onclick={() => (mostraGitFlow = true)}>🌱</button>
          <button class="fantasma" title="Aggiorna (F5)" onclick={() => stato.ricarica()}>⟳</button>
          <button class="fantasma" title="Apri la cartella" onclick={apriCartella}>📂</button>
          <button onclick={fetch} disabled={stato.occupato}>Fetch</button>
          <div class="menu-wrap">
            <button onclick={() => pull("ff")} disabled={stato.occupato}>Pull</button>
            <button class="fantasma" title="Strategia di pull" onclick={() => (menuPull = !menuPull)}>▾</button>
            {#if menuPull}
              <div class="menu">
                <button onclick={() => pull("ff")}>Pull (solo fast-forward)</button>
                <button onclick={() => pull("merge")}>Pull con merge</button>
                <button onclick={() => pull("rebase")}>Pull con rebase</button>
              </div>
            {/if}
          </div>
          <div class="menu-wrap">
            <button class="primario" onclick={push} disabled={stato.occupato}>Push</button>
            <button class="fantasma" title="Altre opzioni di push" onclick={() => (menuPush = !menuPush)}>▾</button>
            {#if menuPush}
              <div class="menu">
                <button onclick={pushForza}>Push --force</button>
                <button onclick={pushTags}>Push delle tag</button>
              </div>
            {/if}
          </div>
          <button class="fantasma" title="Impostazioni" onclick={() => (mostraImpostazioni = true)}>⚙</button>
        </div>
      </div>

      <div class="contenuto">
        {#if stato.vista === "panoramica"}
          <Dashboard vai={(v) => (stato.vista = v)} />
        {:else if stato.vista === "modifiche"}
          <Modifiche />
        {:else if stato.vista === "cronologia"}
          <Cronologia />
        {:else if stato.vista === "insights"}
          <Insights />
        {:else if stato.vista === "timeline"}
          <Timeline />
        {:else if stato.vista === "terminale"}
          <Terminale />
        {/if}
      </div>

      <div class="statusbar">
        <span class="sb-ramo">⎇ {info?.ramo ?? "…"}</span>
        {#if info}
          <span class="sb-sync">↑{info.avanti} ↓{info.indietro}</span>
          <span class="sb-cambi">{nCambi} modific{nCambi === 1 ? "a" : "he"}</span>
        {/if}
        <span class="spazio"></span>
        <span class="sb-vista">{titoli[stato.vista] ?? ""}</span>
        <span class="sb-nome">Oops v0.9.1</span>
      </div>
    </div>
  </div>
{/if}

{#if mostraImpostazioni}
  <Impostazioni chiudi={() => (mostraImpostazioni = false)} />
{/if}

{#if mostraGitFlow}
  <GitFlow chiudi={() => (mostraGitFlow = false)} />
{/if}

{#if stato.ricercaAperta}
  <Ricerca />
{/if}

{#if stato.storiaFile}
  <Blame file={stato.storiaFile} chiudi={() => (stato.storiaFile = null)} />
{/if}

{#if stato.credAperta}
  <Credenziali />
{/if}

{#if stato.nota}
  <div class="toast {stato.tipoNota}">{stato.nota}</div>
{/if}
