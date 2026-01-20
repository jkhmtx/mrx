# \_.mrx.run.many

Given a list of apps, run each app in the order given by `each`.

Produces a `nixpkgs.writeShellApplication` that handles the boilerplate of composing several other applications run in series.

Requires `name` and `each` arguments.

May be provided `extraRuntimeEnv`, which allows specifying an environment variable to all the inner scripts.

Caller may pass `keepGoing = true` if the script should continue if one of the steps failed. If a step does fail, the process reports exit code 1 and summarizes the failing step from `each` by printing the derivation's `getName`. If `keepGoing = "no-stderr-summary"` is passed, the same behavior applies, but printing the report is skipped.
