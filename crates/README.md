# Crates

miscomp-issue project crates.

## `miscomp-issue`

Main entry, builds the miscomp-issue executable binary for Linux, macOS and Windows. Code is mostly startup config and CLI. Also
builds resources that are embedded on the executable file metadata on Windows.

## `miscomp-issue-mobile`

Main entry for Android, builds the miscomp-issue binary for the app package.

## `gui` 

GUI library, statically linked, but can build as cdylib for hot-reload.

The app "view" is implemented here.

## `shared`

Utils library, embedded resources.