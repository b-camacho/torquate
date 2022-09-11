let
  # Pinned nixpkgs, deterministic. Last updated: 2/12/21.
  fenix = import (fetchTarball "https://github.com/nix-community/fenix/archive/main.tar.gz") { };
  pkgs = import (builtins.fetchGit {
    url = "https://github.com/NixOS/nixpkgs.git";
    ref = "master";
    rev = "14257d122fea775da8116b8628a276ae7798412e";
  }) {};
in pkgs.mkShell {
  buildInputs = [ 
    fenix.complete.toolchain
    pkgs.dbus
  ];
}

