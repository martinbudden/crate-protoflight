use std::{env, fs::File, io::Write, path::PathBuf};

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR environment variable not found"));

    // 1. Write the custom memory mappings into the compiler path
    File::create(out.join("memory.x"))
        .expect("Failed to create memory.x")
        .write_all(include_bytes!("memory.x"))
        .expect("Failed to write memory.x");

    // 2. FORCE GENERATION: Generate a clean defmt template locally.
    // This provides the structural mapping sections defmt needs,
    // ensuring rust-lld never fails with "cannot find linker script defmt.x".
    File::create(out.join("defmt.x"))
        .expect("Failed to create defmt.x")
        .write_all(
            b"SECTIONS {
            .defmt 1 (INFO) : {
                . = 1;
                *(.defmt.prim.*);
                *(.defmt.trace.*);
                *(.defmt.debug.*);
                *(.defmt.info.*);
                *(.defmt.warn.*);
                *(.defmt.error.*);
                *(.defmt.struct.*);
            }
        }
        EXTERN(_defmt_panic);",
        )
        .expect("Failed to write defmt.x");

    // 3. Point Cargo to look in OUT_DIR for both memory.x and defmt.x
    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=build.rs");
}
