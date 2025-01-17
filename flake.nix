{
    description = "The Eikumi Discord Bot";

    inputs = {
        flake-utils.url = "github:numtide/flake-utils";
        poetry2nix.url = "github:nix-community/poetry2nix";
        # flake-compat = {
        #     url = "github:edolstra/flake-compat";
        #     flake = false;
        # };
    };

    outputs = { self, nixpkgs, flake-utils, poetry2nix }:
        flake-utils.lib.eachDefaultSystem (system:
            let
                pkgs = nixpkgs.legacyPackages.${system};
                poetry-env = pkgs.poetry2nix.mkPoetryEnv { projectDir = ./.; };
                inherit (poetry2nix.lib.mkPoetry2Nix { inherit pkgs; }) mkPoetryPackages;
                packages = with pkgs; [
                    postgresql.lib
                ];
                packagesWithPoetry = (mkPoetryPackages {
                    projectDir = ./.;
                }).poetryPackages ++ packages;
            in
                {
                    devShell = pkgs.mkShell {
                        buildInputs = packagesWithPoetry;
                        LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath packagesWithPoetry}:$LD_LIBRARY_PATH";
                    };
                    packages = {
                        default = packagesWithPoetry;
                    };
                }
        );
}