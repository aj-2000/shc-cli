use clap::{arg, Command};

pub fn cli() -> Command {
    Command::new("shc")
        .about("share code in minimum time")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(Command::new("login").about("logging to use shc"))
        .subcommand(
            Command::new("add")
                .about("upload file")
                .arg(arg!(<FILE> "file path to upload"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("list")
                .about("list all files")
                .arg(arg!(<SEARCH> "search key").required(false)),
        )
}
