// Copyright (c) 2023 Jacob Allen Morris
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use serde::Deserialize;
use std::{
    env, fs,
    io::{self, BufReader, BufWriter},
    path::PathBuf,
};

#[derive(Deserialize, Debug)]
struct TomlInputs {
    packages: toml::Table,
}

pub fn run(args: crate::arguments::Arguments) {

    let mut esodiff_path = env::current_dir().unwrap();
    if args.diff_file.is_none() {
        esodiff_path.push("esodiff.toml");
    } else {
        esodiff_path.push(args.diff_file.unwrap());
    }
    let esodiff = fs::read(esodiff_path).unwrap();
    let esodiff = String::from_utf8(esodiff).unwrap();
    let esodiff: TomlInputs = toml::from_str(esodiff.as_str()).unwrap();
    for (crate_name, values) in esodiff.packages {
        let version = values.get("version").unwrap().as_str().unwrap();
        let patch = values.get("patch").unwrap().as_str().unwrap();
        let output = values.get("output").unwrap().as_str().unwrap();
        patch_crate_using_file(crate_name, version.into(), patch.into(), output.into());
    }
}

fn create_dirs(path: PathBuf) {
    let mut path = path.clone();
    let mut path_list: Vec<PathBuf> = Vec::new();
    loop {
        if fs::read_dir(&path).is_ok() {
            break;
        } else {
            path_list.push((&path).to_owned());
            path.pop();
        }
    }
    for i in 0..path_list.len() {
        fs::create_dir(&path_list[path_list.len() - 1 - i]).unwrap();
    }
}

fn patch_crate_using_file(
    crate_name: String,
    version: String,
    input_diff_file: String,
    output_dir: String,
) {
    let diff_file = {
        let new_path = PathBuf::from(input_diff_file);
        let diff_contents = fs::read(new_path).unwrap();
        git2::Diff::from_buffer(&diff_contents[..]).unwrap()
    };
    let root_path = PathBuf::from(output_dir);
    let test_path = {
        let mut new_path = root_path.clone();
        new_path.push(format!("{}-{}", crate_name, version));
        new_path
    };
    if fs::read_dir(test_path).is_err() {
        let raw_file = reqwest::blocking::get(format!(
            "https://crates.io/api/v1/crates/{}/{}/download",
            crate_name, version
        ))
        .unwrap();
        let raw_file = raw_file.bytes().unwrap().to_vec();
        let mut raw_file = BufReader::new(&raw_file[..]);

        let mut raw_tar: Vec<u8> = Vec::new();
        let mut raw_tar_writer = BufWriter::new(&mut raw_tar);

        let mut decoder = libflate::gzip::Decoder::new(&mut raw_file).unwrap();
        io::copy(&mut decoder, &mut raw_tar_writer).unwrap();
        drop(raw_tar_writer);
        let raw_tar_reader = BufReader::new(&raw_tar[..]);

        let mut repository = root_path.clone();
        repository.push(format!("{}-{}", crate_name, version));
        create_dirs((&repository).to_owned());
        let repository = git2::Repository::init(repository.to_str().unwrap()).unwrap();

        let mut archive = tar::Archive::new(raw_tar_reader);
        for item in archive.entries().unwrap() {
            let mut item = item.unwrap();
            let header = item.header();
            let mut item_path = root_path.clone();
            item_path.push(header.path().unwrap());
            let item_name = String::from((&item_path).file_name().unwrap().to_str().unwrap());
            item_path.pop();
            create_dirs((&item_path).to_owned());
            item_path.push(item_name);
            item.unpack(item_path).unwrap();
        }
        repository
            .apply(&diff_file, git2::ApplyLocation::WorkDir, None)
            .unwrap();
    }
}
