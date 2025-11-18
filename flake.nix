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

      # With gcc (default), the WASM build fails silently
      # at runtime there is an error message similar to https://github.com/DioxusLabs/dioxus/discussions/3807.
      # this seems to be caused by the ring dependency: https://github.com/briansmith/ring/issues/1473
      CC_wasm32_unknown_unknown = "${pkgs.llvmPackages.clang-unwrapped}/bin/clang";

      # Include path to standard library is missing by default.
      CFLAGS_wasm32_unknown_unknown = "-I${pkgs.llvmPackages.clang}/resource-root/include/";
    };
  };
}
