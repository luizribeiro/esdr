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
    # futuresdr dependencies
    cargo
    libiconv
    pkg-config
    soapysdr
    soapyrtlsdr
    darwin.objc4
  ];

  shellHook = ''
    export CC=/usr/bin/clang
    export CXX=/usr/bin/clang++
    export SOAPY_SDR_PLUGIN_PATH="${pkgs.soapyrtlsdr}/lib/SoapySDR/modules0.8"
  '';
}
