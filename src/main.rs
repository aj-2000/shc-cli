use std::path::PathBuf;
mod app_config;
mod cli;
mod command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = dirs::home_dir()
        .unwrap()
        .join("Documents/DEV/shc-cli/config.toml");
    let mut config = app_config::AppConfig::new(&config_path);

    let matches = cli::cli().get_matches();

    match matches.subcommand() {
        Some(("login", _)) => command::login::login(&mut config, &config_path).await?,
        None => println!("No subcommand was used"),

        _ => {
            command::login::check_for_api_key(&mut config, &config_path).await?;
            match matches.subcommand() {
                Some(("add", sub_matches)) => {
                    let file = sub_matches.get_one::<String>("FILE").expect("required");
                    let file_path = PathBuf::from(file);
                    if !file_path.exists() {
                        println!("File not found");
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            "File not found",
                        )
                        .into());
                    }
                    let file_name = file_path.file_name().unwrap().to_str().unwrap();
                    println!("Sharing file: {} ", file_name);
                    command::add::upload_file(
                        &file_path,
                        &config.user_id.as_ref().unwrap(),
                        &config.password.as_ref().unwrap(),
                    )
                    .await?;
                }
                Some(("list", sub_matches)) => {
                    let default: String = "".to_string();
                    let search = sub_matches.get_one::<String>("SEARCH").unwrap_or(&default);
                    command::list::list_files(
                        &search,
                        &config.user_id.as_ref().unwrap(),
                        &config.password.as_ref().unwrap(),
                    )
                    .await?;
                }
                _ => println!("Command not found."),
            };
        }
    };
    Ok(())
}
