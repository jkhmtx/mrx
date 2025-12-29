# \_.run.many

Given a list of apps, run each app in the order given by `each`.

Produces a `nixpkgs.writeShellApplication` that handles the boilerplate of composing several other applications run in series.

Requires `name` and `each` arguments.

May be provided `extraRuntimeEnv`, which allows specifying an environment variable to all the inner scripts.
