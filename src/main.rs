use std::path::PathBuf;

mod api_client;
mod app_config;
mod cli;
mod command;
mod consts;
mod models;
mod tui;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shc_folder = dirs::home_dir().unwrap().join(".shc-cli");
    if !shc_folder.exists() {
        std::fs::create_dir_all(&shc_folder)?;
    }
    let config_path = dirs::home_dir().unwrap().join(".shc-cli/config.toml");
    let mut config = app_config::AppConfig::new(&config_path);

    let mut api_client = crate::api_client::ApiClient::new();

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
                    command::add::upload_file(&file_path, &mut api_client).await?;
                }
                Some(("rename", sub_matches)) => {
                    let default: String = "".to_string();
                    //TODO: todo rename search to filter
                    let search = sub_matches.get_one::<String>("FILTER").unwrap_or(&default);
                    command::rename::rename_file(search, &mut api_client).await?;
                }
                Some(("get", sub_matches)) => {
                    let default: String = "".to_string();
                    //TODO: todo rename search to filter
                    let search = sub_matches.get_one::<String>("FILTER").unwrap_or(&default);
                    command::get::download_file(search, &mut api_client).await?;
                }

                Some(("remove", sub_matches)) => {
                    let default: String = "".to_string();
                    //TODO: todo rename search to filter
                    let search = sub_matches.get_one::<String>("FILTER").unwrap_or(&default);
                    command::remove::remove_file(search, &mut api_client).await?;
                }
                Some(("visibility", sub_matches)) => {
                    let default: String = "".to_string();
                    //TODO: todo rename search to filter
                    let search = sub_matches.get_one::<String>("FILTER").unwrap_or(&default);
                    command::visibility::toggle_file_visibility(search, &mut api_client).await?;
                }
                Some(("list", sub_matches)) => {
                    let default: String = "".to_string();
                    //TODO: todo rename search to filter
                    let search = sub_matches.get_one::<String>("FILTER").unwrap_or(&default);
                    command::list::list_files(search, &mut api_client).await?;
                }

                _ => println!("Command not found."),
            };
        }
    };
    Ok(())
}
