// Ponte verso il backend Tauri: ogni funzione qui sotto richiama un comando
// Rust via `invoke`. La UI usa solo queste funzioni, mai `invoke` direttamente.
import { invoke } from "@tauri-apps/api/core";

// ---- Apertura repository ----
export const apriRepo = (percorso) => invoke("apri_repo", { percorso });
export const initRepo = (percorso) => invoke("init_repo", { percorso });
export const clona = (url, destinazione, cred = null) =>
  invoke("clona", { url, destinazione, cred });

// ---- Stato e cronologia ----
export const stato = (percorso) => invoke("stato", { percorso });
export const log = (percorso, limite = 100) => invoke("log", { percorso, limite });
export const logFile = (percorso, file, limite = 100) =>
  invoke("log_file", { percorso, file, limite });
export const blame = (percorso, file) => invoke("blame_file", { percorso, file });

// ---- Staging ----
export const stageAggiungi = (percorso, file) =>
  invoke("stage_aggiungi", { percorso, file });
export const stageAggiungiTutto = (percorso) =>
  invoke("stage_aggiungi_tutto", { percorso });
export const stageTogli = (percorso, file) => invoke("stage_togli", { percorso, file });
export const stageTogliTutto = (percorso) => invoke("stage_togli_tutto", { percorso });
export const scarta = (percorso, file) => invoke("scarta", { percorso, file });
export const scartaTutto = (percorso) => invoke("scarta_tutto", { percorso });
export const pulisciNonTracciati = (percorso) =>
  invoke("pulisci_non_tracciati", { percorso });

// ---- Commit ----
export const creaCommit = (percorso, messaggio, nome = "", email = "") =>
  invoke("crea_commit", { percorso, messaggio, nome, email });
export const amend = (percorso, messaggio) => invoke("amend", { percorso, messaggio });
export const ultimoMessaggio = (percorso) => invoke("ultimo_messaggio", { percorso });
export const condensa = (percorso, id, messaggio) =>
  invoke("condensa", { percorso, id, messaggio });
export const ripristinaFile = (percorso, id, file) =>
  invoke("ripristina_file", { percorso, id, file });

// ---- Diff ----
export const diffFile = (percorso, file, inStage) =>
  invoke("diff_file", { percorso, file, inStage });
export const diffCommit = (percorso, id) => invoke("diff_commit", { percorso, id });
export const listaFileCommit = (percorso, id) =>
  invoke("lista_file_commit", { percorso, id });
export const diffCommitFile = (percorso, id, file) =>
  invoke("diff_commit_file", { percorso, id, file });

// ---- Diff per hunk ----
export const hunkStage = (percorso, file, indice, inStage) =>
  invoke("hunk_stage", { percorso, file, indice, inStage });
export const hunkScarta = (percorso, file, indice) =>
  invoke("hunk_scarta", { percorso, file, indice });

// ---- Stash ----
export const stashLista = (percorso) => invoke("stash_lista", { percorso });
export const stashSalva = (percorso, messaggio, includiNonTracciati) =>
  invoke("stash_salva", { percorso, messaggio, includiNonTracciati });
export const stashApplica = (percorso, indice) =>
  invoke("stash_applica", { percorso, indice });
export const stashPop = (percorso, indice) => invoke("stash_pop", { percorso, indice });
export const stashElimina = (percorso, indice) =>
  invoke("stash_elimina", { percorso, indice });
export const stashDiff = (percorso, indice) => invoke("stash_diff", { percorso, indice });

// ---- Tag ----
export const tagLista = (percorso) => invoke("tag_lista", { percorso });
export const tagCrea = (percorso, nome, messaggio = "") =>
  invoke("tag_crea", { percorso, nome, messaggio });
export const tagElimina = (percorso, nome) => invoke("tag_elimina", { percorso, nome });

// ---- Azioni su commit ----
export const resetCommit = (percorso, id, modo) =>
  invoke("reset_commit", { percorso, id, modo });
export const cherryPick = (percorso, id) => invoke("cherry_pick", { percorso, id });
export const configUtente = (percorso) => invoke("config_utente", { percorso });
export const impostaConfigUtente = (percorso, nome, email) =>
  invoke("imposta_config_utente", { percorso, nome, email });

// ---- Rami ----
export const ramiLista = (percorso) => invoke("rami_lista", { percorso });
export const ramoCrea = (percorso, nome, cambia = true) =>
  invoke("ramo_crea", { percorso, nome, cambia });
export const ramoCheckout = (percorso, nome) =>
  invoke("ramo_checkout", { percorso, nome });
export const ramoElimina = (percorso, nome) => invoke("ramo_elimina", { percorso, nome });
export const ramoMerge = (percorso, nome) => invoke("ramo_merge", { percorso, nome });
export const ramoRebase = (percorso, su) => invoke("ramo_rebase", { percorso, su });
export const ramoCreaDa = (percorso, nome, id, cambia = true) =>
  invoke("ramo_crea_da", { percorso, nome, id, cambia });
export const ramoCheckoutCommit = (percorso, id) =>
  invoke("ramo_checkout_commit", { percorso, id });

// ---- Revert ----
export const revert = (percorso, id) => invoke("revert", { percorso, id });

// ---- Conflitti ----
export const conflittiLista = (percorso) => invoke("conflitti_lista", { percorso });
export const conflittoRisolvi = (percorso, file, lato) =>
  invoke("conflitto_risolvi", { percorso, file, lato });
export const conflittoSegnaRisolto = (percorso, file) =>
  invoke("conflitto_segna_risolto", { percorso, file });
export const operazioneAnnulla = (percorso) => invoke("operazione_annulla", { percorso });

// ---- Remoti ----
export const remotiLista = (percorso) => invoke("remoti_lista", { percorso });
export const remotiDettagli = (percorso) => invoke("remoti_dettagli", { percorso });
export const remotoAggiungi = (percorso, nome, url) =>
  invoke("remoto_aggiungi", { percorso, nome, url });
export const remotoImpostaUrl = (percorso, nome, url) =>
  invoke("remoto_imposta_url", { percorso, nome, url });
export const remotoRimuovi = (percorso, nome) => invoke("remoto_rimuovi", { percorso, nome });
export const eliminaRamoRemoto = (percorso, remoto, ramo, cred = null) =>
  invoke("elimina_ramo_remoto", { percorso, remoto, ramo, cred });
export const fetch = (percorso, remoto = "origin", cred = null) =>
  invoke("fetch", { percorso, remoto, cred });
export const pull = (percorso, remoto = "origin", strategia = "ff", cred = null) =>
  invoke("pull", { percorso, remoto, strategia, cred });
export const push = (percorso, remoto = "origin", forza = false, cred = null) =>
  invoke("push", { percorso, remoto, forza, cred });
export const pushTags = (percorso, remoto = "origin", cred = null) =>
  invoke("push_tags", { percorso, remoto, cred });

// ---- Repository recenti ----
export const recentiLista = () => invoke("recenti_lista");
export const recentiAggiungi = (percorso) => invoke("recenti_aggiungi", { percorso });
export const recentiRimuovi = (percorso) => invoke("recenti_rimuovi", { percorso });
