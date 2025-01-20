// build.rs

use std::env;
use std::fs;
use std::path::Path;

fn generate_transitions(
    symbol: &str,
    states: &[&str],
    next_state: &str,
) -> String {
    let transitions = states
        .iter()
        .map(|state| {
            format!(
                "
        ((St::{symbol}, St::{state}), (St::{next_state}, St::{next_state})),"
            )
        })
        .collect::<Vec<String>>()
        .join("");

    transitions
}

macro_rules! add {
    ($r:ident, $symbol:literal, $states:expr, $next_state:literal) => {
        $r.push_str(&generate_transitions(
            $symbol,
            $states,
            $next_state,
        ));
    };
}

macro_rules! transitions_table {
    ($r:ident) => {
        let all = &[
            "ST", "HE", "GC", "OC", "FL", "CT", "PC", "PM", "PP",
            "TC", "MS", "MP", "MX", "MI",
        ];

        add!($r, "TC", &["ST", "HE"], "HE");
        add!(
            $r,
            "TC",
            &[
                "GC", "OC", "FL", "TC", "PC", "PM", "PP", "MS", "MP",
                "MX", "MI"
            ],
            "TC"
        );
        add!($r, "GC", all, "GC");
        add!($r, "OC", all, "OC");
        add!($r, "FL", all, "FL");
        add!($r, "PC", all, "PC");
        add!($r, "PM", all, "PM");
        add!($r, "PP", all, "PP");
        add!(
            $r,
            "CT",
            &[
                "ST", "HE", "GC", "OC", "FL", "TC", "PC", "PM", "PP",
                "MS", "MX"
            ],
            "CT"
        );
        add!(
            $r,
            "MI",
            &[
                "ST", "HE", "GC", "OC", "FL", "CT", "TC", "PC", "PM",
                "PP", "MS", "MX"
            ],
            "MI"
        );
        add!($r, "MP", &["TC", "GC", "PC", "PM", "PP", "MI"], "MP");
        add!($r, "MS", &["MI", "MP", "TC"], "MS");
        add!($r, "MX", &["MI", "MX", "MP", "TC"], "MX");
        add!(
            $r,
            "MC",
            &["CT", "MI", "MP", "MS", "MX", "PM", "PP", "PC"],
            "MC"
        );
    };
}

fn generate_build_transitions_function() -> String {
    let mut r = String::from(
        "fn build_transitions() -> Transitions {
    HashMap::from([",
    );
    transitions_table!(r);
    r.push_str(
        "
    ])
}",
    );
    r
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path =
        Path::new(&out_dir).join("poparser-transitions.rs");

    fs::write(dest_path, generate_build_transitions_function())
        .unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
