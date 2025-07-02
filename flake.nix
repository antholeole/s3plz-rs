{
  description = "a super simple static s3 host";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
  };

  outputs = {
    nixpkgs,
    self,
    ...
  }: let
    system = "x86_64-linux"; # your version
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    devShells.${system}.default =
      pkgs.mkShell
      {
        packages = with pkgs; [
          rustc
          cargo
        ];
      };

    packages.${system}.default = pkgs.rustPlatform.buildRustPackage (finalAttrs: {
      pname = "s3plz";
      version = "1.0.0";

      src = self;
      cargoHash = "sha256-XKITwRrzWwKf1ofR+r2jMkWzUkufFhTKOTsvL++ONK8=";

      meta.main = "s3plz";
    });
  };
}
