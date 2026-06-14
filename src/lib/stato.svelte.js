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
  avvisa(testo, tipo = "") {
    this.nota = testo;
    this.tipoNota = tipo;
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => (this.nota = ""), 3500);
  }
}

export const stato = new Stato();
