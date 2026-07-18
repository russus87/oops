<script>
  // Mostra "+aggiunte −rimozioni" e una barretta proporzionale verde/rossa per un
  // file, come nel pannello "Files Changed" dei client moderni. `stat` è
  // { aggiunte, rimozioni, binario } oppure null (niente da mostrare).
  let { stat = null } = $props();
  const N = 5; // numero di segmenti della barretta
  let segmenti = $derived(calcola(stat));

  function calcola(s) {
    if (!s || s.binario) return [];
    const tot = s.aggiunte + s.rimozioni;
    if (tot === 0) return [];
    const verdi = Math.round((s.aggiunte / tot) * N);
    return Array.from({ length: N }, (_, i) => (i < verdi ? "agg" : "rim"));
  }
</script>

{#if stat}
  <span class="barrastat">
    {#if stat.binario}
      <span class="bs-bin">bin</span>
    {:else}
      <span class="bs-num agg">+{stat.aggiunte}</span>
      <span class="bs-num rim">−{stat.rimozioni}</span>
      <span class="bs-barra">
        {#each segmenti as c}<span class="bs-seg {c}"></span>{/each}
      </span>
    {/if}
  </span>
{/if}
