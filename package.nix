{
  stdenv,
  lib,
  rustPlatform,
  installShellFiles,
  makeBinaryWrapper,
  darwin,
  rev ? "dirty",
}:
let
  runtimeDeps = lib.optionals [];
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
in
  rustPlatform.buildRustPackage {
    pname = "magnetz";
    version = "${cargoToml.package.version}-${rev}";

    src = lib.fileset.toSource {
      root = ./.;
      fileset =
        lib.fileset.intersection
        (lib.fileset.fromSource (lib.sources.cleanSource ./.))
        (lib.fileset.unions [
          ./src
          ./Cargo.toml
          ./Cargo.lock
        ]);
    };

    strictDeps = true;

    nativeBuildInputs = [
      installShellFiles
      makeBinaryWrapper
    ];

    buildInputs = lib.optionals stdenv.isDarwin [darwin.apple_sdk.frameworks.SystemConfiguration];

    doCheck = true;

    postFixup = ''
      wrapProgram $out/bin/magnetz \
        --prefix PATH : ${lib.makeBinPath runtimeDeps}
    '';

    cargoLock.lockFile = ./Cargo.lock;

    meta = {
      description = "Get stats from torrents";
      mainProgram = "magnetz";
    };
  }
