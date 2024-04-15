# xml2arb

xml2arb is a tool which converts Android app project string resource XML files to
[ARB](https://github.com/google/app-resource-bundle/wiki/ApplicationResourceBundleSpecification)
files which Flutter supports.

Only a small subset of Android XML string resource features are currently
supported. Check [examples](#example-xml-files) to see what features are
supported.

## Usage

1. The tool is installed/compiled with Cargo, so install Rust if you don't have
   it already. <https://www.rust-lang.org/>

2. Clone repository.

3. Install the tool using Cargo.
```
cd xml2arb
cargo install --path .
```

4. Configure Flutter app project so that ARB files and command `flutter
   gen-l10n` are working. Instructions: <https://docs.flutter.dev/ui/accessibility-and-internationalization/internationalization>

5. In Flutter app project directory, create a script with the following commands:
```
xml2arb --input-dir path-to-android-app-project-res-directory --output-dir arb-files --arb-file-name-template app_en.arb
flutter gen-l10n
```

6. Run the script when XML files are modified.

7. Consider running the script automatically when XML files are modified.
For example `fswatch` can detect when a file is saved.

<https://github.com/emcrisostomo/fswatch?tab=readme-ov-file#usage>

macOS:
```
fswatch -o -e Updated android-project-res-dir | xargs -n1 -I{} ./script.sh
```

Linux:
```
fswatch -m poll_monitor -o -e Updated android-project-res-dir/values/strings.xml | xargs -n1 -I{} ./script.sh
```

## Recommended development workflow

Have a small Android project in your Flutter project repository which only
contains the string resources. This way you can edit the string resources in
Android Studio and also use the Android Studio's translation tools.

The Android project is separate from the possible Flutter app Android project
to keep Android Studio loading times at minimum.

If you have some of the previous `fswatch` commands running, Dart files are updated
automatically when you save your changes in Android Studio with Ctrl+S.

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
