app-title = Starcoat
app-comment = Save, preview, create, and switch COSMIC themes
about = About
repository = Repository
file = File
view = View
themes = Themes
no-themes = No saved themes yet
import-themes = Import Themes…
import-folder = Import Folder…
dark = Dark
light = Light
my-theme = My Theme

imported = { $imported ->
    [one] Imported 1 theme
   *[other] Imported { $imported } themes
}{ $failed ->
    [0] {""}
   *[other] , skipped { $failed } that couldn't be read
}
import-failed = Couldn't import { $name }: { $error }
dialog-failed = Couldn't open the file picker: { $error }
library-failed = Couldn't open the theme library: { $error }

compat-newer = Your COSMIC desktop uses theme format v{ $desktop }, but this Starcoat writes v{ $app }. Update Starcoat before applying themes.
compat-older = Your COSMIC desktop uses theme format v{ $desktop }, but this Starcoat writes v{ $app }. Update COSMIC before applying themes.
