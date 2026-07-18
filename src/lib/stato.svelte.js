// Stato globale dell'app, condiviso tra i componenti tramite le "runes" di
// Svelte 5. Tiene il repository aperto e fa da bacheca per i messaggi (toast).

class Stato {
  // Percorso del repository aperto (null = nessun repo, mostra la schermata avvio).
  percorso = $state(null);
  // Nome breve del repository (ultima parte del percorso).
  nome = $state("");
  // Contatore: ogni volta che cambia, i componenti ricaricano i dati.
  tic = $state(0);
  // true mentre è in corso un'operazione di rete (fetch/pull/push).
  occupato = $state(false);

  // Messaggio temporaneo in basso (toast).
  nota = $state("");
  tipoNota = $state(""); // "", "ok", "errore"
  #timer = null;

  // Tema dell'interfaccia ("scuro" o "chiaro"), ricordato tra le sessioni.
  tema = $state(localStorage.getItem("oops-tema") || "scuro");

  // Se true, i diff ignorano le differenze di soli spazi.
  ignoraSpazi = $state(localStorage.getItem("oops-ignora-spazi") === "1");

  // Impostazioni AI (Anthropic). Il token resta solo su questo computer
  // (localStorage), non viene mai salvato nel repository.
  aiToken = $state(localStorage.getItem("oops-ai-token") || "");
  aiModello = $state(localStorage.getItem("oops-ai-modello") || "claude-sonnet-5");

  impostaAi(token, modello) {
    this.aiToken = token;
    this.aiModello = modello;
    localStorage.setItem("oops-ai-token", token);
    localStorage.setItem("oops-ai-modello", modello);
  }

  // Azioni personalizzate (comandi git salvati), eseguibili dal Terminale.
  azioniGit = $state(JSON.parse(localStorage.getItem("oops-azioni") || "[]"));
  salvaAzioni() {
    localStorage.setItem("oops-azioni", JSON.stringify(this.azioniGit));
  }
  aggiungiAzione(nome, comando) {
    this.azioniGit = [...this.azioniGit, { nome, comando }];
    this.salvaAzioni();
  }
  rimuoviAzione(i) {
    this.azioniGit = this.azioniGit.filter((_, k) => k !== i);
    this.salvaAzioni();
  }

  // Oid a cui tornare con "Rifai" (redo), impostato prima di un Undo.
  redoOid = $state(null);

  // Se true, le operazioni di rete NON verificano il certificato TLS/host SSH
  // (per server interni con certificati self-signed). Riduce la sicurezza.
  tlsInsicuro = $state(localStorage.getItem("oops-tls-insicuro") === "1");
  cambiaTlsInsicuro(v) {
    this.tlsInsicuro = v;
    localStorage.setItem("oops-tls-insicuro", v ? "1" : "0");
  }

  // Vista corrente dell'area di lavoro. La controllano sia la barra laterale
  // (navigazione) sia le scorciatoie da tastiera.
  // "panoramica" | "modifiche" | "cronologia" | "insights" | "timeline" | "terminale"
  vista = $state("panoramica");

  // Elemento trascinato (drag&drop). Può essere un commit { tipo:"commit", id, breve }
  // o un ramo { tipo:"ramo", nome }. Null quando non si trascina nulla.
  trascina = $state(null);

  // Mostra la heat map sul grafo (colora i nodi per quantità di modifiche).
  heatMap = $state(false);

  // Palette di ricerca globale (Ctrl+K) aperta/chiusa.
  ricercaAperta = $state(false);

  // Commit su cui la cronologia deve posizionarsi (impostato dalla ricerca).
  commitScelto = $state(null);

  // File di cui aprire la cronologia/blame (modale). Null = chiusa.
  storiaFile = $state(null);

  // Vai a un commit specifico nella cronologia.
  vaiACommit(id) {
    this.commitScelto = id;
    this.vista = "cronologia";
  }

  // Timeline delle azioni della sessione (fetch, commit, push, checkout…), per
  // capire "cosa ho fatto". Solo in memoria: si azzera alla chiusura.
  azioni = $state([]);

  // Registra un'azione nella timeline (con orario locale hh:mm).
  registra(testo, tipo = "") {
    const d = new Date();
    const ora =
      String(d.getHours()).padStart(2, "0") + ":" + String(d.getMinutes()).padStart(2, "0");
    this.azioni = [{ ora, testo, tipo }, ...this.azioni].slice(0, 100);
  }

  cambiaIgnoraSpazi() {
    this.ignoraSpazi = !this.ignoraSpazi;
    localStorage.setItem("oops-ignora-spazi", this.ignoraSpazi ? "1" : "0");
    this.ricarica();
  }

  // Applica il tema corrente al documento.
  applicaTema() {
    document.documentElement.dataset.tema = this.tema;
  }

  // Passa da scuro a chiaro e viceversa.
  cambiaTema() {
    this.tema = this.tema === "scuro" ? "chiaro" : "scuro";
    localStorage.setItem("oops-tema", this.tema);
    this.applicaTema();
  }

  // Apre un repository e ne forza il caricamento.
  apri(percorso) {
    this.percorso = percorso;
    this.nome = percorso.replace(/[\\/]+$/, "").split(/[\\/]/).pop() || percorso;
    this.tic++;
  }

  // Torna alla schermata di avvio.
  chiudi() {
    this.percorso = null;
    this.nome = "";
  }

  // Forza tutti i componenti a rileggere i dati dal backend.
  ricarica() {
    this.tic++;
  }

  // --- Richiesta credenziali (modale gestita da App) ---
  credAperta = $state(false);
  #credResolve = null;

  // Apre la modale credenziali e restituisce una Promise con le credenziali
  // inserite (o null se l'utente annulla).
  chiediCredenziali() {
    this.credAperta = true;
    return new Promise((res) => (this.#credResolve = res));
  }
  inviaCredenziali(cred) {
    this.credAperta = false;
    this.#credResolve?.(cred);
    this.#credResolve = null;
  }
  annullaCredenziali() {
    this.credAperta = false;
    this.#credResolve?.(null);
    this.#credResolve = null;
  }

  // Mostra un messaggio temporaneo. tipo = "" | "ok" | "errore".
  // Gli esiti significativi (ok/errore) finiscono anche nella timeline.
  avvisa(testo, tipo = "") {
    this.nota = testo;
    this.tipoNota = tipo;
    if (tipo === "ok" || tipo === "errore") this.registra(testo, tipo);
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => (this.nota = ""), 3500);
  }
}

export const stato = new Stato();
