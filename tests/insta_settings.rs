use insta::Settings;

/// Configuración para insta snapshots
pub fn setup_insta() {
    let mut settings = Settings::clone_current();
    settings.set_snapshot_path("tests/snapshots");
    settings.bind(|| {});
}
