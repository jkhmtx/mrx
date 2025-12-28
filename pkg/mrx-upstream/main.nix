{
  nixpkgs,
  upstreamMrx,
  ...
}:
nixpkgs.writeShellApplication {
  name = "mrx";

  runtimeInputs = [upstreamMrx];

  text = builtins.readFile ./run.sh;
}
