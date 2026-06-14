// Versione dell'app e changelog, mostrati nel pannello Info.
export const VERSIONE = "0.2.0";

export const changelog = [
  {
    versione: "0.2.0",
    note: [
      "Stage / unstage / scarta del singolo blocco (hunk) nel diff",
      "Amend dell'ultimo commit",
      "Stash: metti da parte, ripristina (pop) ed elimina le modifiche",
      "Tag: crea (leggere o annotate) ed elimina",
      "Cronologia: lista dei file di un commit e diff per singolo file",
      "Reset a un commit (soft / mixed / hard) e cherry-pick",
      "Impostazione di nome ed email dell'autore",
    ],
  },
  {
    versione: "0.1.0",
    note: [
      "Apri / inizializza / clona un repository",
      "Stato dei file: stage, unstage, scarta modifiche",
      "Commit con i file in staging",
      "Cronologia dei commit",
      "Rami: elenco, crea, cambia, elimina, merge",
      "Diff colorato dei file e dei commit",
      "Fetch / Pull / Push verso il remoto",
      "Elenco dei repository recenti",
    ],
  },
];
