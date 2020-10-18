{ pkgs ? import <nixpkgs> { } }:
pkgs.rustPlatform.buildRustPackage {
  name = "humblegen";
  src = ./.;
  cargoSha256 = "0jmiqvlg3lkx742vggnsnbwn27mh4kx722imlk1kyimq4d7dlsyz";

  # Tests are currently disabled, as they spew quite a bit of output and may fail.
  doCheck = false;
}
