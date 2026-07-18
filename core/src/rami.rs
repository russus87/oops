//! Rami: elenco, creazione, checkout, eliminazione e merge.

use git2::build::CheckoutBuilder;
use git2::{BranchType, Repository};

use crate::model::Ramo;

/// Elenca tutti i rami, prima i locali poi i remoti. Segna quello corrente.
pub fn lista(percorso: &str) -> Result<Vec<Ramo>, String> {
    let repo = crate::apri(percorso)?;
    let corrente = crate::repo::nome_ramo(&repo);

    let mut rami = Vec::new();
    aggiungi_rami(&repo, BranchType::Local, false, &corrente, &mut rami)?;
    aggiungi_rami(&repo, BranchType::Remote, true, &corrente, &mut rami)?;
    Ok(rami)
}

/// Aggiunge alla lista i rami di un certo tipo (locale o remoto), con
/// avanti/indietro rispetto all'upstream e i dati dell'ultimo commit.
fn aggiungi_rami(
    repo: &Repository,
    tipo: BranchType,
    remoto: bool,
    corrente: &str,
    fuori: &mut Vec<Ramo>,
) -> Result<(), String> {
    let rami = repo.branches(Some(tipo)).map_err(|e| e.to_string())?;
    for r in rami {
        let (ramo, _) = r.map_err(|e| e.to_string())?;
        let nome = match ramo.name().map_err(|e| e.to_string())? {
            Some(n) => n.to_string(),
            None => continue,
        };

        // Ultimo commit del ramo (titolo + data).
        let (ultimo_titolo, ultimo_quando) = match ramo.get().peel_to_commit() {
            Ok(c) => (
                c.summary().unwrap_or("").to_string(),
                c.time().seconds(),
            ),
            Err(_) => (String::new(), 0),
        };

        // Avanti/indietro solo per i rami locali con un upstream configurato.
        let (avanti, indietro) = if !remoto {
            ramo.upstream()
                .ok()
                .and_then(|u| {
                    let a = ramo.get().target()?;
                    let b = u.get().target()?;
                    repo.graph_ahead_behind(a, b).ok()
                })
                .unwrap_or((0, 0))
        } else {
            (0, 0)
        };

        fuori.push(Ramo {
            corrente: !remoto && nome == corrente,
            remoto,
            avanti,
            indietro,
            ultimo_titolo,
            ultimo_quando,
            nome,
        });
    }
    Ok(())
}

/// Crea un nuovo ramo a partire dal commit corrente (HEAD).
/// Se `cambia` è true, ci si sposta subito sul nuovo ramo.
pub fn crea(percorso: &str, nome: &str, cambia: bool) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let commit = repo
        .head()
        .map_err(|e| e.to_string())?
        .peel_to_commit()
        .map_err(|e| e.to_string())?;
    repo.branch(nome, &commit, false).map_err(|e| e.to_string())?;
    if cambia {
        checkout(percorso, nome)?;
    }
    Ok(())
}

/// Si sposta su un altro ramo (git checkout / git switch).
pub fn checkout(percorso: &str, nome: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;

    let (oggetto, riferimento) = repo
        .revparse_ext(nome)
        .map_err(|e| e.to_string())?;

    // Aggiorna la cartella di lavoro al contenuto del ramo.
    let mut co = CheckoutBuilder::new();
    co.safe();
    repo.checkout_tree(&oggetto, Some(&mut co))
        .map_err(|e| e.to_string())?;

    // Sposta HEAD sul ramo (o sul commit, se è un nome senza riferimento).
    match riferimento {
        Some(rif) => repo
            .set_head(rif.name().ok_or("riferimento senza nome")?)
            .map_err(|e| e.to_string()),
        None => repo.set_head_detached(oggetto.id()).map_err(|e| e.to_string()),
    }
}

/// Elimina un ramo locale.
pub fn elimina(percorso: &str, nome: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let mut ramo = repo
        .find_branch(nome, BranchType::Local)
        .map_err(|e| e.to_string())?;
    ramo.delete().map_err(|e| e.to_string())
}

/// Unisce un altro ramo in quello corrente (git merge).
/// Gestisce: già aggiornato, fast-forward e merge normale. In caso di conflitti
/// restituisce un errore chiaro (i file in conflitto restano da risolvere).
pub fn merge(percorso: &str, nome: &str) -> Result<String, String> {
    let repo = crate::apri(percorso)?;

    // Commit da unire (la punta dell'altro ramo).
    let riferimento = repo
        .find_branch(nome, BranchType::Local)
        .map_err(|e| e.to_string())?
        .into_reference();
    let annotato = repo
        .reference_to_annotated_commit(&riferimento)
        .map_err(|e| e.to_string())?;

    let (analisi, _) = repo.merge_analysis(&[&annotato]).map_err(|e| e.to_string())?;

    if analisi.is_up_to_date() {
        return Ok("già aggiornato".into());
    }

    if analisi.is_fast_forward() {
        // Sposta semplicemente il ramo corrente in avanti.
        let nome_head = repo
            .head()
            .map_err(|e| e.to_string())?
            .name()
            .ok_or("HEAD senza nome")?
            .to_string();
        let mut rif = repo.find_reference(&nome_head).map_err(|e| e.to_string())?;
        rif.set_target(annotato.id(), "fast-forward")
            .map_err(|e| e.to_string())?;
        repo.set_head(&nome_head).map_err(|e| e.to_string())?;
        repo.checkout_head(Some(CheckoutBuilder::new().force()))
            .map_err(|e| e.to_string())?;
        return Ok("fast-forward".into());
    }

    // Merge vero e proprio: applica le modifiche nell'indice/cartella.
    repo.merge(&[&annotato], None, None).map_err(|e| e.to_string())?;

    let index = repo.index().map_err(|e| e.to_string())?;
    if index.has_conflicts() {
        return Err("merge con conflitti: risolvi i file e fai un commit".into());
    }

    // Niente conflitti: creiamo il commit di merge con due genitori.
    let firma = repo
        .signature()
        .or_else(|_| git2::Signature::now("Oops", "oops@local"))
        .map_err(|e| e.to_string())?;
    let albero_id = repo
        .index()
        .map_err(|e| e.to_string())?
        .write_tree()
        .map_err(|e| e.to_string())?;
    let albero = repo.find_tree(albero_id).map_err(|e| e.to_string())?;
    let nostro = repo.head().map_err(|e| e.to_string())?.peel_to_commit().map_err(|e| e.to_string())?;
    let loro = repo.find_commit(annotato.id()).map_err(|e| e.to_string())?;

    repo.commit(
        Some("HEAD"),
        &firma,
        &firma,
        &format!("Merge del ramo '{nome}'"),
        &albero,
        &[&nostro, &loro],
    )
    .map_err(|e| e.to_string())?;

    repo.cleanup_state().map_err(|e| e.to_string())?;
    Ok("merge completato".into())
}

/// Unisce il ramo `sorgente` DENTRO il ramo `destinazione` (drag&drop di un ramo
/// sopra un altro): passa a `destinazione` e poi fa il merge di `sorgente`.
pub fn merge_rami(percorso: &str, sorgente: &str, destinazione: &str) -> Result<String, String> {
    checkout(percorso, destinazione)?;
    merge(percorso, sorgente)
}

/// Riposiziona (rebase) il ramo `sorgente` sopra `destinazione`: passa a
/// `sorgente` e poi fa il rebase su `destinazione`.
pub fn rebase_rami(percorso: &str, sorgente: &str, destinazione: &str) -> Result<String, String> {
    checkout(percorso, sorgente)?;
    rebase(percorso, destinazione)
}

/// Crea un nuovo ramo a partire da un commit specifico.
pub fn crea_da(percorso: &str, nome: &str, id: &str, cambia: bool) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let oid = git2::Oid::from_str(id).map_err(|e| e.to_string())?;
    let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;
    repo.branch(nome, &commit, false).map_err(|e| e.to_string())?;
    if cambia {
        checkout(percorso, nome)?;
    }
    Ok(())
}

/// Si sposta su un commit specifico (HEAD "staccata").
pub fn checkout_commit(percorso: &str, id: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let oid = git2::Oid::from_str(id).map_err(|e| e.to_string())?;
    let oggetto = repo
        .find_commit(oid)
        .map_err(|e| e.to_string())?
        .into_object();
    repo.checkout_tree(&oggetto, Some(CheckoutBuilder::new().safe()))
        .map_err(|e| e.to_string())?;
    repo.set_head_detached(oid).map_err(|e| e.to_string())
}

/// Riposiziona (rebase) il ramo corrente sopra un altro ramo. In caso di
/// conflitti annulla tutto e restituisce un errore (rebase non interattivo).
pub fn rebase(percorso: &str, su: &str) -> Result<String, String> {
    let repo = crate::apri(percorso)?;

    let bersaglio = repo
        .find_branch(su, BranchType::Local)
        .map_err(|e| e.to_string())?;
    let su_ac = repo
        .reference_to_annotated_commit(bersaglio.get())
        .map_err(|e| e.to_string())?;

    let firma = repo
        .signature()
        .or_else(|_| git2::Signature::now("Oops", "oops@local"))
        .map_err(|e| e.to_string())?;

    // branch = None -> usa HEAD; upstream = ramo bersaglio; onto = None -> bersaglio.
    let mut rb = repo
        .rebase(None, Some(&su_ac), None, None)
        .map_err(|e| e.to_string())?;

    let mut applicati = 0;
    loop {
        match rb.next() {
            None => break,
            Some(Ok(_op)) => {}
            Some(Err(e)) => {
                let _ = rb.abort();
                return Err(e.to_string());
            }
        }
        // Se l'applicazione del commit ha generato conflitti, abortiamo.
        if repo
            .index()
            .map_err(|e| e.to_string())?
            .has_conflicts()
        {
            let _ = rb.abort();
            return Err("rebase con conflitti: annullato".into());
        }
        rb.commit(None, &firma, None).map_err(|e| e.to_string())?;
        applicati += 1;
    }

    rb.finish(Some(&firma)).map_err(|e| e.to_string())?;
    Ok(format!("rebase completato ({applicati} commit)"))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    /// Nome del ramo corrente (utile nei test: init può creare master o main).
    fn ramo_corrente(p: &str) -> String {
        let repo = git2::Repository::open(p).unwrap();
        let head = repo.head().unwrap();
        let nome = head.shorthand().unwrap().to_string();
        nome
    }

    #[test]
    fn crea_e_elenca() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_str().unwrap();
        Repository::init(dir.path()).unwrap();
        fs::write(dir.path().join("a.txt"), "x").unwrap();
        crate::stage::aggiungi(p, "a.txt").unwrap();
        crate::commit::crea(p, "init", "T", "t@t.it").unwrap();

        crea(p, "sviluppo", false).unwrap();
        let rami = lista(p).unwrap();
        let nomi: Vec<&str> = rami.iter().map(|r| r.nome.as_str()).collect();
        assert!(nomi.contains(&"sviluppo"));
        // Uno solo dev'essere quello corrente.
        assert_eq!(rami.iter().filter(|r| r.corrente).count(), 1);
    }

    #[test]
    fn rebase_lineare() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_str().unwrap();
        git2::Repository::init(dir.path()).unwrap();

        // Commit base su main.
        fs::write(dir.path().join("base.txt"), "base\n").unwrap();
        crate::stage::aggiungi(p, "base.txt").unwrap();
        crate::commit::crea(p, "base", "T", "t@t.it").unwrap();

        // Nome del ramo principale (dipende dalla config: master o main).
        let principale = ramo_corrente(p);

        // Ramo funzione con un suo commit (file diverso = niente conflitti).
        crea(p, "funzione", true).unwrap();
        fs::write(dir.path().join("funz.txt"), "funz\n").unwrap();
        crate::stage::aggiungi(p, "funz.txt").unwrap();
        crate::commit::crea(p, "lavoro", "T", "t@t.it").unwrap();

        // Avanza il ramo principale con un altro commit.
        checkout(p, &principale).unwrap();
        fs::write(dir.path().join("main2.txt"), "m2\n").unwrap();
        crate::stage::aggiungi(p, "main2.txt").unwrap();
        crate::commit::crea(p, "main avanti", "T", "t@t.it").unwrap();

        // Rebase di funzione sul principale: "lavoro" viene riapplicato in cima.
        checkout(p, "funzione").unwrap();
        rebase(p, &principale).unwrap();

        let storia: Vec<String> = crate::commit::log(p, 10)
            .unwrap()
            .into_iter()
            .map(|v| v.titolo)
            .collect();
        // "lavoro" è in cima e ora la storia include "main avanti" (prima assente):
        // il rebase ha riportato il commit del ramo sopra al principale aggiornato.
        assert_eq!(storia.len(), 3);
        assert_eq!(storia[0], "lavoro");
        assert!(storia.contains(&"main avanti".to_string()));
    }
}
