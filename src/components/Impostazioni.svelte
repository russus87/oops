<script>
  // Impostazioni: aspetto (tema), autore dei commit, gestione dei remoti e info.
  import { confirm } from "@tauri-apps/plugin-dialog";
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";
  import { VERSIONE, changelog } from "../lib/versione.js";

  let { chiudi } = $props();

  let cfgNome = $state("");
  let cfgEmail = $state("");
  let remoti = $state([]);
  let nuovoNome = $state("");
  let nuovoUrl = $state("");

  $effect(() => {
    api.configUtente(stato.percorso).then((c) => {
      cfgNome = c.nome;
      cfgEmail = c.email;
    });
    ricaricaRemoti();
  });

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
