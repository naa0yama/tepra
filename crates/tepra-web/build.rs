//! Build script: compiles CSS via Tailwind 4 + `DaisyUI` 5, copies fonts and HTMX into `OUT_DIR`.

use std::path::Path;
use std::process::Command;

use anyhow::{Context as _, anyhow};

fn main() -> anyhow::Result<()> {
    let out_dir = std::env::var("OUT_DIR").context("OUT_DIR not set")?;

    // 1. pnpm presence check — fall back to stubs in CI environments without Node.js
    let pnpm_available = Command::new("pnpm")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());

    if !pnpm_available {
        println!(
            "cargo:warning=pnpm not found; writing stub assets. \
             Real build requires `mise exec -- cargo build`."
        );
        emit_git_hash();
        write_stubs(&out_dir)?;
        return Ok(());
    }

    // 2. Install frontend deps from lockfile
    let install = Command::new("pnpm")
        .args(["install", "--frozen-lockfile"])
        .status()
        .context("failed to run pnpm install")?;
    if !install.success() {
        return Err(anyhow!(
            "pnpm install --frozen-lockfile failed. \
             Run `pnpm install` locally, commit the updated lockfile, retry."
        ));
    }

    // 3. Compile CSS
    let out_css = Path::new(&out_dir).join("app.css");
    let build = Command::new("pnpm")
        .args(["exec", "tailwindcss", "-i", "src/styles/app.css", "-o"])
        .arg(&out_css)
        .arg("--minify")
        .status()
        .context("failed to run tailwindcss")?;
    if !build.success() {
        return Err(anyhow!(
            "tailwindcss compilation failed. Check src/styles/app.css."
        ));
    }

    // 4. Copy @fontsource woff2 files
    let fonts_dir = Path::new(&out_dir).join("fonts");
    for family in ["ibm-plex-sans-jp", "ibm-plex-mono"] {
        let font_src = Path::new("node_modules/@fontsource")
            .join(family)
            .join("files");
        let font_out = fonts_dir.join(family);
        std::fs::create_dir_all(&font_out)
            .with_context(|| format!("failed to create font dir: {}", font_out.display()))?;
        for entry in walkdir::WalkDir::new(&font_src)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().is_some_and(|x| x == "woff2"))
        {
            let dest = font_out.join(entry.file_name());
            std::fs::copy(entry.path(), &dest).with_context(|| {
                format!(
                    "failed to copy font: {} -> {}",
                    entry.path().display(),
                    dest.display()
                )
            })?;
        }
        println!("cargo:rerun-if-changed={}", font_src.display());
    }

    // 5. Copy HTMX
    let htmx_src = Path::new("node_modules/htmx.org/dist/htmx.min.js");
    let htmx_dst = Path::new(&out_dir).join("htmx.min.js");
    std::fs::copy(htmx_src, &htmx_dst).with_context(|| {
        format!(
            "failed to copy htmx.min.js: {}\nEnsure htmx.org is in package.json devDependencies.",
            htmx_src.display()
        )
    })?;
    println!("cargo:rerun-if-changed={}", htmx_src.display());

    // 6. rerun-if-changed
    // GIT_HASH for OTel resource attribute vcs.repository.ref.revision
    emit_git_hash();

    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=pnpm-lock.yaml");
    println!("cargo:rerun-if-changed=src/styles");

    for entry in walkdir::WalkDir::new("../tepra/templates")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        println!("cargo:rerun-if-changed={}", entry.path().display());
    }

    Ok(())
}

fn emit_git_hash() {
    let hash = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_owned());
    println!("cargo:rustc-env=GIT_HASH={hash}");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");
}

fn write_stubs(out_dir: &str) -> anyhow::Result<()> {
    let out_css = Path::new(out_dir).join("app.css");
    std::fs::write(&out_css, b"").context("failed to write stub app.css")?;

    let htmx_dst = Path::new(out_dir).join("htmx.min.js");
    std::fs::write(&htmx_dst, b"").context("failed to write stub htmx.min.js")?;

    let fonts_dir = Path::new(out_dir).join("fonts");
    for family in ["ibm-plex-sans-jp", "ibm-plex-mono"] {
        std::fs::create_dir_all(fonts_dir.join(family))
            .with_context(|| format!("failed to create stub font dir: {family}"))?;
    }

    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=pnpm-lock.yaml");
    println!("cargo:rerun-if-changed=src/styles");

    Ok(())
}
