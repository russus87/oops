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

/// Applica un commit su un ramo diverso da quello corrente: prima passa a
/// `ramo` (checkout) e poi esegue il cherry-pick. È la logica dietro il
/// drag&drop di un commit su un ramo nella barra laterale. Se il ramo è già
/// quello corrente si comporta come un normale cherry-pick.
pub fn cherry_pick_su(percorso: &str, id: &str, ramo: &str) -> Result<(), String> {
    crate::rami::checkout(percorso, ramo)?;
    cherry_pick(percorso, id)
}

/// Cherry-pick "squash": applica le modifiche di un commit su un ramo e le
/// fonde nell'ultimo commit di quel ramo (amend), senza creare un commit nuovo.
pub fn cherry_pick_squash(percorso: &str, id: &str, ramo: &str) -> Result<(), String> {
    crate::rami::checkout(percorso, ramo)?;
    let repo = crate::apri(percorso)?;
    let oid = Oid::from_str(id).map_err(|e| e.to_string())?;
    let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;
    let tip = repo
        .head()
        .map_err(|e| e.to_string())?
        .peel_to_commit()
        .map_err(|e| e.to_string())?;

    let mut index = repo
        .cherrypick_commit(&commit, &tip, 0, None)
        .map_err(|e| e.to_string())?;
    if index.has_conflicts() {
        return Err("cherry-pick squash con conflitti: risolvi a mano".into());
    }
    let albero_oid = index.write_tree_to(&repo).map_err(|e| e.to_string())?;
    let albero = repo.find_tree(albero_oid).map_err(|e| e.to_string())?;
    tip.amend(Some("HEAD"), None, None, None, None, Some(&albero))
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Cherry-pick "move": copia il commit sul ramo di destinazione e poi lo rimuove
/// dal ramo di partenza (dove ci si trovava). Solo storie lineari.
pub fn cherry_pick_muovi(percorso: &str, id: &str, ramo: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let sorgente = crate::repo::nome_ramo(&repo);
    if sorgente == ramo {
        return Err("origine e destinazione coincidono".into());
    }
    if sorgente.contains("staccata") {
        return Err("non su un ramo: impossibile spostare".into());
    }
    drop(repo);
    crate::azioni::cherry_pick_su(percorso, id, ramo)?;
    crate::rami::checkout(percorso, &sorgente)?;
    crate::rebase_int::rimuovi(percorso, id)?;
    Ok(())
}

/// Ripristina un singolo file alla versione che aveva in un certo commit
/// (git restore --source <commit> <file>): aggiorna cartella e stage.
pub fn ripristina_file(percorso: &str, id: &str, file: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let oid = Oid::from_str(id).map_err(|e| e.to_string())?;
    let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;
    let albero = commit.tree().map_err(|e| e.to_string())?;

    let mut co = CheckoutBuilder::new();
    co.path(file).force().update_index(true);
    repo.checkout_tree(albero.as_object(), Some(&mut co))
        .map_err(|e| e.to_string())
}

/// Annulla un commit creandone uno nuovo che ne inverte le modifiche
/// (git revert). In caso di conflitti restituisce un errore.
pub fn revert(percorso: &str, id: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let oid = Oid::from_str(id).map_err(|e| e.to_string())?;
    let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;

    repo.revert(&commit, None).map_err(|e| e.to_string())?;

    if repo.index().map_err(|e| e.to_string())?.has_conflicts() {
        return Err("revert con conflitti: risolvi i file e fai un commit".into());
    }

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
    let firma = repo
        .signature()
        .or_else(|_| git2::Signature::now("Oops", "oops@local"))
        .map_err(|e| e.to_string())?;
    let titolo = commit.summary().unwrap_or("commit");

    repo.commit(
        Some("HEAD"),
        &firma,
        &firma,
        &format!("Revert \"{titolo}\""),
        &albero,
        &[&testa],
    )
    .map_err(|e| e.to_string())?;

    repo.cleanup_state().map_err(|e| e.to_string())
}

/// Annulla l'ultima operazione che ha mosso HEAD (commit, merge, reset, rebase,
/// checkout…) riportando il ramo allo stato precedente registrato nel reflog.
/// È l'"undo universale" one-click. ATTENZIONE: usa un reset hard, quindi le
/// modifiche non salvate della working copy possono andare perse.
pub fn annulla_ultima(percorso: &str) -> Result<String, String> {
    let repo = crate::apri(percorso)?;
    let reflog = repo.reflog("HEAD").map_err(|e| e.to_string())?;
    let voce = reflog.get(0).ok_or("niente da annullare")?;
    let vecchio = voce.id_old();
    if vecchio.is_zero() {
        return Err("niente da annullare (è la prima operazione)".into());
    }
    let descrizione = voce.message().unwrap_or("operazione").to_string();
    let oggetto = repo.find_object(vecchio, None).map_err(|e| e.to_string())?;
    repo.reset(&oggetto, ResetType::Hard, None)
        .map_err(|e| e.to_string())?;
    Ok(format!("Annullato: {descrizione}"))
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
