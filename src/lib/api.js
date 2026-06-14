// Ponte verso il backend Tauri: ogni funzione qui sotto richiama un comando
// Rust via `invoke`. La UI usa solo queste funzioni, mai `invoke` direttamente.
import { invoke } from "@tauri-apps/api/core";

// ---- Apertura repository ----
export const apriRepo = (percorso) => invoke("apri_repo", { percorso });
export const initRepo = (percorso) => invoke("init_repo", { percorso });
export const clona = (url, destinazione) => invoke("clona", { url, destinazione });

// ---- Stato e cronologia ----
export const stato = (percorso) => invoke("stato", { percorso });
export const log = (percorso, limite = 100) => invoke("log", { percorso, limite });

// ---- Staging ----
export const stageAggiungi = (percorso, file) =>
  invoke("stage_aggiungi", { percorso, file });
export const stageAggiungiTutto = (percorso) =>
  invoke("stage_aggiungi_tutto", { percorso });
export const stageTogli = (percorso, file) => invoke("stage_togli", { percorso, file });
export const stageTogliTutto = (percorso) => invoke("stage_togli_tutto", { percorso });
export const scarta = (percorso, file) => invoke("scarta", { percorso, file });

// ---- Commit ----
export const creaCommit = (percorso, messaggio, nome = "", email = "") =>
  invoke("crea_commit", { percorso, messaggio, nome, email });
export const amend = (percorso, messaggio) => invoke("amend", { percorso, messaggio });
export const ultimoMessaggio = (percorso) => invoke("ultimo_messaggio", { percorso });

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

// ---- Remoti ----
export const remotiLista = (percorso) => invoke("remoti_lista", { percorso });
export const fetch = (percorso, remoto = "origin") =>
  invoke("fetch", { percorso, remoto });
export const pull = (percorso, remoto = "origin") => invoke("pull", { percorso, remoto });
export const push = (percorso, remoto = "origin") => invoke("push", { percorso, remoto });

// ---- Repository recenti ----
export const recentiLista = () => invoke("recenti_lista");
export const recentiAggiungi = (percorso) => invoke("recenti_aggiungi", { percorso });
export const recentiRimuovi = (percorso) => invoke("recenti_rimuovi", { percorso });
