# shellcheck shell=bash
function envrc-mrx() {
  function has() {
    if command -v "${1}" >/dev/null 2>&1; then
      echo "${1}"
    else
      echo no-"${1}"
    fi
  }

  function mrx-mode() {
    has_cargo="$(has cargo)"
    has_upstream="$(has mrx)"
    case "${USE_LOCAL_MRX:-false}" in
    1 | true | yes) needs_local_cargo=needs-local-cargo ;;
    *) needs_local_cargo=no-needs-local-cargo ;;
    esac

    case "${has_cargo}:${has_upstream}:${needs_local_cargo}" in
    cargo:*:needs-local-cargo)
      echo "cargo"
      ;;
    no-cargo:*:needs-local-cargo)
      echo "build-cargo"
      ;;
    *:mrx:no-needs-local-cargo)
      echo "upstream"
      ;;
    *:no-mrx:no-needs-local-cargo)
      echo "build-upstream"
      ;;
    *)
      echo "Unhandled case: ${has_cargo}:${has_upstream}:${needs_local_cargo}" >&2

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
  build-upstream)
    nix build '#_.pkg.mrx' --out-link .direnv/mrx

    PATH_add .direnv/mrx/bin
    ;;
  cargo) ;;
  upstream) ;;
  esac

  # Run
  case "${mode}" in
  build-cargo)
    "${shell_path}"/bin/cargo run -- "${@}"
    ;;
  build-upstream)
    .direnv/mrx/bin/mrx "${@}"
    ;;
  cargo)
    cargo run -- "${@}"
    ;;
  upstream)
    mrx "${@}"
    ;;
  esac
}
