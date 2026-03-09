use git2::{Cred, Error};

pub fn credentials_callback(
    _url: &str,
    username_from_url: Option<&str>,
    _allowed_types: git2::CredentialType,
) -> Result<Cred, Error> {
    let user = username_from_url.unwrap_or("git");

    // Try SSH agent first
    if let Ok(cred) = Cred::ssh_key_from_agent(user) {
        return Ok(cred);
    }

    // Fallback to default SSH keys if agent fails or is not available
    let home = std::env::var("HOME").map_err(|_| Error::from_str("HOME environment variable not set"))?;
    let ssh_path = std::path::Path::new(&home).join(".ssh");
    
    // List of common private key names to try
    let key_names = ["id_rsa", "id_ed25519", "id_ecdsa", "id_dsa"];
    
    for key_name in &key_names {
        let priv_key = ssh_path.join(key_name);
        let pub_key = ssh_path.join(format!("{}.pub", key_name));
        
        if priv_key.exists() {
            return Cred::ssh_key(
                user,
                Some(&pub_key),
                &priv_key,
                None, // No passphrase for now
            );
        }
    }

    Err(Error::from_str("No SSH keys found and ssh-agent failed"))
}
