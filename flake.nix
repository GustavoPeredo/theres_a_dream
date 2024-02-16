{
  description = "A development environment with Rust and Node.js/TypeScript";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, flake-utils, naersk, fenix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        toolchain = with fenix.packages.${system};
          combine [
            minimal.rustc
            minimal.cargo
            targets.x86_64-unknown-linux-musl.latest.rust-std
          ];

        naersk' = naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        };
      in {
        packages = {
          backend = naersk'.buildPackage {
            src = ./backend;
            doCheck = true;
            release = true;
            nativeBuildInputs = with pkgs; [ pkgsStatic.stdenv.cc ];

            # Tells Cargo that we're building for musl.
            # (https://doc.rust-lang.org/cargo/reference/config.html#buildtarget)
            CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";

            # Tells Cargo to enable static compilation.
            # (https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags)
            #
            # Note that the resulting binary might still be considered dynamically
            # linked by ldd, but that's just because the binary might have
            # position-independent-execution enabled.
            # (see: https://github.com/rust-lang/rust/issues/79624#issuecomment-737415388)
            CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
          };

          frontend = pkgs.buildNpmPackage {
            name = "frontend";

            # The packages required by the build process
            buildInputs = [
              pkgs.nodejs
            ];

            # The code sources for the package
            src = ./frontend;
            npmDepsHash = "sha256-KLO6xdSDoQLcrgBJpvl4KEgLgXkXa0iOFGPfc7tnJyc=";

            # How the output of the build phase
            installPhase = ''
              mkdir $out
              npm run build
              cp -r dist/* $out
            '';
          };

          # Write a shell script that runs the backend and frontend
          deploy = pkgs.writeScriptBin "deploy" ''
            WEB_APP_DIR=${self.packages.${system}.frontend} ${self.packages.${system}.backend}/bin/backend
            '';
          
            #WEB_APP_DIR=$out ${self.packages.${system}.backend}/bin/backend
          #};
        };
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
	    rust-analyzer
	    rustc
            rustfmt
            clippy
            cargo

            nodejs
            typescript
          ];

          shellHook = ''
            echo "Welcome to your Rust and Node.js/TypeScript development environment."
          '';
        };
      }
    );
}
