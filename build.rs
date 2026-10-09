use std::{env, fs::File, io::Write, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var_os("OUT_DIR").ok_or("OUT_DIR environment variable is not set")?;
    let out = PathBuf::from(out_dir);

    // 1. Write the custom memory mappings into the compiler path
    File::create(out.join("memory.x"))?.write_all(include_bytes!("memory.x"))?;

    // 2. FORCE GENERATION: Generate a clean defmt template locally.
    // This provides the structural mapping sections defmt needs,
    // ensuring rust-lld never fails with "cannot find linker script defmt.x".
    File::create(out.join("defmt.x"))?
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

    Ok(())
}
