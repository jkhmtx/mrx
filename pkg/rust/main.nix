{nixpkgs, ...}:
nixpkgs.rust-bin.selectLatestNightlyWith (toolchain:
    toolchain.minimal.override {
      targets = [
        "x86_64-unknown-linux-gnu"
        "aarch64-unknown-linux-gnu"
      ];
      extensions = [
        "cargo"
        "clippy"
        "rust-analyzer"
        "rust-src"
        "rustc"
        "rustc-codegen-cranelift-preview"
        "rustfmt"
      ];
    })
