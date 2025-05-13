A dead simple bash script manager written in Rust.

```
fastbash — quick script manager

USAGE:
    fastbash create         # Create a new script interactively
    fastbash edit <script>  # Open script for editing
    fastbash <script> [...] # Run a saved script with optional args
    fastbash ls             # List saved scripts
    fastbash rm <script>    # Delete a saved script
    fastbash help           # Show this help message

NOTES:
    - Scripts are saved in ~/.fastbash/scripts
    - Make sure your scripts start with a shebang line (e.g., #!/bin/bash)
    - Set the EDITOR env variable to control which editor is used
```
