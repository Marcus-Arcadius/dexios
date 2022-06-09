use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    process::exit,
};

use std::io::Cursor;

use anyhow::{Context, Result};
use paris::Logger;

use dexios_core::header::Header;

use crate::global::states::SkipMode;

use super::prompt::{get_answer, overwrite_check};

// this function dumps the first 64 bytes of
// the input file into the output file
// it's used for extracting an encrypted file's header for backups and such
// it implements a check to ensure the header is valid
pub fn dump(input: &str, output: &str, skip: SkipMode) -> Result<()> {
    let mut logger = Logger::new();
    logger.warn("THIS FEATURE IS FOR ADVANCED USERS ONLY AND MAY RESULT IN A LOSS OF DATA - PROCEED WITH CAUTION");

    let mut header = [0u8; 64];

    let mut input_file =
        File::open(input).with_context(|| format!("Unable to open input file: {}", input))?;
    input_file
        .read_exact(&mut header)
        .with_context(|| format!("Unable to read header from file: {}", input))?;

    let mut header_reader = Cursor::new(header);
    if Header::deserialize(&mut header_reader).is_err() {
        logger.error("File does not contain a valid Dexios header - exiting");
        drop(input_file);
        exit(1);
    }

    if !overwrite_check(output, skip)? {
        std::process::exit(0);
    }

    let mut output_file =
        File::create(output).with_context(|| format!("Unable to open output file: {}", output))?;
    output_file
        .write_all(&header)
        .with_context(|| format!("Unable to write header to output file: {}", output))?;

    logger.success(format!("Header dumped to {} successfully.", output));
    Ok(())
}

// this function reads the first 64 bytes (header) from the input file
// and then overwrites the first 64 bytes of the output file with it
// this can be used for restoring a dumped header to a file that had it's header stripped
// it implements a check to ensure the header is valid before restoring to a file
pub fn restore(input: &str, output: &str, skip: SkipMode) -> Result<()> {
    let mut logger = Logger::new();
    logger.warn("THIS FEATURE IS FOR ADVANCED USERS ONLY AND MAY RESULT IN A LOSS OF DATA - PROCEED WITH CAUTION");
    let prompt = format!(
        "Are you sure you'd like to restore the header in {} to {}?",
        input, output
    );
    if !get_answer(&prompt, false, skip == SkipMode::HidePrompts)? {
        exit(0);
    }

    let mut header = vec![0u8; 64];

    let mut input_file =
        File::open(input).with_context(|| format!("Unable to open header file: {}", input))?;
    input_file
        .read_exact(&mut header)
        .with_context(|| format!("Unable to read header from file: {}", input))?;

    let mut header_reader = Cursor::new(header.clone());
    if Header::deserialize(&mut header_reader).is_err() {
        logger.error("File does not contain a valid Dexios header - exiting");
        drop(input_file);
        exit(1);
    }

    let mut output_file = OpenOptions::new()
        .write(true)
        .open(output)
        .with_context(|| format!("Unable to open output file: {}", output))?;

    output_file
        .write_all(&header)
        .with_context(|| format!("Unable to write header to file: {}", output))?;

    logger.success(format!(
        "Header restored to {} from {} successfully.",
        output, input
    ));
    Ok(())
}

// this wipes the first 64 bytes (header) from the provided file
// it can be useful for storing the header separate from the file, to make an attacker's life that little bit harder
// it implements a check to ensure the header is valid before stripping
pub fn strip(input: &str, skip: SkipMode) -> Result<()> {
    let mut logger = Logger::new();
    logger.warn("THIS FEATURE IS FOR ADVANCED USERS ONLY AND MAY RESULT IN A LOSS OF DATA - PROCEED WITH CAUTION");

    let prompt = format!("Are you sure you'd like to wipe the header for {}?", input);
    if !get_answer(&prompt, false, skip == SkipMode::HidePrompts)? {
        exit(0);
    }

    let prompt = "This can be destructive! Make sure you dumped the header first. Would you like to continue?";
    if !get_answer(prompt, false, skip == SkipMode::HidePrompts)? {
        exit(0);
    }

    let buffer = vec![0u8; 64];

    let mut input_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(input)
        .with_context(|| format!("Unable to open input file: {}", input))?;

    if Header::deserialize(&mut input_file).is_err() {
        logger.error("File does not contain a valid Dexios header - exiting");
        drop(input_file);
        exit(1);
    } else {
        input_file
            .seek(std::io::SeekFrom::Current(-64))
            .context("Unable to seek back to the start of the file")?;
    }

    input_file
        .write_all(&buffer)
        .with_context(|| format!("Unable to wipe header for file: {}", input))?;

    logger.success(format!("Header stripped from {} successfully.", input));
    Ok(())
}