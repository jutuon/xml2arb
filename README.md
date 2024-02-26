# xml2arb

Convert Android app project string resource XML files to ARB files, which
contain strings for Flutter app projects.

## Usage

1. Clone repository.
2. Install the tool using cargo.
```
cd xml2arb
cargo install --path .
```
3. Configure Flutter app project so that ARB files and command `flutter
   gen-l10n` are working. Instructions: <https://docs.flutter.dev/ui/accessibility-and-internationalization/internationalization>

4. In Flutter app project directory, create a script with the following commands:
```
xml2arb --input-dir path-to-android-app-project-res-directory --output-dir arb-files --arb-file-name-template app_en.arb
flutter gen-l10n
```

5. Run the script when XML files are modified.

## License

MIT License or Apache License 2.0
