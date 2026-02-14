# shellcheck shell=bash
function envrc-mrx() {
  function mrx-mode() {
    if command -v cargo >/dev/null 2>&1; then
      has_cargo=cargo
    else
      has_cargo=no-cargo
    fi

    if test -s .direnv/mrx/bin/mrx; then
      has_cache=mrx
    else
      has_cache=no-mrx
    fi

    case "${USE_LOCAL_MRX:-false}" in
    1 | true | yes) needs_local_cargo=needs-local-cargo ;;
    *) needs_local_cargo=no-needs-local-cargo ;;
    esac

    case "${has_cargo}:${has_cache}:${needs_local_cargo}" in
    cargo:*:needs-local-cargo)
      echo "cargo"
      ;;
    no-cargo:*:needs-local-cargo)
      echo "build-cargo"
      ;;
    *:mrx:no-needs-local-cargo)
      echo "cache"
      ;;
    *:no-mrx:no-needs-local-cargo)
      echo "build-cache"
      ;;
    *)
      echo "Unhandled case: ${has_cargo}:${has_cache}:${needs_local_cargo}" >&2

      exit 1
      ;;
    esac
  }

  # Build
  mode="$(mrx-mode)"
  echo "mode: ${mode}" >&2

  case "${mode}" in
  build-cargo)
    shell_path="$(nix build '#_.shell' --no-link --print-out-paths)"
    PATH_add "${shell_path}"/bin
    ;;
  build-cache)
    nix build '#_.pkg.mrx' --out-link .direnv/mrx

    PATH_add .direnv/mrx/bin
    ;;
  cargo) ;;
  cache) ;;
  esac

  # Run
  case "${mode}" in
  build-cargo)
    "${shell_path}"/bin/cargo run -- "${@}"
    ;;
  build-cache | cache)
    .direnv/mrx/bin/mrx "${@}"
    ;;
  cargo)
    cargo run -- "${@}"
    ;;
  esac
}
