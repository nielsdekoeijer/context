{ mkShell, rust, ... }:
mkShell {
  nativeBuildInputs = [
    rust
  ];
  shellHook = ''
    PS1="(dev) $PS1"
  '';
}
