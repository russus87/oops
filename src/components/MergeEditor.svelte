<script>
  // Editor di merge "a blocchi": mostra il file con i conflitti spezzati in
  // sezioni. Il testo fuori dai conflitti resta com'è; per ogni conflitto si
  // vedono affiancate la versione "Nostra" e "Loro" (più la Base, se disponibile)
  // e si sceglie con un clic: Accetta nostra / loro / entrambi, oppure si scrive
  // a mano il risultato. In fondo si salva il file risolto.
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";

  let { file, chiudi } = $props();

  let parti = $state([]); // sequenza di { tipo, ... } che ricostruisce il file
  let baseFile = $state("");

  $effect(() => {
    api.conflittoVersioni(stato.percorso, file).then((dati) => {
      baseFile = dati.base || "";
      parti = analizza(dati.corrente || "");
    });
  });

  // Spezza il contenuto (con i marcatori <<<<<<< ======= >>>>>>>) in parti:
  // testo normale e blocchi di conflitto (nostra / base / loro).
  function analizza(testo) {
    const righe = testo.split("\n");
    const out = [];
    let testoCorr = [];
    const scaricaTesto = () => {
      if (testoCorr.length) out.push({ tipo: "testo", righe: testoCorr });
      testoCorr = [];
    };
    let i = 0;
    while (i < righe.length) {
      const r = righe[i];
      if (r.startsWith("<<<<<<<")) {
        scaricaTesto();
        const nostra = [], base = [], loro = [];
        let sez = "nostra";
        i++;
        while (i < righe.length && !righe[i].startsWith(">>>>>>>")) {
          const l = righe[i];
          if (l.startsWith("|||||||")) sez = "base";
          else if (l.startsWith("=======")) sez = "loro";
          else if (sez === "nostra") nostra.push(l);
          else if (sez === "base") base.push(l);
          else loro.push(l);
          i++;
        }
        i++; // salta la riga >>>>>>>
        out.push({ tipo: "conflitto", nostra, base, loro, scelta: null, manuale: "" });
      } else {
        testoCorr.push(r);
        i++;
      }
    }
    scaricaTesto();
    return out;
  }

  const conflitti = $derived(parti.filter((p) => p.tipo === "conflitto"));
  const risolti = $derived(conflitti.filter((p) => p.scelta).length);
  const tuttoRisolto = $derived(conflitti.length > 0 && risolti === conflitti.length);

  function scegli(p, scelta) {
    p.scelta = scelta;
    if (scelta === "manuale" && !p.manuale) {
      // Precarica l'area manuale con "nostra" come punto di partenza.
      p.manuale = p.nostra.join("\n");
    }
    parti = parti; // notifica la reattività di Svelte
  }

  // Ricostruisce il contenuto finale del file secondo le scelte fatte.
  function componi() {
    const linee = [];
    for (const p of parti) {
      if (p.tipo === "testo") {
        linee.push(...p.righe);
      } else {
        if (p.scelta === "nostra") linee.push(...p.nostra);
        else if (p.scelta === "loro") linee.push(...p.loro);
        else if (p.scelta === "entrambe") linee.push(...p.nostra, ...p.loro);
        else if (p.scelta === "manuale") linee.push(...p.manuale.split("\n"));
      }
    }
    return linee.join("\n");
  }

  async function salva() {
    try {
      await api.conflittoSalva(stato.percorso, file, componi());
      stato.avvisa("Conflitto risolto: " + file, "ok");
      chiudi();
      stato.ricarica();
    } catch (e) {
      stato.avvisa("Salvataggio fallito: " + e, "errore");
    }
  }
</script>

<div class="overlay" onclick={chiudi}>
  <div class="modale grande merge" onclick={(e) => e.stopPropagation()}>
    <div class="modale-testa">
      <h2>Risolvi conflitto: {file}</h2>
      <span class="merge-conta" class:ok={tuttoRisolto}>
        {risolti}/{conflitti.length} risolti
      </span>
      <button class="fantasma" onclick={chiudi}>✕</button>
    </div>

    <div class="merge-blocchi">
      {#each parti as p}
        {#if p.tipo === "testo"}
          {#if p.righe.some((r) => r.trim() !== "")}
            <pre class="mb-contesto">{p.righe.join("\n")}</pre>
          {/if}
        {:else}
          <div class="mb-conflitto" class:risolto={p.scelta}>
            <div class="mb-barra">
              <span class="mb-etichetta">⚠ Conflitto</span>
              <span class="mb-spazio"></span>
              <button class:on={p.scelta === "nostra"} onclick={() => scegli(p, "nostra")}>Accetta nostra</button>
              <button class:on={p.scelta === "loro"} onclick={() => scegli(p, "loro")}>Accetta loro</button>
              <button class:on={p.scelta === "entrambe"} onclick={() => scegli(p, "entrambe")}>Entrambi</button>
              <button class:on={p.scelta === "manuale"} onclick={() => scegli(p, "manuale")}>Manuale ✎</button>
            </div>
            {#if p.scelta === "manuale"}
              <textarea class="mb-manuale" bind:value={p.manuale}></textarea>
            {:else}
              <div class="mb-colonne" class:con-base={p.base.length > 0}>
                <div class="mb-col nostra" class:vinta={p.scelta === "nostra" || p.scelta === "entrambe"}>
                  <div class="mb-tit">Nostra (HEAD)</div>
                  <pre>{p.nostra.join("\n")}</pre>
                </div>
                {#if p.base.length > 0}
                  <div class="mb-col base">
                    <div class="mb-tit">Base</div>
                    <pre>{p.base.join("\n")}</pre>
                  </div>
                {/if}
                <div class="mb-col loro" class:vinta={p.scelta === "loro" || p.scelta === "entrambe"}>
                  <div class="mb-tit">Loro (in arrivo)</div>
                  <pre>{p.loro.join("\n")}</pre>
                </div>
              </div>
            {/if}
          </div>
        {/if}
      {/each}
      {#if conflitti.length === 0}
        <div class="diff-vuoto">Nessun marcatore di conflitto in questo file.</div>
      {/if}
    </div>

    <div class="pulsanti" style="padding:12px 18px">
      <button onclick={chiudi}>Annulla</button>
      <button class="primario" disabled={!tuttoRisolto} onclick={salva}>
        {tuttoRisolto ? "Salva e risolvi" : `Risolvi tutti i conflitti (${risolti}/${conflitti.length})`}
      </button>
    </div>
  </div>
</div>
