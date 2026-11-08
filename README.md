# backupcopy

`backupcopy` is a small command-line tool for Windows that makes the destination directory an exact
**mirror** of the source directory. It copies what is new or changed, removes what is gone, and lets
you leave out files and directories with `.backup_rules` files kept in the source tree.

> **Warning:** the tool **deletes** anything in the destination that is not in the source, or that
> the rules exclude. Point it at a dedicated backup directory, never at a directory that holds other
> data.

## Building

You need a Rust toolchain (edition 2021). The tool uses `std::os::windows`, so it builds and runs
only on Windows.

```sh
cargo build --release
```

The binary is written to `target\release\backupcopy.exe`.

## Usage

```
backupcopy [-v] [--debug] [--remove-ro] <source> <destination>
```

| Argument        | Meaning                                                                       |
|-----------------|-------------------------------------------------------------------------------|
| `<source>`      | Directory to back up. It must exist.                                          |
| `<destination>` | Directory that receives the mirror. **It must already exist**; it is not created. |
| `-v`, `--verbose` | Print the scan phases and how many items were found on each side.           |
| `--debug`       | Print the parsed arguments.                                                   |
| `--remove-ro`   | Clear the read-only attribute on a destination file before deleting it.      |

Options can go anywhere on the command line. The first two other arguments are taken as source and
destination; any further argument is an error.

Example:

```sh
backupcopy -v --remove-ro D:\Projects E:\Backup\Projects
```

There is no dry-run mode: every planned action runs straight away.

## How it works

A run has three phases.

### 1. Scan

Both trees are walked recursively. Each file is recorded with its size and last-write time; each
directory is recorded by its path. Items are keyed by their path **relative to the root**, so
`D:\Projects\app\main.rs` and `E:\Backup\Projects\app\main.rs` are treated as the same item.

While scanning the **source**, every file named `.backup_rules` is loaded (see
[Rules files](#rules-files)). Rules files in the destination are ignored.

Symbolic links are neither files nor directories to the scanner, so they are skipped.

### 2. Plan

Each item in the destination gets one of these decisions:

| Destination item                                   | Decision |
|----------------------------------------------------|----------|
| Not in the source                                  | delete   |
| A file in one tree and a directory in the other    | delete   |
| File; the source copy is newer **or** the size differs | delete (it will be copied again) |
| File; the rules exclude it                         | delete   |
| File; same size and the source is not newer        | keep as is |
| Directory; the rules exclude it                    | delete   |
| Directory; otherwise                               | keep as is |

Each item in the source then gets one of these:

| Source item                                        | Decision |
|----------------------------------------------------|----------|
| Already in the destination and kept as is          | skip     |
| Excluded by the rules                              | skip     |
| Otherwise                                          | copy     |

Files are compared only by size and last-write time, never by content. A destination file that is
*newer* than the source but the same size is left alone.

### 3. Execute

The actions run in this order:

1. Delete the destination files marked for deletion (clearing the read-only attribute first when
   `--remove-ro` is given).
2. Delete the destination directories marked for deletion, deepest first. A directory is removed only
   when it is empty.
3. Create the missing directories in the destination.
4. Copy the files, printing progress as `n/total Copy "<from>" to "<to>" ... OK`.

If a single file or directory can't be deleted or copied, the error is printed and the run goes on.
Failing to scan a tree, read a rules file, or create a directory stops the run.

## Rules files

To leave items out of the backup, put a file named `.backup_rules` in any directory of the
**source** tree. Its rules apply to that directory and everything below it. You can have several
rules files at different levels, and they all apply together.

### Syntax

One rule per line:

```
<prefix>: <regular expression>
```

| Prefix                        | Effect                                        |
|-------------------------------|-----------------------------------------------|
| `skip:`, `remove:`, `-:`      | **Exclude** matching items from the backup    |
| `keep:`, `add:`, `+:`         | **Include** matching items, overriding any exclusion |

- Blank lines and lines that start with `#` are ignored.
- Spaces around the line and after the colon are trimmed.
- A line with any other prefix, or with an invalid regular expression, stops the run before
  anything is copied or deleted. The tool prints the rules file with the bad line marked `>>`,
  followed by the line number and the details of the error.

### How matching works

- The expression uses [Rust `regex`](https://docs.rs/regex/latest/regex/#syntax) syntax.
- It is matched against the item's path **relative to the directory that holds the rules file**,
  for example `build\output\app.exe`, not against the full path.
- Paths use Windows separators, so match a separator with `\\` (an escaped backslash).
- A match anywhere in the path counts. Use `^` and `$` to anchor: `tmp` also matches `src\tmpfile.rs`.
- Rules apply to files and directories alike.
- **Include wins.** If any `keep` rule matches an item, it is backed up, whatever exclude rules
  also match and wherever their rules files are. Otherwise, if any exclude rule matches, it is left
  out. Items no rule matches are backed up.

### Excluding a whole directory

Every file and subdirectory is checked against the rules on its own. A pattern that matches only the
directory's name does not exclude what is inside it. For example, `skip: ^build$` excludes the `build`
directory but not `build\app.exe`. The tool then tries to copy that file into a directory it never
created, and the copy fails.

Use a pattern that matches the directory **and its contents**:

```
skip: ^build(\\|$)
```

The same goes for a directory nested at any depth:

```
skip: (^|\\)node_modules(\\|$)
```

### Example

`D:\Projects\.backup_rules`:

```
# Build output and dependencies
skip: (^|\\)target(\\|$)
skip: (^|\\)node_modules(\\|$)

# Temporary and log files anywhere
-: \.(tmp|log|bak)$

# ...but keep the release notes log
+: ^docs\\changes\.log$
```

`D:\Projects\website\.backup_rules` (applies only inside `website`):

```
skip: ^cache(\\|$)
```

### Notes

- The `.backup_rules` files are backed up like any other file, unless a rule excludes them.
- Items that the rules exclude are **also deleted from the destination** if they are there, for
  example from an earlier run before you added the rule.

## Read-only files

Copying keeps the source file's read-only attribute, so the destination can hold read-only files
that later have to be deleted. `--remove-ro` clears the attribute before each delete.

Whether you need it depends on the Rust version the tool was built with: older standard libraries
refuse to delete read-only files on Windows, while recent ones (such as Rust 1.99) delete them
anyway.

## License

See [LICENSE](LICENSE).
