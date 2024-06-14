{
  mkShell,
  lib,
  rust-analyzer-unwrapped,
  rustfmt,
  clippy,
  cargo,
  rustc,
  rustPlatform,
  openssl,
  pkg-config
}:
mkShell {
  strictDeps = true;

  nativeBuildInputs = [
    cargo
    rustc

    rust-analyzer-unwrapped
    rustfmt
    clippy
  ];

  env = {
    RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
  };

  #shellHook = ''
  #  export PKG_CONFIG_PATH="${openssl.dev}/lib/pkgconfig";
  #'';
}
