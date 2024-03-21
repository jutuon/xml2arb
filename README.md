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

6. Consider running the script automatically when XML files are modified.
For example `fswatch` can detect when a file is saved.

<https://github.com/emcrisostomo/fswatch?tab=readme-ov-file#usage>

```
fswatch -o -e Updated xml-dir | xargs -n1 -I{} ./script.sh
```

## Example XML files

`res/values/strings.xml`:
```xml
<resources>
   <string name="color_title">Selected color</string>
   <string name="color_info" description="Text for currently selected color">Red: %s, Green: %s, Blue: %s</string>

   <!-- Supported Android strings.xml escape sequences -->
   <string name="escape_sequences">\\ \' %%</string>
</resources>
```

`res/values-fi/strings.xml`:
```xml
<resources>
   <string name="color_title">Valittu väri</string>
   <string name="color_info">Punainen: %s, Vihreä: %s, Sininen: %s</string>
</resources>
```

### Generated ARB files

With command

```
xml2arb --input-dir res --output-dir arb-files --arb-file-name-template app_en.arb
```

the generated files are:

`arb-files/app_en.arb`:
```json
{
  "color_info": "Red: {param0}, Green: {param1}, Blue: {param2}",
  "@color_info": {
    "description": "Text for currently selected color",
    "placeholders": {
      "param0": {
        "type": "String"
      },
      "param1": {
        "type": "String"
      },
      "param2": {
        "type": "String"
      }
    }
  },
  "color_title": "Selected color",
  "escape_sequences": "\\ ' %"
}

```
`arb-files/app_fi.arb`:
```json
{
  "color_info": "Punainen: {param0}, Vihreä: {param1}, Sininen: {param2}",
  "color_title": "Valittu väri"
}
```

## License

MIT License or Apache License 2.0
