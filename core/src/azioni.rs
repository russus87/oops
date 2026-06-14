//! Azioni varie su singoli commit: reset, cherry-pick, e lettura/scrittura
//! del nome/email dell'autore (config di Git).

use git2::{build::CheckoutBuilder, Oid, ResetType};

use crate::model::ConfigUtente;

/// Riporta il ramo corrente a un commit. `modo`:
/// - "soft":  sposta solo HEAD (le modifiche restano in stage)
/// - "mixed": sposta HEAD e svuota lo stage (le modifiche restano nei file)
/// - "hard":  sposta HEAD e BUTTA le modifiche (distruttivo!)
pub fn reset(percorso: &str, id: &str, modo: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let oid = Oid::from_str(id).map_err(|e| e.to_string())?;
    let oggetto = repo
        .find_commit(oid)
        .map_err(|e| e.to_string())?
        .into_object();

    let tipo = match modo {
        "soft" => ResetType::Soft,
        "hard" => ResetType::Hard,
        _ => ResetType::Mixed,
    };

    let mut co = CheckoutBuilder::new();
    repo.reset(&oggetto, tipo, Some(&mut co))
        .map_err(|e| e.to_string())
}

/// Applica un commit sopra il ramo corrente e lo registra (git cherry-pick).
/// Mantiene messaggio e autore originali. In caso di conflitti restituisce
/// un errore: i file vanno risolti e poi si fa un commit a mano.
pub fn cherry_pick(percorso: &str, id: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let oid = Oid::from_str(id).map_err(|e| e.to_string())?;
    let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;

    // Applica le modifiche del commit a indice + cartella di lavoro.
    repo.cherrypick(&commit, None).map_err(|e| e.to_string())?;

    if repo.index().map_err(|e| e.to_string())?.has_conflicts() {
        return Err("cherry-pick con conflitti: risolvi i file e fai un commit".into());
    }

    // Crea il nuovo commit conservando messaggio e autore originali.
    let albero_id = repo
        .index()
        .map_err(|e| e.to_string())?
        .write_tree()
        .map_err(|e| e.to_string())?;
    let albero = repo.find_tree(albero_id).map_err(|e| e.to_string())?;
    let testa = repo
        .head()
        .map_err(|e| e.to_string())?
        .peel_to_commit()
        .map_err(|e| e.to_string())?;
    let committer = repo
        .signature()
        .or_else(|_| git2::Signature::now("Oops", "oops@local"))
        .map_err(|e| e.to_string())?;

    repo.commit(
        Some("HEAD"),
        &commit.author(),
        &committer,
        commit.message().unwrap_or("cherry-pick"),
        &albero,
        &[&testa],
    )
    .map_err(|e| e.to_string())?;

    repo.cleanup_state().map_err(|e| e.to_string())
}

/// Legge nome ed email dell'autore dalla configurazione di Git.
pub fn config_utente(percorso: &str) -> Result<ConfigUtente, String> {
    let repo = crate::apri(percorso)?;
    let cfg = repo.config().map_err(|e| e.to_string())?;
    Ok(ConfigUtente {
        nome: cfg.get_string("user.name").unwrap_or_default(),
        email: cfg.get_string("user.email").unwrap_or_default(),
    })
}

/// Imposta nome ed email dell'autore nella config locale del repository.
pub fn imposta_config_utente(percorso: &str, nome: &str, email: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let mut cfg = repo.config().map_err(|e| e.to_string())?;
    cfg.set_str("user.name", nome).map_err(|e| e.to_string())?;
    cfg.set_str("user.email", email).map_err(|e| e.to_string())
}
