use cargo_px_env::generated_pkg_manifest_path;
use pavex_cli_client::Client;
use std::error::Error;

/// Generate the `sdk` crate using Pavex's CLI.
fn main() -> Result<(), Box<dyn Error>> {
	let generated_dir = generated_pkg_manifest_path()?
		.parent()
		.unwrap()
		.to_path_buf();

	Client::new()
		.generate(app::blueprint(), generated_dir)
		.execute()?;
	Ok(())
}
