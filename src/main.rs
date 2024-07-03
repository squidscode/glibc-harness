use clap::{arg, command};
use regex::Regex;
use std::collections::HashSet;
use std::process::Command;

fn main() {
    let matches = command!("libc-harness")
        .about(
            "Creates a shared library with c harnesses\
            for all libc functions called by the given binary.",
        )
        .version("1.0.0")
        .arg(arg!(<BINARY>).help("the target binary for the harness"))
        .get_matches();

    let objdump_output = Command::new("objdump")
        .args(["-d", "--section=.plt"])
        .arg(
            matches
                .get_one::<String>("BINARY")
                .expect("binary is required"),
        )
        .output()
        .expect("objdump errored");

    let objdump_output = String::from_utf8(objdump_output.stdout).unwrap();
    let plt_functions_re = Regex::new(r"<(\w+)@plt>").unwrap();
    let plt_functions = plt_functions_re
        .captures_iter(&objdump_output)
        .map(|m| m.extract::<1>().1[0])
        .collect::<Vec<&str>>();

    let json_str = include_str!("../resources/libfunctions_formatted.json");
    let libfunctions: serde_json::Value = serde_json::from_str(json_str).unwrap();

    // println!("{:?}", plt_functions);
    // println!("{:?}", libfunctions["functions"]);

    let used_functions = plt_functions
        .iter()
        .filter_map(|func| {
            if libfunctions["functions"]
                .as_object()
                .unwrap()
                .contains_key(*func)
            {
                Some(String::from(*func))
            } else {
                None
            }
        })
        .collect::<Vec<String>>();
    let c_functions = used_functions
        .iter()
        .map(|func| {
            format!(
                "{} {}({}) {{\n    // TODO: fill this in\n}}",
                libfunctions["functions"][func]["return type"]
                    .as_str()
                    .unwrap(),
                func,
                libfunctions["functions"][func]["arguments"]
                    .as_array()
                    .unwrap()
                    .into_iter()
                    .map(|v| v.as_str().unwrap())
                    .collect::<Vec<&str>>()
                    .join(", ")
            )
        })
        .collect::<Vec<String>>()
        .join("\n\n");
    let c_includes = used_functions
        .iter()
        .map(|func| {
            Regex::new(r"^[\w./]+")
                .unwrap()
                .find_iter(
                    libfunctions["functions"][func]["description"]
                        .as_str()
                        .unwrap()
                        .split(" ")
                        .next()
                        .unwrap(),
                )
                .next()
                .unwrap()
                .as_str()
        })
        .collect::<HashSet<&str>>()
        .iter()
        .map(|header_file| format!("#include <{}>", header_file))
        .collect::<Vec<String>>()
        .join("\n");

    println!("{}\n\n{}", c_includes, c_functions);
}
