<script>
  // Impostazioni: aspetto (tema), autore dei commit, gestione dei remoti e info.
  import { confirm, open } from "@tauri-apps/plugin-dialog";
  import { check } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";
  import { VERSIONE, changelog } from "../lib/versione.js";

  let { chiudi } = $props();

  let cfgNome = $state("");
  let cfgEmail = $state("");
  let remoti = $state([]);
  let nuovoNome = $state("");
  let nuovoUrl = $state("");
  let reflog = $state([]);
  let submoduli = $state([]);
  let worktree = $state([]);
  let wtNome = $state("");
  let wtCartella = $state("");

  // Impostazioni AI (Anthropic).
  let aiToken = $state(stato.aiToken);
  let aiModello = $state(stato.aiModello);
  const modelli = ["claude-opus-4-8", "claude-sonnet-5", "claude-haiku-4-5-20251001"];
  function salvaAi() {
    stato.impostaAi(aiToken.trim(), aiModello);
    stato.avvisa("Impostazioni AI salvate", "ok");
  }

  // Azioni personalizzate (comandi git salvati).
  let azNome = $state("");
  let azComando = $state("");
  function aggiungiAzione() {
    if (!azNome.trim() || !azComando.trim()) return;
    stato.aggiungiAzione(azNome.trim(), azComando.trim());
    azNome = "";
    azComando = "";
  }

  $effect(() => {
    api.configUtente(stato.percorso).then((c) => {
      cfgNome = c.nome;
      cfgEmail = c.email;
    });
    ricaricaRemoti();
    api.reflogLista(stato.percorso).then((r) => (reflog = r)).catch(() => {});
    api.submoduliLista(stato.percorso).then((s) => (submoduli = s)).catch(() => {});
    api.worktreeLista(stato.percorso).then((w) => (worktree = w)).catch(() => {});
  });

  async function controllaAggiornamenti() {
    try {
      const update = await check();
      if (!update) {
        stato.avvisa("Sei già aggiornato", "ok");
        return;
      }
      if (await confirm("Disponibile la versione " + update.version + ". Installare?")) {
        await update.downloadAndInstall();
        await relaunch();
      }
    } catch (e) {
      stato.avvisa("Controllo aggiornamenti fallito: " + e, "errore");
    }
  }

  async function aggiornaSub(nome) {
    try {
      await api.submoduloAggiorna(stato.percorso, nome);
      stato.avvisa("Sottomodulo aggiornato", "ok");
    } catch (e) {
      stato.avvisa(String(e), "errore");
    }
  }

  async function scegliCartellaWt() {
    const d = await open({ directory: true, title: "Cartella del worktree" });
    if (d) wtCartella = d;
  }

  async function aggiungiWorktree() {
    if (!wtNome.trim() || !wtCartella.trim()) return;
    try {
      await api.worktreeAggiungi(stato.percorso, wtNome.trim(), wtCartella.trim());
      wtNome = "";
      wtCartella = "";
      worktree = await api.worktreeLista(stato.percorso);
      stato.avvisa("Worktree creato", "ok");
    } catch (e) {
      stato.avvisa(String(e), "errore");
    }
  }

  async function applicaPatch() {
    const f = await open({ title: "Scegli una patch da applicare" });
    if (!f) return;
    try {
      await api.patchApplica(stato.percorso, f);
      stato.avvisa("Patch applicata", "ok");
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Applicazione patch fallita: " + e, "errore");
    }
  }

  function ricaricaRemoti() {
    api.remotiDettagli(stato.percorso).then((r) => (remoti = r));
  }

  async function salvaAutore() {
    try {
      await api.impostaConfigUtente(stato.percorso, cfgNome, cfgEmail);
      stato.avvisa("Autore salvato", "ok");
    } catch (e) {
      stato.avvisa("Errore: " + e, "errore");
    }
  }

  async function aggiungiRemoto() {
    if (!nuovoNome.trim() || !nuovoUrl.trim()) return;
    try {
      await api.remotoAggiungi(stato.percorso, nuovoNome.trim(), nuovoUrl.trim());
      nuovoNome = "";
      nuovoUrl = "";
      ricaricaRemoti();
    } catch (e) {
      stato.avvisa("Errore: " + e, "errore");
    }
  }

  async function salvaUrl(r) {
    await api.remotoImpostaUrl(stato.percorso, r.nome, r.url);
    stato.avvisa("URL aggiornato", "ok");
  }

  async function rimuoviRemoto(nome) {
    if (!(await confirm("Rimuovere il remoto «" + nome + "»?"))) return;
    await api.remotoRimuovi(stato.percorso, nome);
    ricaricaRemoti();
  }
</script>

<div class="overlay" onclick={chiudi}>
  <div class="modale grande" onclick={(e) => e.stopPropagation()}>
    <div class="modale-testa">
      <h2>Impostazioni</h2>
      <button class="fantasma" onclick={chiudi}>✕</button>
    </div>

    <div class="modale-corpo impostazioni">
      <section>
        <h3>Aspetto</h3>
        <button onclick={() => stato.cambiaTema()}>
          Tema: {stato.tema === "scuro" ? "🌙 Scuro" : "☀ Chiaro"} (cambia)
        </button>
      </section>

      <section>
        <h3>Autore dei commit</h3>
        <div class="campo">
          <label for="an">Nome</label>
          <input id="an" bind:value={cfgNome} placeholder="Mario Rossi" />
        </div>
        <div class="campo">
          <label for="ae">Email</label>
          <input id="ae" bind:value={cfgEmail} placeholder="mario@esempio.it" />
        </div>
        <button class="primario" onclick={salvaAutore}>Salva autore</button>
      </section>

      <section>
        <h3>Assistente AI (Anthropic)</h3>
        <p style="color:var(--testo2);font-size:12px;margin-top:0">
          Genera i messaggi di commit dal diff in stage. Il token resta solo su questo
          computer e non viene mai messo nel repository. Ottieni una chiave da
          console.anthropic.com.
        </p>
        <div class="campo">
          <label for="ait">Token API (x-api-key)</label>
          <input id="ait" type="password" bind:value={aiToken} placeholder="sk-ant-…" />
        </div>
        <div class="campo">
          <label for="aim">Modello</label>
          <select id="aim" bind:value={aiModello}>
            {#each modelli as m}<option value={m}>{m}</option>{/each}
          </select>
        </div>
        <button class="primario" onclick={salvaAi}>Salva impostazioni AI</button>
      </section>

      <section>
        <h3>Remoti</h3>
        {#each remoti as r}
          <div class="remoto-riga">
            <span class="r-nome">{r.nome}</span>
            <input bind:value={r.url} />
            <button onclick={() => salvaUrl(r)}>Salva</button>
            <button class="pericolo" onclick={() => rimuoviRemoto(r.nome)}>✕</button>
          </div>
        {/each}
        <div class="remoto-riga nuovo">
          <input bind:value={nuovoNome} placeholder="nome (es. origin)" />
          <input bind:value={nuovoUrl} placeholder="URL" />
          <button class="primario" onclick={aggiungiRemoto}>Aggiungi</button>
        </div>
      </section>

      <section>
        <h3>Azioni personalizzate</h3>
        <p style="color:var(--testo2);font-size:12px;margin-top:0">
          Comandi git salvati, eseguibili con un clic dal Terminale (scrivi il comando
          senza «git», es. <code>fetch --all --prune</code>).
        </p>
        {#each stato.azioniGit as a, i}
          <div class="remoto-riga">
            <span class="r-nome">{a.nome}</span>
            <span style="flex:1;color:var(--testo2);font-size:11px;font-family:ui-monospace,monospace">git {a.comando}</span>
            <button class="pericolo" onclick={() => stato.rimuoviAzione(i)}>✕</button>
          </div>
        {/each}
        <div class="remoto-riga nuovo">
          <input bind:value={azNome} placeholder="nome (es. Sync)" />
          <input bind:value={azComando} placeholder="comando (es. pull --rebase)" />
          <button class="primario" onclick={aggiungiAzione}>Aggiungi</button>
        </div>
      </section>

      <section>
        <h3>Patch</h3>
        <button onclick={applicaPatch}>Applica una patch (.patch / .diff)…</button>
      </section>

      <section>
        <h3>Sottomoduli</h3>
        {#if submoduli.length === 0}
          <p style="color:var(--testo2);font-size:12px">Nessun sottomodulo.</p>
        {/if}
        {#each submoduli as s}
          <div class="remoto-riga">
            <span class="r-nome">{s.nome}</span>
            <span style="flex:1;color:var(--testo2);font-size:11px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">{s.url}</span>
            <button onclick={() => aggiornaSub(s.nome)}>Aggiorna</button>
          </div>
        {/each}
      </section>

      <section>
        <h3>Worktree</h3>
        {#each worktree as w}
          <div class="remoto-riga">
            <span class="r-nome">{w.nome}</span>
            <span style="flex:1;color:var(--testo2);font-size:11px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">{w.percorso}</span>
          </div>
        {/each}
        <div class="remoto-riga nuovo">
          <input bind:value={wtNome} placeholder="nome" />
          <input bind:value={wtCartella} placeholder="cartella" />
          <button onclick={scegliCartellaWt}>…</button>
          <button class="primario" onclick={aggiungiWorktree}>Aggiungi</button>
        </div>
      </section>

      <section>
        <h3>Reflog (HEAD)</h3>
        <div class="reflog">
          {#each reflog as r}
            <div class="reflog-riga"><span class="hash">{r.id_breve}</span> {r.messaggio}</div>
          {/each}
        </div>
      </section>

      <section>
        <h3>Sicurezza rete</h3>
        <label class="sic-toggle">
          <input type="checkbox" checked={stato.tlsInsicuro} onchange={(e) => stato.cambiaTlsInsicuro(e.target.checked)} />
          <span>Disabilita la verifica del certificato TLS/SSH</span>
        </label>
        {#if stato.tlsInsicuro}
          <div class="sic-avviso">
            ⚠ <b>Attenzione:</b> fetch/pull/push/clone <b>non</b> verificheranno il certificato
            del server. Usalo solo con server interni fidati (certificati self-signed):
            disattiva la protezione contro attacchi man-in-the-middle.
          </div>
        {:else}
          <p style="color:var(--testo2);font-size:12px;margin:6px 0 0">
            Utile per server aziendali con certificati self-signed. Tienilo spento se non ti serve.
          </p>
        {/if}
      </section>

      <section>
        <h3>Aggiornamenti</h3>
        <button onclick={controllaAggiornamenti}>Controlla aggiornamenti</button>
      </section>

      <section>
        <h3>Info — Oops v{VERSIONE}</h3>
        {#each changelog as c}
          <div class="cl-versione">
            <strong>v{c.versione}</strong>
            <ul>
              {#each c.note as n}<li>{n}</li>{/each}
            </ul>
          </div>
        {/each}
      </section>
    </div>
  </div>
</div>
