//! Runner del "terminale git" integrato: esegue un comando `git` nella cartella
//! del repository e restituisce l'output combinato (stdout + stderr). La UI poi
//! ricarica lo stato, così ogni comando aggiorna l'interfaccia. Per sicurezza si
//! esegue sempre e solo il binario `git` (mai una shell arbitraria).

use std::process::Command;

/// Esegue `git <args>` dentro `percorso` e restituisce l'output testuale.
pub fn git(percorso: &str, args: Vec<String>) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(percorso)
        .args(&args)
        .output()
        .map_err(|e| format!("impossibile eseguire git: {e}"))?;

    let mut testo = String::new();
    testo.push_str(&String::from_utf8_lossy(&output.stdout));
    let err = String::from_utf8_lossy(&output.stderr);
    if !err.trim().is_empty() {
        if !testo.is_empty() && !testo.ends_with('\n') {
            testo.push('\n');
        }
        testo.push_str(&err);
    }
    if testo.trim().is_empty() {
        testo = if output.status.success() {
            "(nessun output)".into()
        } else {
            "il comando è fallito senza output".into()
        };
    }
    Ok(testo)
}
