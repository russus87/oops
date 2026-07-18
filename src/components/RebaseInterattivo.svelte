<script>
  // Editor del piano di rebase interattivo: per ogni commit (dal più vecchio al
  // più recente) si sceglie pick/squash/reword/drop, si riordina e si rinomina.
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";

  let { base, commits, chiudi } = $props();

  // Copia locale modificabile (ordine: dal più vecchio al più recente).
  let righe = $state(
    commits.map((c) => ({
      id: c.id,
      id_breve: c.id_breve,
      titolo: c.titolo,
      azione: "pick",
      messaggio: c.titolo,
    }))
  );

  function su(i) {
    if (i === 0) return;
    [righe[i - 1], righe[i]] = [righe[i], righe[i - 1]];
  }
  function giu(i) {
    if (i === righe.length - 1) return;
    [righe[i + 1], righe[i]] = [righe[i], righe[i + 1]];
  }

  // Riordino con drag&drop delle righe.
  let trascina = $state(-1);
  let sopra = $state(-1);
  function muovi(da, a) {
    if (da === a || da < 0) return;
    const copia = [...righe];
    const [el] = copia.splice(da, 1);
    copia.splice(a, 0, el);
    righe = copia;
  }

  async function esegui() {
    const mosse = righe.map((r) => ({
      id: r.id,
      azione: r.azione,
      messaggio: r.azione === "reword" || r.azione === "squash" ? r.messaggio : null,
    }));
    try {
      const esito = await api.rebaseInterattivo(stato.percorso, base, mosse);
      stato.avvisa(esito, "ok");
      chiudi();
      stato.ricarica();
    } catch (e) {
      stato.avvisa(String(e), "errore");
    }
  }
</script>

<div class="overlay" onclick={chiudi}>
  <div class="modale grande" onclick={(e) => e.stopPropagation()}>
    <div class="modale-testa">
      <h2>Rebase interattivo</h2>
      <button class="fantasma" onclick={chiudi}>✕</button>
    </div>
    <div class="modale-corpo">
      <p style="color:var(--testo2);font-size:12px;margin-top:0">
        Dall'alto (più vecchio) in basso (più recente). "squash" unisce nel commit sopra.
      </p>
      {#each righe as r, i}
        <div
          class="rb-riga"
          class:sopra={sopra === i}
          class:trascinata={trascina === i}
          draggable="true"
          ondragstart={() => (trascina = i)}
          ondragover={(e) => { e.preventDefault(); sopra = i; }}
          ondragleave={() => (sopra === i && (sopra = -1))}
          ondrop={(e) => { e.preventDefault(); muovi(trascina, i); trascina = -1; sopra = -1; }}
          ondragend={() => { trascina = -1; sopra = -1; }}
        >
          <span class="rb-drag" title="Trascina per riordinare">⠿</span>
          <span class="rb-ord">
            <button class="fantasma" onclick={() => su(i)}>▲</button>
            <button class="fantasma" onclick={() => giu(i)}>▼</button>
          </span>
          <select bind:value={r.azione}>
            <option value="pick">pick</option>
            <option value="reword">reword</option>
            <option value="squash">squash</option>
            <option value="drop">drop</option>
          </select>
          <span class="hash">{r.id_breve}</span>
          {#if r.azione === "reword" || r.azione === "squash"}
            <input bind:value={r.messaggio} />
          {:else}
            <span class="rb-tit" class:drop={r.azione === "drop"}>{r.titolo}</span>
          {/if}
        </div>
      {/each}
    </div>
    <div class="pulsanti" style="padding:12px 18px">
      <button onclick={chiudi}>Annulla</button>
      <button class="primario" onclick={esegui}>Esegui rebase</button>
    </div>
  </div>
</div>
