{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = {nixpkgs, ...}: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    devShells.${system}.default = pkgs.mkShell {
      packages = with pkgs; [
        dioxus-cli
        wasm-bindgen-cli_0_2_100
      ];

      # https://docs.rs/getrandom/0.3.3/getrandom/#webassembly-support
      RUSTFLAGS = "--cfg getrandom_backend=\"wasm_js\"";

      # gcc is default
      CC_wasm32_unknown_unknown = "${pkgs.llvmPackages.clang-unwrapped}/bin/clang";

      # include path to standard library is missing by default
      CFLAGS_wasm32_unknown_unknown = "-I${pkgs.llvmPackages.clang}/resource-root/include/";
    };
  };
}
