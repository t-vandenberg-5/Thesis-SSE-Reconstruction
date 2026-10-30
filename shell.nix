{
  pkgs ? import <nixpkgs> { },
}:
pkgs.callPackage (
  {
    mkShell,
    rustPlatform,
    pkg-config,
  }:
  mkShell {
    strictDeps = true;
    nativeBuildInputs = [
      rustPlatform.bindgenHook
      # optional: add pkg-config support
      pkg-config
    ];
    buildInputs = [
      # add desired native packages
			pkgs.fontconfig
			pkgs.perf
    ];
    # ...
  }
) { }

