# Hara extension declaration contract

The authoritative document is [`hara-extension.edn`](hara-extension.edn). Extensions are declared under `:project/extensions` in `project.edn`. The package builder embeds normalized declarations in the generated `package.edn`; runtimes load those declarations without a second authored manifest.
