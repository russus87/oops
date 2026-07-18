<script>
  // Modale per inserire le credenziali quando un'operazione di rete fallisce
  // per autenticazione. Niente viene salvato: i dati servono solo al retry.
  import { open } from "@tauri-apps/plugin-dialog";
  import { stato } from "../lib/stato.svelte.js";

  let tipo = $state("https"); // "https" | "ssh"
  let utente = $state("");
  let password = $state("");
  let chiave = $state("");
  let passphrase = $state("");

  function conferma() {
    const cred =
      tipo === "https"
        ? { utente: utente || null, password: password || null }
        : { chiave: chiave || null, passphrase: passphrase || null };
    azzera();
    stato.inviaCredenziali(cred);
  }

  function annulla() {
    azzera();
    stato.annullaCredenziali();
  }

  function azzera() {
    password = "";
    passphrase = "";
  }

  async function scegliChiave() {
    const f = await open({ title: "Scegli la chiave privata SSH" });
    if (f) chiave = f;
  }
</script>

<div class="overlay" onclick={annulla}>
  <div class="modale" onclick={(e) => e.stopPropagation()}>
    <h2>Servono le credenziali</h2>
    <div class="tabs-mini" style="margin-bottom:12px">
      <button class:attivo={tipo === "https"} onclick={() => (tipo = "https")}>HTTPS</button>
      <button class:attivo={tipo === "ssh"} onclick={() => (tipo = "ssh")}>Chiave SSH</button>
    </div>

    {#if tipo === "https"}
      <div class="campo">
        <label for="cu">Utente</label>
        <input id="cu" bind:value={utente} placeholder="nome utente" />
      </div>
      <div class="campo">
        <label for="cp">Password o token</label>
        <input id="cp" type="password" bind:value={password} placeholder="password / personal access token" />
      </div>
    {:else}
      <div class="campo">
        <label for="ck">File chiave privata</label>
        <div class="riga">
          <input id="ck" bind:value={chiave} placeholder="~/.ssh/id_ed25519" />
          <button onclick={scegliChiave}>Sfoglia</button>
        </div>
      </div>
      <div class="campo">
        <label for="cpp">Passphrase (se presente)</label>
        <input id="cpp" type="password" bind:value={passphrase} />
      </div>
    {/if}

    <div class="pulsanti">
      <button onclick={annulla}>Annulla</button>
      <button class="primario" onclick={conferma}>Riprova</button>
    </div>
  </div>
</div>
