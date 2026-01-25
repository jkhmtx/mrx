A new repo needs:

`nix run 'github:jkhmtx/mrx' -- install`

Which:

- Creates an empty/initial `mrx.generated.nix`
- Creates an empty/initial `mrx.toml`
- Creates an empty/initial `shell/main.nix`
- Adds a `envrc.mrx.sh`, containing:

```bash
strict_env

function mrx() {
  .direnv/mrx/bin/mrx "${@}"
}

nix build --out-link .direnv/mrx --print-build-logs

# Build and add paths discovered by 'mrx build'
while read -r file; do
  PATH_add "${file}"
done < <(
  mrx build \
    --generate \
    --hook
)

# Add watch-files for dependencies within this file
dependencies=()

while read -r file; do
  watch_file "${file}"
done < <(
  envrc-mrx show \
    watch-files \
    "${dependencies[@]}"
)

watch_file flake.lock
```

- Adds a `mrx.project.nix` containing:

```nix
{ nixpkgsSrc, system, ... }:
    project = system: let
      instantiatedNixpkgs = import nixpkgs {
        inherit system;
        overlays = [mrx.rustOverlay];
      };
    in
      mrx.mkProject {
        nixpkgs = instantiatedNixpkgs;
        pathAttrImports = {
          _ = import ./mrx.generated.nix;
        };
      };
```
