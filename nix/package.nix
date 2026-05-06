{
  version,
  lib,
  installShellFiles,
  rustPlatform,
  buildFeatures ? [ ],
}:

rustPlatform.buildRustPackage {
  pname = "xpmkg";

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../src
      ../crates
      ../Cargo.lock
      ../Cargo.toml
    ];
  };

  inherit buildFeatures;
  inherit version;

  # inject version from nix into the build
  env.NIX_RELEASE_VERSION = version;

  cargoLock = {
    lockFile = ../Cargo.lock;
    outputHashes = {
      "umls-0.1.0" = "sha256-JDSh+PuqOR8em2MLN0zZKjpIqD4nH1Lb5VJOl1GpCak=";
    };
  };

  nativeBuildInputs = [
    installShellFiles

    rustPlatform.bindgenHook
  ];

  buildInputs = [ ];

  meta = with lib; {
    description = "A multitool for extending PubMed-MeSH knowledge graphs (CSV-based for Neo4J) with additional nodes, relationships, and external metadata";
    mainProgram = "xpmkg";
    homepage = "https://github.com/c2fc2f/Extend-PubMed-MeSH-KG";
    license = licenses.mit;
    maintainers = [ maintainers.c2fc2f ];
  };
}
