pub fn install_path() -> std::path::PathBuf {
    let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Maxis\\The Sims").unwrap();
    let sims_install_path: String = key.get_value("InstallPath").unwrap();
    let sims_install_path = std::path::PathBuf::from(sims_install_path);
    assert!(sims_install_path.is_dir());
    sims_install_path
}
