// Ponte verso il backend Tauri: ogni funzione qui sotto richiama un comando
// Rust via `invoke`. La UI usa solo queste funzioni, mai `invoke` direttamente.
import { invoke } from "@tauri-apps/api/core";
import { stato as statoApp } from "./stato.svelte.js";

// Flag "TLS insicuro" corrente (impostazione persistita). Le funzioni di rete
// lo passano al backend di default, così i chiamanti non devono preoccuparsene.
const insic = () => statoApp.tlsInsicuro;

// ---- Apertura repository ----
export const apriRepo = (percorso) => invoke("apri_repo", { percorso });
export const initRepo = (percorso) => invoke("init_repo", { percorso });
export const clona = (url, destinazione, cred = null) =>
  invoke("clona", { url, destinazione, cred, insicuro: insic() });

// ---- Stato e cronologia ----
export const stato = (percorso) => invoke("stato", { percorso });
export const panoramica = (percorso) => invoke("panoramica", { percorso });
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
export const diffFile = (percorso, file, inStage, ignoraSpazi = false) =>
  invoke("diff_file", { percorso, file, inStage, ignoraSpazi });
export const diffCommit = (percorso, id, ignoraSpazi = false) =>
  invoke("diff_commit", { percorso, id, ignoraSpazi });
export const diffCommitGenitore = (percorso, id, genitore, ignoraSpazi = false) =>
  invoke("diff_commit_genitore", { percorso, id, genitore, ignoraSpazi });
export const listaFileCommit = (percorso, id) =>
  invoke("lista_file_commit", { percorso, id });
export const diffCommitFile = (percorso, id, file, ignoraSpazi = false) =>
  invoke("diff_commit_file", { percorso, id, file, ignoraSpazi });

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
export const cherryPickSu = (percorso, id, ramo) =>
  invoke("cherry_pick_su", { percorso, id, ramo });
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
export const conflittoVersioni = (percorso, file) =>
  invoke("conflitto_versioni", { percorso, file });
export const conflittoSalva = (percorso, file, contenuto) =>
  invoke("conflitto_salva", { percorso, file, contenuto });

// ---- Rebase interattivo ----
export const rebaseInterattivo = (percorso, base, mosse) =>
  invoke("rebase_interattivo", { percorso, base, mosse });

// ---- Strumenti avanzati ----
export const reflogLista = (percorso) => invoke("reflog_lista", { percorso });
export const submoduliLista = (percorso) => invoke("submoduli_lista", { percorso });
export const submoduloAggiorna = (percorso, nome) =>
  invoke("submodulo_aggiorna", { percorso, nome });
export const worktreeLista = (percorso) => invoke("worktree_lista", { percorso });
export const worktreeAggiungi = (percorso, nome, cartella) =>
  invoke("worktree_aggiungi", { percorso, nome, cartella });
export const patchEsporta = (percorso, id, destinazione) =>
  invoke("patch_esporta", { percorso, id, destinazione });
export const patchApplica = (percorso, file) => invoke("patch_applica", { percorso, file });

// ---- Auto-refresh ----
export const avviaOsservatore = (percorso) => invoke("avvia_osservatore", { percorso });

// ---- Remoti ----
export const remotiLista = (percorso) => invoke("remoti_lista", { percorso });
export const remotiDettagli = (percorso) => invoke("remoti_dettagli", { percorso });
export const remotoAggiungi = (percorso, nome, url) =>
  invoke("remoto_aggiungi", { percorso, nome, url });
export const remotoImpostaUrl = (percorso, nome, url) =>
  invoke("remoto_imposta_url", { percorso, nome, url });
export const remotoRimuovi = (percorso, nome) => invoke("remoto_rimuovi", { percorso, nome });
export const eliminaRamoRemoto = (percorso, remoto, ramo, cred = null) =>
  invoke("elimina_ramo_remoto", { percorso, remoto, ramo, cred, insicuro: insic() });
export const fetch = (percorso, remoto = "origin", cred = null) =>
  invoke("fetch", { percorso, remoto, cred, insicuro: insic() });
export const pull = (percorso, remoto = "origin", strategia = "ff", cred = null) =>
  invoke("pull", { percorso, remoto, strategia, cred, insicuro: insic() });
export const push = (percorso, remoto = "origin", forza = false, cred = null) =>
  invoke("push", { percorso, remoto, forza, cred, insicuro: insic() });
export const pushTags = (percorso, remoto = "origin", cred = null) =>
  invoke("push_tags", { percorso, remoto, cred, insicuro: insic() });

// ---- Drag&drop ramo-su-ramo ----
export const mergeRami = (percorso, sorgente, destinazione) =>
  invoke("merge_rami", { percorso, sorgente, destinazione });
export const rebaseRami = (percorso, sorgente, destinazione) =>
  invoke("rebase_rami", { percorso, sorgente, destinazione });

// ---- Statistiche +/- e heat map ----
export const statLavoro = (percorso, inStage) =>
  invoke("stat_lavoro", { percorso, inStage });
export const statCommit = (percorso, id) => invoke("stat_commit", { percorso, id });
export const calore = (percorso, limite = 100) => invoke("calore", { percorso, limite });

// ---- Insights ----
export const insights = (percorso, limite = 500) =>
  invoke("insights", { percorso, limite });

// ---- Undo universale ----
export const annullaUltima = (percorso) => invoke("annulla_ultima", { percorso });

// ---- Terminale git integrato ----
export const eseguiGit = (percorso, args) => invoke("esegui_git", { percorso, args });

// ---- Compare, stage per riga, cherry-pick avanzato, release notes ----
export const diffTraCommit = (percorso, a, b, ignoraSpazi = false) =>
  invoke("diff_tra_commit", { percorso, a, b, ignoraSpazi });
export const stageRighe = (percorso, patch, reverse = false) =>
  invoke("stage_righe", { percorso, patch, reverse });
export const cherryPickSquash = (percorso, id, ramo) =>
  invoke("cherry_pick_squash", { percorso, id, ramo });
export const cherryPickMuovi = (percorso, id, ramo) =>
  invoke("cherry_pick_muovi", { percorso, id, ramo });
export const commitRimuovi = (percorso, id) => invoke("commit_rimuovi", { percorso, id });
export const noteRelease = (percorso, da, a) => invoke("note_release", { percorso, da, a });

// ---- Workspace ----
export const workspaceLista = () => invoke("workspace_lista");
export const workspaceSalva = (nome, percorsi) => invoke("workspace_salva", { nome, percorsi });
export const workspaceElimina = (nome) => invoke("workspace_elimina", { nome });

// ---- Anteprima file (markdown / immagini) ----
export const leggiTestoLavoro = (percorso, file) =>
  invoke("leggi_testo_lavoro", { percorso, file });
export const leggiB64Lavoro = (percorso, file) =>
  invoke("leggi_b64_lavoro", { percorso, file });
export const leggiB64Head = (percorso, file) =>
  invoke("leggi_b64_head", { percorso, file });

// ---- Assistente AI ----
export const generaCommitAi = (percorso, token, modello) =>
  invoke("genera_commit_ai", { percorso, token, modello });

// ---- Repository recenti ----
export const recentiLista = () => invoke("recenti_lista");
export const recentiAggiungi = (percorso) => invoke("recenti_aggiungi", { percorso });
export const recentiRimuovi = (percorso) => invoke("recenti_rimuovi", { percorso });
