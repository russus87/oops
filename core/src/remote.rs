//! Remoti: elenco e operazioni di rete (fetch, pull, push).
//!
//! Le credenziali vengono prese dall'agent SSH o dal credential helper di
//! sistema (lo stesso che usa `git` da terminale), così funziona con
//! GitHub/GitLab senza salvare password nell'app.

use git2::build::CheckoutBuilder;
use git2::{
    Cred, CredentialType, FetchOptions, PushOptions, RemoteCallbacks, Repository,
};

/// Callback per le credenziali, condiviso da clone/fetch/push.
pub fn credenziali(
    url: &str,
    utente: Option<&str>,
    permessi: CredentialType,
) -> Result<Cred, git2::Error> {
    if permessi.contains(CredentialType::SSH_KEY) {
        Cred::ssh_key_from_agent(utente.unwrap_or("git"))
    } else if permessi.contains(CredentialType::USER_PASS_PLAINTEXT) {
        let cfg = git2::Config::open_default()?;
        Cred::credential_helper(&cfg, url, utente)
    } else {
        Cred::default()
    }
}

/// Crea i callback di rete con la gestione credenziali già collegata.
fn callbacks() -> RemoteCallbacks<'static> {
    let mut cb = RemoteCallbacks::new();
    cb.credentials(credenziali);
    cb
}

/// Elenca i nomi dei remoti configurati (di solito "origin").
pub fn lista(percorso: &str) -> Result<Vec<String>, String> {
    let repo = crate::apri(percorso)?;
    let remoti = repo.remotes().map_err(|e| e.to_string())?;
    Ok(remoti.iter().flatten().map(|s| s.to_string()).collect())
}

/// Scarica gli aggiornamenti dal remoto senza toccare i file (git fetch).
pub fn fetch(percorso: &str, remoto: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let mut r = repo.find_remote(remoto).map_err(|e| e.to_string())?;
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(callbacks());
    // Passando una lista vuota di refspec, git2 usa quelle del remoto.
    let refspec: [&str; 0] = [];
    r.fetch(&refspec, Some(&mut fo), None).map_err(|e| e.to_string())
}

/// Scarica e integra le modifiche del ramo corrente (git pull).
/// Gestisce solo "già aggiornato" e fast-forward; se i due lati divergono
/// chiede di fare prima un merge/push manuale (per non creare pasticci).
pub fn pull(percorso: &str, remoto: &str) -> Result<String, String> {
    let repo = crate::apri(percorso)?;

    // Prima un fetch.
    fetch(percorso, remoto)?;

    // Ramo corrente e suo corrispondente remoto.
    let head = repo.head().map_err(|e| e.to_string())?;
    let ramo = head.shorthand().ok_or("ramo corrente sconosciuto")?.to_string();

    let rif_remoto = format!("refs/remotes/{remoto}/{ramo}");
    let riferimento = repo
        .find_reference(&rif_remoto)
        .map_err(|_| format!("nessun ramo remoto {remoto}/{ramo}"))?;
    let annotato = repo
        .reference_to_annotated_commit(&riferimento)
        .map_err(|e| e.to_string())?;

    let (analisi, _) = repo.merge_analysis(&[&annotato]).map_err(|e| e.to_string())?;

    if analisi.is_up_to_date() {
        return Ok("già aggiornato".into());
    }
    if analisi.is_fast_forward() {
        let nome_head = head.name().ok_or("HEAD senza nome")?.to_string();
        let mut rif = repo.find_reference(&nome_head).map_err(|e| e.to_string())?;
        rif.set_target(annotato.id(), "pull fast-forward")
            .map_err(|e| e.to_string())?;
        repo.set_head(&nome_head).map_err(|e| e.to_string())?;
        repo.checkout_head(Some(CheckoutBuilder::new().force()))
            .map_err(|e| e.to_string())?;
        return Ok("aggiornato (fast-forward)".into());
    }
    Err("le modifiche locali e remote divergono: serve un merge manuale".into())
}

/// Carica i commit del ramo corrente sul remoto (git push).
pub fn push(percorso: &str, remoto: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let head = repo.head().map_err(|e| e.to_string())?;
    let ramo = head.shorthand().ok_or("ramo corrente sconosciuto")?;

    let mut r = repo.find_remote(remoto).map_err(|e| e.to_string())?;
    let mut po = PushOptions::new();
    po.remote_callbacks(callbacks());

    let refspec = format!("refs/heads/{ramo}:refs/heads/{ramo}");
    r.push(&[&refspec], Some(&mut po)).map_err(|e| e.to_string())?;

    // Collega il ramo locale al suo upstream, così d'ora in poi conosciamo
    // l'avanti/indietro senza dover indovinare.
    imposta_upstream(&repo, ramo, remoto);
    Ok(())
}

/// Imposta l'upstream del ramo locale (branch.<ramo>.remote/merge in config).
fn imposta_upstream(repo: &Repository, ramo: &str, remoto: &str) {
    if let Ok(mut b) = repo.find_branch(ramo, git2::BranchType::Local) {
        let _ = b.set_upstream(Some(&format!("{remoto}/{ramo}")));
    }
}
