<script>
  // Git Flow Assistant: guida passo-passo per una nuova feature, usando le
  // operazioni che Oops ha già. Riduce "cosa devo fare adesso?" a una sequenza
  // chiara: crea ramo → commit → push → integra (merge) → elimina ramo.
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";

  let { chiudi } = $props();

  let passo = $state(1);
  let nome = $state("");
  let base = $state("main");
  let ramo = $state(""); // ramo feature creato
  let occupato = $state(false);

  async function creaRamo() {
    const n = nome.trim().replace(/\s+/g, "-");
    if (!n) return;
    ramo = "feature/" + n;
    occupato = true;
    try {
      await api.ramoCrea(stato.percorso, ramo, true);
      stato.avvisa("Ramo " + ramo + " creato", "ok");
      stato.ricarica();
      passo = 2;
    } catch (e) {
      stato.avvisa(String(e), "errore");
    } finally {
      occupato = false;
    }
  }

  function vaiACommit() {
    stato.vista = "modifiche";
    chiudi();
  }

  async function push() {
    occupato = true;
    try {
      await api.push(stato.percorso, "origin", false, null);
      stato.avvisa("Push di " + ramo + " eseguito", "ok");
      passo = 4;
    } catch (e) {
      stato.avvisa("Push fallito (potrebbe servire autenticazione): " + e, "errore");
    } finally {
      occupato = false;
    }
  }

  async function integra() {
    occupato = true;
    try {
      const esito = await api.mergeRami(stato.percorso, ramo, base);
      stato.avvisa("Merge in " + base + ": " + esito, "ok");
      stato.ricarica();
      passo = 5;
    } catch (e) {
      stato.avvisa(String(e), "errore");
    } finally {
      occupato = false;
    }
  }

  async function eliminaRamo() {
    occupato = true;
    try {
      await api.ramoElimina(stato.percorso, ramo);
      stato.avvisa("Ramo " + ramo + " eliminato", "ok");
      stato.ricarica();
      chiudi();
    } catch (e) {
      stato.avvisa(String(e), "errore");
    } finally {
      occupato = false;
    }
  }

  const passi = ["Nuova feature", "Sviluppo", "Push", "Integra", "Pulizia"];
</script>

<div class="overlay" onclick={chiudi}>
  <div class="modale" onclick={(e) => e.stopPropagation()} style="width:460px">
    <h2>🌱 Git Flow — nuova feature</h2>
    <div class="gf-passi">
      {#each passi as p, i}
        <div class="gf-passo" class:on={passo === i + 1} class:fatto={passo > i + 1}>
          <span class="gf-num">{passo > i + 1 ? "✓" : i + 1}</span>{p}
        </div>
      {/each}
    </div>

    {#if passo === 1}
      <div class="campo">
        <label for="gfn">Nome della feature</label>
        <input id="gfn" bind:value={nome} placeholder="es. login-social" />
      </div>
      <div class="campo">
        <label for="gfb">Ramo base (dove verrà integrata)</label>
        <input id="gfb" bind:value={base} placeholder="main" />
      </div>
      <p class="gf-nota">Verrà creato il ramo <code>feature/{nome.trim().replace(/\s+/g, "-") || "…"}</code> da qui.</p>
      <div class="pulsanti">
        <button onclick={chiudi}>Annulla</button>
        <button class="primario" disabled={occupato || !nome.trim()} onclick={creaRamo}>Crea ramo</button>
      </div>
    {:else if passo === 2}
      <p class="gf-nota">Sei su <code>{ramo}</code>. Fai le tue modifiche e i commit, poi torna qui.</p>
      <div class="pulsanti">
        <button onclick={vaiACommit}>Vai alle Modifiche</button>
        <button class="primario" onclick={() => (passo = 3)}>Ho committato →</button>
      </div>
    {:else if passo === 3}
      <p class="gf-nota">Pubblica il ramo sul remoto (opzionale, per una PR).</p>
      <div class="pulsanti">
        <button onclick={() => (passo = 4)}>Salta</button>
        <button class="primario" disabled={occupato} onclick={push}>Push del ramo</button>
      </div>
    {:else if passo === 4}
      <p class="gf-nota">Integra <code>{ramo}</code> in <code>{base}</code> (merge).</p>
      <div class="pulsanti">
        <button onclick={chiudi}>Chiudi</button>
        <button class="primario" disabled={occupato} onclick={integra}>Merge in {base}</button>
      </div>
    {:else}
      <p class="gf-nota">Feature integrata 🎉 Vuoi eliminare il ramo <code>{ramo}</code>?</p>
      <div class="pulsanti">
        <button onclick={chiudi}>Tienilo</button>
        <button class="pericolo" disabled={occupato} onclick={eliminaRamo}>Elimina ramo</button>
      </div>
    {/if}
  </div>
</div>
