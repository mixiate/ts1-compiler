pub fn install_path() -> anyhow::Result<std::path::PathBuf> {
    use anyhow::Context;

    let error_message = "Failed to find The Sims installation";

    let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Maxis\\The Sims").context(error_message)?;
    let sims_install_path: String = key.get_value("InstallPath").context(error_message)?;
    let sims_install_path = std::path::PathBuf::from(sims_install_path);
    anyhow::ensure!(
        sims_install_path.is_dir(),
        format!(
            "Failed to find The Sims installation directory {}",
            sims_install_path.display()
        )
    );
    Ok(sims_install_path)
}
