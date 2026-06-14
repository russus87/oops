// Versione dell'app e changelog, mostrati nel pannello Info.
export const VERSIONE = "0.3.0";

export const changelog = [
  {
    versione: "0.3.0",
    note: [
      "Risoluzione conflitti: tieni nostra/loro o segna risolto, e annulla l'operazione",
      "Rebase del ramo corrente su un altro ramo",
      "Revert di un commit; checkout di un commit; crea ramo da un commit",
      "Blame riga per riga e cronologia per singolo file",
      "Diff affiancato (side-by-side) oltre a quello unificato",
      "Decorazioni: rami e tag mostrati sui commit nella cronologia",
      "Gestione remoti (aggiungi/modifica/rimuovi) + push --force e push tag",
      "Elimina ramo sul remoto",
      "Scarta tutto e pulisci i file non tracciati",
      "Pannello Impostazioni con tema chiaro/scuro e info/changelog",
    ],
  },
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
