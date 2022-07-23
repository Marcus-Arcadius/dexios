use anyhow::Result;
use global::parameters::get_param;
use global::parameters::key_manipulation_params;
use global::parameters::skipmode;
use subcommands::list::show_values;

mod cli;
mod domain;
mod file;
mod global;
mod subcommands;
pub(crate) mod utils;

// this is where subcommand function calling is handled
// it goes hand-in-hand with `subcommands.rs`
// it works so that's good enough, and any changes are rather simple to make to it
// it handles the calling of other functions, and some (minimal) argument parsing
fn main() -> Result<()> {
    let matches = cli::get_matches();

    match matches.subcommand() {
        Some(("encrypt", sub_matches)) => {
            subcommands::encrypt(sub_matches)?;
        }
        Some(("decrypt", sub_matches)) => {
            subcommands::decrypt(sub_matches)?;
        }
        Some(("erase", sub_matches)) => {
            subcommands::erase(sub_matches)?;
        }
        Some(("pack", sub_matches)) => {
            subcommands::pack(sub_matches)?;
        }
        Some(("unpack", sub_matches)) => {
            subcommands::unpack(sub_matches)?;
        }
        Some(("hash", sub_matches)) => {
            let files: Vec<String> = if sub_matches.is_present("input") {
                let list: Vec<&str> = sub_matches.values_of("input").unwrap().collect();
                list.iter().map(std::string::ToString::to_string).collect()
            } else {
                Vec::new()
            };

            subcommands::hashing::hash_stream(&files)?;
        }
        Some(("list", sub_matches)) => {
            show_values(&get_param("input", sub_matches)?)?;
        }
        Some(("header", sub_matches)) => match sub_matches.subcommand_name() {
            Some("dump") => {
                let sub_matches_dump = sub_matches.subcommand_matches("dump").unwrap();
                let skip = skipmode(sub_matches_dump);

                subcommands::header::dump(
                    &get_param("input", sub_matches_dump)?,
                    &get_param("output", sub_matches_dump)?,
                    skip,
                )?;
            }
            Some("restore") => {
                let sub_matches_restore = sub_matches.subcommand_matches("restore").unwrap();
                let skip = skipmode(sub_matches_restore);

                subcommands::header::restore(
                    &get_param("input", sub_matches_restore)?,
                    &get_param("output", sub_matches_restore)?,
                    skip,
                )?;
            }
            Some("strip") => {
                let sub_matches_strip = sub_matches.subcommand_matches("strip").unwrap();
                let skip = skipmode(sub_matches_strip);

                subcommands::header::strip(&get_param("input", sub_matches_strip)?, skip)?;
            }
            Some("details") => {
                let sub_matches_details = sub_matches.subcommand_matches("details").unwrap();

                subcommands::header::details(&get_param("input", sub_matches_details)?)?;
            }
            _ => (),
        },
        Some(("key", sub_matches)) => match sub_matches.subcommand_name() {
            Some("change") => {
                let sub_matches_change_key = sub_matches.subcommand_matches("change").unwrap();

                let (key_old, key_new) = key_manipulation_params(sub_matches_change_key)?;

                subcommands::header_key::change_key(
                    &get_param("input", sub_matches_change_key)?,
                    &key_old,
                    &key_new,
                )?;
            }
            Some("add") => {
                let sub_matches_add_key = sub_matches.subcommand_matches("add").unwrap();

                let (key_old, key_new) = key_manipulation_params(sub_matches_add_key)?;

                subcommands::header_key::add_key(
                    &get_param("input", sub_matches_add_key)?,
                    &key_old,
                    &key_new,
                )?;
            }
            Some("del") => {
                // TODO(brxken128): unify `Key` creation with one function
                use crate::global::states::Key;
                use anyhow::Context;

                let sub_matches_del_key = sub_matches.subcommand_matches("del").unwrap();

                let key = if sub_matches_del_key.is_present("keyfile") {
                    Key::Keyfile(
                        sub_matches_del_key
                            .value_of("keyfile")
                            .context("No keyfile/invalid text provided")?
                            .to_string(),
                    )
                } else if std::env::var("DEXIOS_KEY").is_ok() {
                    Key::Env
                } else if let Ok(true) = sub_matches_del_key.try_contains_id("autogenerate") {
                    Key::Generate
                } else {
                    Key::User
                };

                subcommands::header_key::del_key(&get_param("input", sub_matches_del_key)?, &key)?;
            }
            _ => (),
        },
        _ => (),
    }
    Ok(())
}
