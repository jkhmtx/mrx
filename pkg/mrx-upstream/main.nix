{
  nixpkgs,
  upstreamMrx,
  ...
}:
nixpkgs.writeShellApplication {
  name = "mrx-upstream";

  runtimeInputs = [upstreamMrx];

  text = builtins.readFile ./run.sh;
}
