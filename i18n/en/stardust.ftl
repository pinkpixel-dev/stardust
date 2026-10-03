app-title = Stardust
app-comment = Save, preview, create, and switch COSMIC themes
about = About
repository = Repository
file = File
view = View
size-small = Small Previews
size-medium = Medium Previews
size-large = Large Previews
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

compat-newer = Your COSMIC desktop uses theme format v{ $desktop }, but this Stardust writes v{ $app }. Update Stardust before applying themes.
compat-older = Your COSMIC desktop uses theme format v{ $desktop }, but this Stardust writes v{ $app }. Update COSMIC before applying themes.

apply-theme = Apply { $name }
active-theme = Current theme
applied = Applied { $name }
apply-failed = Couldn't apply { $name }: { $error }

icons = Icons
no-icon-themes = No icon themes found
import-icon-themes = Import Icon Themes…
import-icon-folder = Import Icon Folder…
active-icon-theme = Current icon theme
icons-installed = { $installed ->
    [one] Installed 1 icon theme
   *[other] Installed { $installed } icon themes
}{ $failed ->
    [0] {""}
   *[other] , skipped { $failed }
}
icons-applied-restart = Applied { $name }. Apps that were already open use it after a restart.
task-failed = Something went wrong. Check the terminal output for details.
