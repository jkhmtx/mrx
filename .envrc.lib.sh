# shellcheck shell=bash
function envrc-mrx() {
  function has() {
    if command -v "${1}" >/dev/null 2>&1; then
      echo "${1}"
    else
      echo no-"${1}"
    fi
  }

  function strategy() {
    has_cargo="$(has cargo)"
    has_upstream="$(has upstream)"
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
    *:upstream:no-needs-local-cargo)
      echo "upstream"
      ;;
    *:no-upstream:no-needs-local-cargo)
      echo "build-upstream"
      ;;
    *)
      echo "Unhandled case: ${has_cargo}:${has_upstream}:${needs_local_cargo}" >&2

      exit 1
      ;;
    esac
  }

  # Build
  case "$(strategy)" in
  build-cargo | build-upstream)
    paths="$(nix build '#_.shell' --no-link --print-out-paths)"

    while read -r path; do
      echo "${path}" >&2
      PATH_add "${path}"/bin
    done <<<"${paths}"
    ;;
  cargo) ;;
  upstream) ;;
  esac

  # Run
  case "$(strategy)" in
  build-cargo)
    echo "Cargo should be in PATH"
    exit 1
    ;;
  build-upstream)
    echo "mrx should be in PATH"
    exit 1
    ;;
  cargo)
    cargo run -- "${@}"
    ;;
  upstream)
    mrx "${@}"
    ;;
  esac
}
