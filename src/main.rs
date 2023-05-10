// Copyright (c) 2023 Jacob Allen Morris
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

mod modules;
mod arguments;

fn main() {
    let args = arguments::handle_args();
    if args.is_err() {
        panic!("Uhoh! Invalid arguments! Issue: {:?}", args.err().unwrap());
    } else {
        let args = args.unwrap();
        if (&args).command.as_ref().is_some() {
            match (&args).command.as_ref().unwrap().as_str() {
                "apply" => {
                    modules::apply(args);
                }
                _ => {
                    panic!("Uhoh! Invalid command!");
                }
            }
        } else {
            panic!("Uhoh! No command was specified!");
        }
    }
}
