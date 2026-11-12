{
  description = "eep - keep your computer awake, and watch something nice in the terminal";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (pkgs: {
        eep = pkgs.stdenvNoCC.mkDerivation {
          pname = "eep";
          version = self.shortRev or self.dirtyShortRev or "dev";
          src = ./.;

          # Only needed so patchShebangs rewrites `#!/usr/bin/env python3`.
          buildInputs = [ pkgs.python3 ];

          dontBuild = true;
          installPhase = ''
            runHook preInstall
            install -Dm755 eep $out/bin/eep
            runHook postInstall
          '';

          # systemd-inhibit / caffeinate are taken from the host's PATH on purpose:
          # they must talk to the running system's logind / power management.
          meta = {
            description = "Keep your computer awake for a while, with a terminal animation";
            mainProgram = "eep";
            platforms = systems;
          };
        };
        default = self.packages.${pkgs.stdenv.hostPlatform.system}.eep;
      });

      apps = forAllSystems (pkgs: {
        default = {
          type = "app";
          program = nixpkgs.lib.getExe self.packages.${pkgs.stdenv.hostPlatform.system}.eep;
        };
      });

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell { packages = [ pkgs.python3 ]; };
      });

      formatter = forAllSystems (pkgs: pkgs.nixfmt);
    };
}
