# Mitwirken

- Arbeiten Sie nur mit Informationen, die Sie rechtmäßig erlangt haben (eigene Geräte, öffentliche Dokumentation, passive Beobachtung).
- Kein Quellcode, keine Grafiken oder Marken der Original-App in Beiträge übernehmen.
- Keine Schlüssel, Tokens, Seriennummern oder Konto-Daten posten (siehe SECURITY.md).
- Tests: `python -m pytest tests` (Python), `cargo test -p aosu-core` (Rust), `npm test` in `app/` (Frontend). Neue Funktionen brauchen Tests.
- Rust unter Windows ohne Visual Studio Build Tools: `winget install BrechtSanders.WinLibs.POSIX.UCRT --scope user`, dann `rustup toolchain install stable-x86_64-pc-windows-gnu && rustup default stable-x86_64-pc-windows-gnu`. Die projektlokale `.cargo/config.toml` zeigt bereits auf den MinGW-Linker.
- Änderungen als Pull Request, kleine und fokussierte Commits. Eine Aussage zu einem Protokoll muss belegt sein (Messung oder Quelle) und als belegt/Hypothese gekennzeichnet werden.
- Mit dem Beitrag stimmen Sie der Lizenzierung unter MIT-Lizenz zu. Code aus Projekten mit restriktiver Lizenz (z. B. PolyForm Strict) darf nicht übernommen werden.
