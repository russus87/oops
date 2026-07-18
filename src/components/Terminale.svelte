<script>
  // Terminale git integrato: scrivi un comando git (senza "git") e lo esegue nel
  // repository, mostrando l'output. Dopo ogni comando la UI si ricarica, così lo
  // stato resta allineato. Per sicurezza esegue solo il binario `git`.
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";

  let comando = $state("");
  let righe = $state([]); // { cmd, out }
  let storia = $state([]); // comandi passati (per ↑/↓)
  let posStoria = $state(-1);
  let occupato = $state(false);

  // Spezza la riga in argomenti rispettando le virgolette semplici.
  function argomenti(s) {
    const m = s.match(/"[^"]*"|'[^']*'|\S+/g) || [];
    return m.map((a) => a.replace(/^["']|["']$/g, ""));
  }

  async function run(c) {
    if (!c || occupato) return;
    // Toglie un eventuale "git" iniziale digitato per abitudine.
    const args = argomenti(c).filter((a, i) => !(i === 0 && a === "git"));
    occupato = true;
    storia = [c, ...storia].slice(0, 50);
    posStoria = -1;
    try {
      const out = await api.eseguiGit(stato.percorso, args);
      righe = [...righe, { cmd: c, out }];
      stato.registra("git " + args.join(" "), "ok");
      stato.ricarica();
    } catch (e) {
      righe = [...righe, { cmd: c, out: String(e), err: true }];
    } finally {
      occupato = false;
    }
  }

  async function esegui() {
    const c = comando.trim();
    if (!c) return;
    comando = "";
    await run(c);
  }

  function tasti(e) {
    if (e.key === "Enter") { e.preventDefault(); esegui(); }
    else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (posStoria < storia.length - 1) { posStoria++; comando = storia[posStoria]; }
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      if (posStoria > 0) { posStoria--; comando = storia[posStoria]; }
      else { posStoria = -1; comando = ""; }
    }
  }
</script>

<div class="terminale-vista">
  {#if stato.azioniGit.length > 0}
    <div class="term-azioni">
      <span class="ta-eti">Azioni:</span>
      {#each stato.azioniGit as a}
        <button class="fantasma" title={"git " + a.comando} disabled={occupato} onclick={() => run(a.comando)}>{a.nome}</button>
      {/each}
    </div>
  {/if}
  <div class="term-uscita">
    {#if righe.length === 0}
      <div class="term-aiuto">
        Terminale git. Scrivi un comando (es. <code>status</code>, <code>log --oneline -5</code>,
        <code>diff</code>) e premi Invio. La UI si aggiorna dopo ogni comando.
      </div>
    {/if}
    {#each righe as r}
      <div class="term-blocco">
        <div class="term-prompt"><span class="tp-seg">git</span> {r.cmd.replace(/^git\s+/, "")}</div>
        <pre class="term-out" class:err={r.err}>{r.out}</pre>
      </div>
    {/each}
  </div>
  <div class="term-input">
    <span class="ti-seg">git</span>
    <input
      bind:value={comando}
      onkeydown={tasti}
      placeholder="status · log --oneline · diff · branch -a …"
      disabled={occupato}
      autocomplete="off"
    />
    <button class="primario" onclick={esegui} disabled={occupato}>Esegui</button>
  </div>
</div>
