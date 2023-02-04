{ lib
, rustPlatform
, version
, clippy
, curl
, pkg-config
}:

rustPlatform.buildRustPackage {
  pname = "tiny-azagent";
  inherit version;

  src = lib.sourceByRegex ../. ["Cargo\.*" "src.*" ];
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [ curl ];

  meta = {
    description = "Minimal Azure VM provisioning agent";
    homepage = "https://sr.ht/~raphi/tiny-azagent/";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
  };
}
