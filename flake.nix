{
  description = "A fast, minimal Hyprland keybind cheat sheet and editor written in Rust/GTK4";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      nixpkgsFor = forAllSystems (system: import nixpkgs { inherit system; });
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgsFor.${system};
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "hyprkcs";
            version = "1.12.5";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = [
              pkgs.pkg-config
              pkgs.wrapGAppsHook4 # Wraps the app with necessary GTK env vars
            ];

            buildInputs = [
              pkgs.gtk4
              pkgs.libadwaita
              pkgs.gtk4-layer-shell
            ];

            # Rename binary to match pname (lowercase)
            postInstall = ''
              mv $out/bin/hyprKCS $out/bin/hyprkcs
            '';

            meta = with pkgs.lib; {
              description = "A fast, minimal Hyprland keybind cheat sheet written in Rust/GTK4";
              homepage = "https://github.com/kosa12/hyprKCS";
              license = licenses.mit;
              maintainers = [ ]; # Add yourself if you publish to nixpkgs
            };
          };
        });

      devShells = forAllSystems (system:
        let
          pkgs = nixpkgsFor.${system};
        in
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              cargo
              rustc
              rustfmt
              rust-analyzer
              pkg-config
              gtk4
              libadwaita
              gtk4-layer-shell
              fuzzy-matcher
            ];
          };
        });
    };
}
