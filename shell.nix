let
  overlays = [
    (final: prev: {
      soapysdr = prev.soapysdr.overrideAttrs (_old: {
        patches = [ ./soapysdr.patch ];
      });
      soapyrtlsdr = prev.soapyrtlsdr.overrideAttrs (old: {
        nativeBuildInputs = old.nativeBuildInputs ++ [
          prev.darwin.libobjc
          prev.darwin.apple_sdk.frameworks.IOKit
          prev.darwin.apple_sdk.frameworks.Security
        ];
      });
    })
  ];
in
{ pkgs ? import <nixpkgs> { inherit overlays; } }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    # dev tools
    rust-analyzer
    rustfmt

    # futuresdr dependencies
    cargo
    rustc
    libiconv
    pkg-config
    soapysdr
    soapyrtlsdr
    darwin.objc4
  ];

  shellHook = ''
    # this is needed so we use clang from xcode instead of the nixpkgs'
    # apple_sdk. it is only necessary because apple_sdk_11_0 isn't available
    # yet for x86_64, so we can't use frameworks.System (which is required
    # by futuresdr)
    export CC=/usr/bin/clang
    export CXX=/usr/bin/clang++
    export PATH=/usr/bin:$PATH

    export SOAPY_SDR_PLUGIN_PATH="${pkgs.soapyrtlsdr}/lib/SoapySDR/modules0.8"
  '';
}
