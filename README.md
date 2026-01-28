# CTF Man

A lightweight Rust tool for managing CTF challenge directories.

## Core Workflow

```bash
# 1. Select active CTF
ctf

# 2. Create challenge folders
ctf pwn simple_notes     # Creates: ./active-ctf/pwn/simple_notes/
ctf web api_exploit      # Creates: ./active-ctf/web/api_exploit/
ctf crypto rsa_baby      # Creates: ./active-ctf/crypto/rsa_baby/
```

## Quick Start

### Shell Integration (for auto-cd)

To automatically change into newly created challenge directories, add the following to your shell rc:

```bash
# Set this to the path of your ctf-man binary
CTF_MAN_BIN="/path/to/ctf-man"

ctf() {
    local output exit_code path temp_file

    # Temp file for navigation path
    temp_file="${TMPDIR:-/tmp}/ctf-man.path"

    # No arguments = TUI mode, run directly to preserve TTY
    if [ $# -eq 0 ]; then
        "$CTF_MAN_BIN"
        exit_code=$?

        # Check if navigation path was written to temp file
        if [ $exit_code -eq 0 ] && [ -f "$temp_file" ]; then
            path=$(cat "$temp_file")
            rm -f "$temp_file"

            if [ -d "$path" ]; then
                cd "$path" || return 1
            fi
        fi

        return $exit_code
    fi

    # CLI mode - capture output for error handling
    output=$("$CTF_MAN_BIN" "$@" 2>&1)
    exit_code=$?

    if [ $exit_code -eq 0 ]; then
        # Check temp file for navigation path
        if [ -f "$temp_file" ]; then
            path=$(cat "$temp_file")
            rm -f "$temp_file"

            if [ -d "$path" ]; then
                echo "Created and entering: $path"
                cd "$path" || return 1
            fi
        else
            printf '%s\n' "$output"
        fi
    else
        printf '%s\n' "$output" >&2
        return $exit_code
    fi
}
```

## Templates

After first launch and initial config a templates folder will be created in your ctf folder.
These templates will be copied over to every new challenge (so all files in `templates/web` will be copied over to all web challenges). Custom category names are supported