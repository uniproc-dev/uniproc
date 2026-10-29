use anyhow::{Result, bail};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let executable = std::env::current_exe()?;
    match args.first().map(String::as_str) {
        Some("run") => service_fixture::run(
            args.get(1)
                .ok_or_else(|| anyhow::anyhow!("missing SCM service name"))?,
        ),
        Some("install") => {
            service_fixture::Fixture::create(&executable, service_fixture::NAME)?.keep();
            println!(
                "Installed {} (Manual, LocalService, stopped)",
                service_fixture::NAME
            );
            Ok(())
        }
        Some("remove") => service_fixture::remove_installed(&executable),
        _ => bail!("Usage: uniproc-service-fixture <install|remove|run NAME>"),
    }
}
