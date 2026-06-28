//! Morn Bootstrap — the only non-replaceable layer (§1 of DESIGN.md).
//! MVP mode: kernel bootstrap only.
//! Full mode: delegates to full_main.rs for the complete Morn experience.

#[cfg(feature = "full")]
mod full_main;

fn main() {
    #[cfg(feature = "full")]
    {
        let _ = full_main::full_main();
    }

    #[cfg(not(feature = "full"))]
    {
        println!("Morn MVP — Kernel bootstrap OK");
        println!("Run with --features full for complete Morn.");
    }
}
