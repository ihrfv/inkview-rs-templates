{ pkgs, lib, config, inputs, ... }:

let
  # Target package set for the armv7 PocketBook runtime environment
  pkgsCross = pkgs.pkgsCross.armv7l-hf-multiplatform;
{% if framework == "slint" %}
  # ... and for the x86_64 linux desktop emulator (see utils/emulator/). Only the
  # slint variant links C libraries, so this is the only variant that pays for
  # the cross build of fontconfig and friends.
  pkgsCrossEmu = pkgs.pkgsCross.gnu64;
{% endif %}
in
{
  # https://devenv.sh/packages/
  packages = [
    pkgs.git
    pkgs.git-cliff
    pkgs.just
    pkgs.secretspec

    # Native cross-compilation toolchain
    pkgs.zig
    pkgs.cargo-zigbuild

    # Use the target-specific cross-pkg-config tool
    pkgsCross.buildPackages.pkg-config
{% if framework == "slint" %}
    pkgsCrossEmu.buildPackages.pkg-config
{% endif %}
  ];

  # https://devenv.sh/languages/
  languages.rust = {
    enable = true;
    channel = "stable";
    targets = [
      "armv7-unknown-linux-gnueabi"
      # The desktop emulator runs host x86_64 linux binaries (see utils/emulator/).
      "x86_64-unknown-linux-gnu"
    ];
  };

  env = {
    PKG_CONFIG_ALLOW_CROSS = "1";

    # Provide the combined search paths for fontconfig and its system dependencies
    PKG_CONFIG_PATH_armv7_unknown_linux_gnueabi = lib.makeSearchPath "lib/pkgconfig" [
      pkgsCross.fontconfig.dev
      pkgsCross.freetype.dev
      pkgsCross.expat.dev
      pkgsCross.libpng.dev
      pkgsCross.zlib.dev
    ];
{% if framework == "slint" %}
    # Both cross pkg-config packages export a global PKG_CONFIG, so whichever
    # lands last would answer for every target. Name the wrapper per target
    # instead, which makes the two independent of package order.
    PKG_CONFIG_armv7_unknown_linux_gnueabi = "armv7l-unknown-linux-gnueabihf-pkg-config";

    # Same search paths again, for the emulator target.
    PKG_CONFIG_x86_64_unknown_linux_gnu = "x86_64-unknown-linux-gnu-pkg-config";
    PKG_CONFIG_PATH_x86_64_unknown_linux_gnu = lib.makeSearchPath "lib/pkgconfig" [
      pkgsCrossEmu.fontconfig.dev
      pkgsCrossEmu.freetype.dev
      pkgsCrossEmu.expat.dev
      pkgsCrossEmu.libpng.dev
      pkgsCrossEmu.zlib.dev
    ];

    # slint-build embeds glyphs at compile time and resolves "sans-serif" on the
    # *build* machine. A shell without discoverable system fonts fails with
    # "could not determine a default font for sans-serif", so name the font
    # explicitly -- this also keeps the embedded font identical on every host.
    SLINT_DEFAULT_FONT = "${pkgs.dejavu_fonts}/share/fonts/truetype/DejaVuSans.ttf";
{% endif %}
  };
}
