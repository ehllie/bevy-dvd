{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    let name = "bevy-dvd"; in {
      overlays.default = final: prev:
        let
          toml = builtins.fromTOML (builtins.readFile "${self}/Cargo.toml");
          inherit (final)
            alsa-lib
            darwin
            lib
            libxkbcommon
            lld
            makeWrapper
            pkg-config
            rustPlatform
            stdenv
            udev
            vulkan-loader
            wayland
            xorg;

          package = rustPlatform.buildRustPackage ({
            pname = name;
            src = self;
            inherit (toml.package) version;
            cargoLock.lockFile = "${self}/Cargo.lock";

            nativeBuildInputs = [ lld makeWrapper ] ++
              lib.optionals stdenv.isLinux [
                pkg-config
              ];

            buildInputs = lib.optionals stdenv.isLinux [
              alsa-lib
              libxkbcommon
              udev
              vulkan-loader
              wayland
              xorg.libX11
              xorg.libXcursor
              xorg.libXi
              xorg.libXrandr
            ] ++ lib.optionals stdenv.isDarwin [
              darwin.apple_sdk.frameworks.Cocoa
              rustPlatform.bindgenHook
            ];

            postInstall = ''
              mkdir -p $out/share
              cp -r assets $out/share
                wrapProgram "$out/bin/${toml.package.name}" \
                  --set-default CARGO_MANIFEST_DIR $out/share
            '';

          });
        in
        {
          ${name} = package;
          cache = prev.mkShell {
            name = "${name}-devShell";
            inputsFrom = [ package ];
            packages = with prev; [
              rust-analyzer
              clippy
              rustfmt
            ];
            shellHook = ''
              export PATH=$PWD/target/debug:$PATH
            '';
          };
        }
      ;
    } //

    (flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ self.overlays.default ];
        };
        inherit (pkgs) cache;
        package = pkgs.${name};
      in
      {
        packages = {
          inherit cache;
          default = package;
          ${name} = package;
        };
        devShells.default = cache;
      }));
}
