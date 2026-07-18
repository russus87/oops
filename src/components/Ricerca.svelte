<script>
  // Ricerca globale (Ctrl+K): un'unica casella per trovare commit, rami, tag e
  // file. Clic su un risultato → azione (vai al commit / checkout / apri storia).
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";

  let q = $state("");
  let commit = $state([]);
  let rami = $state([]);
  let tag = $state([]);
  let file = $state([]);
  let sel = $state(0);
  let campo;

  // Carica i dati una volta all'apertura.
  $effect(() => {
    if (!stato.percorso) return;
    api.log(stato.percorso, 800).then((c) => (commit = c)).catch(() => {});
    api.ramiLista(stato.percorso).then((r) => (rami = r)).catch(() => {});
    api.tagLista(stato.percorso).then((t) => (tag = t)).catch(() => {});
    api.eseguiGit(stato.percorso, ["ls-files"])
      .then((s) => (file = s.split("\n").filter(Boolean)))
      .catch(() => {});
    setTimeout(() => campo?.focus(), 30);
  });

  const contiene = (s, t) => s.toLowerCase().includes(t);

  // Risultati unificati (max poche voci per categoria), ricalcolati alla ricerca.
  let risultati = $derived(calcola(q));

  function calcola(query) {
    const t = query.trim().toLowerCase();
    if (!t) return [];
    const out = [];
    for (const c of commit) {
      if (contiene(c.titolo, t) || contiene(c.autore, t) || c.id_breve.includes(t)) {
        out.push({ tipo: "commit", eti: c.titolo, sub: c.id_breve + " · " + c.autore, dato: c.id });
        if (out.filter((r) => r.tipo === "commit").length >= 6) break;
      }
    }
    for (const r of rami) {
      if (contiene(r.nome, t))
        out.push({ tipo: r.remoto ? "remoto" : "ramo", eti: r.nome, sub: r.remoto ? "ramo remoto" : "ramo locale", dato: r.nome });
    }
    for (const tg of tag) {
      if (contiene(tg.nome, t)) out.push({ tipo: "tag", eti: tg.nome, sub: "tag", dato: tg.nome });
    }
    for (const f of file) {
      if (contiene(f, t)) {
        out.push({ tipo: "file", eti: f, sub: "file", dato: f });
        if (out.filter((r) => r.tipo === "file").length >= 6) break;
      }
    }
    return out.slice(0, 24);
  }

  const icona = { commit: "◆", ramo: "⎇", remoto: "☁", tag: "🏷", file: "📄" };

  async function scegli(r) {
    stato.ricercaAperta = false;
    if (r.tipo === "commit") stato.vaiACommit(r.dato);
    else if (r.tipo === "ramo") {
      try { await api.ramoCheckout(stato.percorso, r.dato); stato.avvisa("Sei su " + r.dato, "ok"); stato.ricarica(); }
      catch (e) { stato.avvisa(String(e), "errore"); }
    } else if (r.tipo === "remoto" || r.tipo === "tag") {
      stato.vista = "cronologia";
    } else if (r.tipo === "file") {
      stato.storiaFile = r.dato;
    }
  }

  function tasti(e) {
    if (e.key === "Escape") stato.ricercaAperta = false;
    else if (e.key === "ArrowDown") { e.preventDefault(); sel = Math.min(sel + 1, risultati.length - 1); }
    else if (e.key === "ArrowUp") { e.preventDefault(); sel = Math.max(sel - 1, 0); }
    else if (e.key === "Enter" && risultati[sel]) { e.preventDefault(); scegli(risultati[sel]); }
  }
  $effect(() => { q; sel = 0; });
</script>

<div class="overlay ricerca-overlay" onclick={() => (stato.ricercaAperta = false)}>
  <div class="ricerca" onclick={(e) => e.stopPropagation()}>
    <div class="ric-campo">
      <span class="ric-lente">⌕</span>
      <input
        bind:this={campo}
        bind:value={q}
        onkeydown={tasti}
        placeholder="Cerca commit, rami, tag, file…"
        autocomplete="off"
      />
      <span class="ric-hint">Esc</span>
    </div>
    <div class="ric-risultati">
      {#if q.trim() && risultati.length === 0}
        <div class="ric-vuoto">Nessun risultato.</div>
      {/if}
      {#each risultati as r, i}
        <div class="ric-voce" class:sel={i === sel} onmouseenter={() => (sel = i)} onclick={() => scegli(r)}>
          <span class="ric-ico {r.tipo}">{icona[r.tipo]}</span>
          <span class="ric-eti">{r.eti}</span>
          <span class="ric-sub">{r.sub}</span>
        </div>
      {/each}
      {#if !q.trim()}
        <div class="ric-vuoto">Scrivi per cercare in tutto il repository.</div>
      {/if}
    </div>
  </div>
</div>
