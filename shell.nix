{ pkgs ? import <nixpkgs> {} }:

pkgs.stdenv.mkDerivation {
  name = "rdiff";
  nativeBuildInputs = with pkgs; [
    gcc
  ];
}