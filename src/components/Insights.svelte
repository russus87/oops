<script>
  // Repository Insights: grafici calcolati localmente dal log (contributori,
  // attività per settimana e per giorno, linguaggi). Nessuna libreria esterna:
  // solo barre SVG/CSS, coerenti col tema.
  import * as api from "../lib/api.js";
  import { stato } from "../lib/stato.svelte.js";
  import { coloreAvatar, iniziali } from "../lib/util.js";

  let d = $state(null);

  $effect(() => {
    stato.tic;
    if (!stato.percorso) return;
    api.insights(stato.percorso, 1000).then((x) => (d = x)).catch(() => (d = null));
  });

  const giorni = ["Lun", "Mar", "Mer", "Gio", "Ven", "Sab", "Dom"];
  const max = (arr) => Math.max(1, ...arr);
</script>

<div class="insights-vista">
  {#if d}
    <div class="ins-griglia">
      <!-- Contributori -->
      <div class="ins-card">
        <div class="ins-tit">Top contributori</div>
        {#if d.contributori.length === 0}
          <div class="ins-vuoto">Nessun commit.</div>
        {/if}
        {#each d.contributori as c}
          <div class="ins-contrib">
            <div class="avatar" style="background:{coloreAvatar(c.etichetta)}">{iniziali(c.etichetta)}</div>
            <div class="ic-nome">{c.etichetta}</div>
            <div class="ic-barra">
              <div class="ic-fill" style="width:{(c.valore / d.contributori[0].valore) * 100}%"></div>
            </div>
            <div class="ic-num">{c.valore}</div>
          </div>
        {/each}
      </div>

      <!-- Attività per settimana -->
      <div class="ins-card">
        <div class="ins-tit">Attività (ultime 12 settimane)</div>
        <div class="ins-barre">
          {#each d.per_settimana as v}
            <div class="ib-col" title="{v} commit">
              <div class="ib-bar" style="height:{(v / max(d.per_settimana)) * 100}%"></div>
            </div>
          {/each}
        </div>
        <div class="ins-sub">{d.totale_commit} commit totali analizzati</div>
      </div>

      <!-- Attività per giorno della settimana -->
      <div class="ins-card">
        <div class="ins-tit">Per giorno della settimana</div>
        <div class="ins-barre alte">
          {#each d.per_giorno as v, i}
            <div class="ib-col" title="{giorni[i]}: {v} commit">
              <div class="ib-bar blu" style="height:{(v / max(d.per_giorno)) * 100}%"></div>
              <div class="ib-eti">{giorni[i]}</div>
            </div>
          {/each}
        </div>
      </div>

      <!-- File più modificati -->
      <div class="ins-card">
        <div class="ins-tit">File più modificati</div>
        {#if d.file_caldi.length === 0}
          <div class="ins-vuoto">Nessun dato.</div>
        {/if}
        {#each d.file_caldi as f}
          <div class="ins-contrib">
            <div class="ic-nome mono" title={f.etichetta}>{f.etichetta.split("/").pop()}</div>
            <div class="ic-barra">
              <div class="ic-fill" style="width:{(f.valore / d.file_caldi[0].valore) * 100}%"></div>
            </div>
            <div class="ic-num">{f.valore}</div>
          </div>
        {/each}
      </div>

      <!-- Righe totali -->
      <div class="ins-card">
        <div class="ins-tit">Righe cambiate (commit recenti)</div>
        <div class="t-grande" style="margin-top:6px">
          <span class="t-num" style="color:var(--ok)">+{d.aggiunte}</span>
          <span class="t-num" style="color:var(--pericolo)">−{d.rimozioni}</span>
        </div>
        <div class="ins-sub">Somma su un massimo di 300 commit recenti.</div>
      </div>

      <!-- Linguaggi -->
      <div class="ins-card">
        <div class="ins-tit">Linguaggi (per estensione)</div>
        {#if d.lingue.length === 0}
          <div class="ins-vuoto">Nessun file.</div>
        {/if}
        {#each d.lingue as l}
          <div class="ins-contrib">
            <div class="ic-nome mono">.{l.etichetta}</div>
            <div class="ic-barra">
              <div class="ic-fill verde" style="width:{(l.valore / d.lingue[0].valore) * 100}%"></div>
            </div>
            <div class="ic-num">{l.valore}</div>
          </div>
        {/each}
      </div>
    </div>
  {:else}
    <div class="diff-vuoto">Calcolo delle statistiche…</div>
  {/if}
</div>
