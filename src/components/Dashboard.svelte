<script>
  // Dashboard "Repository Health": la prima cosa che vedi aprendo un repository.
  // Riassume in colpo d'occhio ramo, sincronia col remoto, ultimo fetch, conflitti,
  // stato della working tree, conteggi e ultimo commit. Le tessere sono cliccabili
  // per saltare alla sezione giusta.
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";
  import { iniziali, coloreAvatar, tempoRelativo } from "../lib/util.js";

  let { vai } = $props(); // callback per cambiare vista: vai("modifiche" | "cronologia")

  let p = $state(null);

  $effect(() => {
    stato.tic;
    if (!stato.percorso) return;
    api.panoramica(stato.percorso).then((d) => (p = d)).catch(() => (p = null));
  });

  // Salute del repo: verde se tutto tranquillo, giallo se c'è roba da sistemare,
  // rosso se ci sono conflitti in corso.
  let salute = $derived(
    !p
      ? { classe: "", testo: "…", icona: "" }
      : p.conflitti > 0
        ? { classe: "rosso", testo: "Conflitti da risolvere", icona: "⚠" }
        : p.indietro > 0 || p.modificati + p.in_stage + p.non_tracciati > 0
          ? { classe: "giallo", testo: "Ci sono modifiche da gestire", icona: "●" }
          : { classe: "verde", testo: "Tutto in ordine", icona: "✓" }
  );

  const fmtFetch = (ts) => (ts > 0 ? tempoRelativo(ts) : "mai");
</script>

<div class="dashboard">
  {#if p}
    <div class="dash-testa">
      <div class="dash-salute {salute.classe}">
        <span class="ds-icona">{salute.icona}</span>
        <div>
          <div class="ds-titolo">{salute.testo}</div>
          <div class="ds-sub">
            <span class="ramo-corr">⎇ {p.ramo}</span>
            {#if p.upstream}<span class="ds-up">→ {p.upstream}</span>{/if}
            {#if p.vuoto}<span class="ds-up">nessun commit</span>{/if}
          </div>
        </div>
      </div>
    </div>

    <div class="dash-griglia">
      <!-- Sincronia col remoto -->
      <div class="tessera">
        <div class="t-tit">Sincronia</div>
        <div class="t-grande">
          <span class="t-num" class:attivo={p.avanti > 0}>↑ {p.avanti}</span>
          <span class="t-num" class:attivo={p.indietro > 0}>↓ {p.indietro}</span>
        </div>
        <div class="t-sub">
          {p.avanti === 0 && p.indietro === 0
            ? "allineato al remoto"
            : `${p.avanti} da spingere · ${p.indietro} da scaricare`}
        </div>
      </div>

      <!-- Ultimo fetch -->
      <div class="tessera">
        <div class="t-tit">Ultimo fetch</div>
        <div class="t-grande"><span class="t-testo">{fmtFetch(p.ultimo_fetch)}</span></div>
        <div class="t-sub">{p.remoti.length} remot{p.remoti.length === 1 ? "o" : "i"} configurat{p.remoti.length === 1 ? "o" : "i"}</div>
      </div>

      <!-- Conflitti -->
      <div class="tessera" class:pericolo={p.conflitti > 0}
        onclick={() => p.conflitti > 0 && vai("modifiche")}
        style={p.conflitti > 0 ? "cursor:pointer" : ""}>
        <div class="t-tit">Conflitti</div>
        <div class="t-grande"><span class="t-num" class:attivo={p.conflitti > 0}>{p.conflitti}</span></div>
        <div class="t-sub">{p.conflitti > 0 ? "clic per risolvere" : "nessun conflitto"}</div>
      </div>

      <!-- Working tree -->
      <div class="tessera" onclick={() => vai("modifiche")} style="cursor:pointer">
        <div class="t-tit">Working tree</div>
        <div class="t-righe">
          <span class="pill stage">{p.in_stage} in stage</span>
          <span class="pill mod">{p.modificati} modificati</span>
          <span class="pill nuovo">{p.non_tracciati} nuovi</span>
        </div>
        <div class="t-sub">clic per aprire le Modifiche</div>
      </div>

      <!-- Conteggi -->
      <div class="tessera">
        <div class="t-tit">Rami & tag</div>
        <div class="t-righe">
          <span class="pill">⎇ {p.rami_locali} locali</span>
          <span class="pill">☁ {p.rami_remoti} remoti</span>
          <span class="pill">🏷 {p.tag} tag</span>
          <span class="pill">📦 {p.stash} stash</span>
        </div>
      </div>

      <!-- Ultimo commit -->
      {#if p.ultimo_commit}
        <div class="tessera larga" onclick={() => vai("cronologia")} style="cursor:pointer">
          <div class="t-tit">Ultimo commit</div>
          <div class="ultimo-commit">
            <div class="avatar" style="background:{coloreAvatar(p.ultimo_commit.email || p.ultimo_commit.autore)}">
              {iniziali(p.ultimo_commit.autore)}
            </div>
            <div class="uc-corpo">
              <div class="uc-titolo">{p.ultimo_commit.titolo}</div>
              <div class="uc-meta">
                <span class="hash">{p.ultimo_commit.id_breve}</span>
                <span>{p.ultimo_commit.autore}</span>
                <span title={p.ultimo_commit.data}>{tempoRelativo(p.ultimo_commit.timestamp)}</span>
              </div>
            </div>
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <div class="diff-vuoto">Carico la panoramica…</div>
  {/if}
</div>
