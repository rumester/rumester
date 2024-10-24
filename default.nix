let pkgs = import <nixpkgs> {};
in pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    rustc
    pkg-config
  ];
  buildInputs = with pkgs; [
    openssl
  ];
}
