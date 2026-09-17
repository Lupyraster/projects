# Project Queue

A static public status page generated from `projects.yaml`. It shows the current focus, the ordered queue, later projects, progress, and open requests for help.

## Local development

Rust is the only required dependency. Build the site and start the local preview server with:

```bash
cargo run -- --serve
```

The command prints the local URL. Press Ctrl+C to stop the server. Other available commands are:

```bash
cargo run                         # Build into dist/
cargo run -- --help               # Show command-line options
cargo run -- --serve --port 8080  # Use another local port
```

Files in `dist/` are generated and replaced on every build. Edit `projects.yaml`, `src/main.rs`, or files under `static/` instead.

## Project data

All displayed content comes from `projects.yaml`. The `site` object contains `title` and `updated`. Projects appear in file order and support these fields:

- `title` and optional `summary`
- `activity`: `Active`, `Queued`, `Incubating`, `Waiting`, `Parked`, `Completed`, or `Abandoned`
- optional `commitment`: `Committed`, `Likely`, `Tentative`, `Very Tentative`, or `Unlikely`
- optional `focus` and `waiting_on`
- optional `steps` and `help` lists

Help items use `Needed`, `Tentative`, or `Found` for `status`. Their optional `summary` text appears when the item is expanded. Found items remain associated with their projects but are omitted from Help Wanted.

Committed badges are hidden for Active and Queued projects, where they are normally redundant. Other commitment values remain visible.

## GitHub Pages

The included workflow builds and deploys the site whenever the `master` branch is pushed. To publish at `https://YOUR_USERNAME.github.io/projects/`:

1. Create a GitHub repository named `projects`.
2. Push this repository to its `master` branch.
3. Open **Settings → Pages** on GitHub and set **Source** to **GitHub Actions**.

For the initial push:

```bash
git init
git add .
git commit -m "Create project status site"
git branch -M master
git remote add origin https://github.com/YOUR_USERNAME/projects.git
git push -u origin master
```

Later content updates normally require only:

```bash
git add projects.yaml
git commit -m "Update project statuses"
git push
```
