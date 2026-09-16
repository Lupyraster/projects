# Project Queue

A small public project-status site generated from `projects.yaml`. The single page shows the current focus, ordered queue, later projects, and expandable open-help details.

## Edit the page

Edit `projects.yaml`, then build and preview the site locally with one command:

```bash
cargo run -- --serve
```

The program prints the local URL, normally <http://127.0.0.1:8000/>. Press Ctrl+C to stop it. This preview does not make a commit, push anything, or contact GitHub. Do not edit files in `dist/`; the next build replaces them.

Run `cargo run` without options to build without starting the server. Run `cargo run -- --help` to see the options, including `--port` for choosing another local port.

Under `site`, only `title` and `updated` are used. The older `introduction` and `owner` fields are obsolete and can be removed.

Each project can have a `summary`, which is displayed directly below its title. The accepted commitment values are `Committed`, `Likely`, `Tentative`, `Very Tentative`, and `Unlikely`. Activity accepts `Active`, `Queued`, `Incubating`, `Waiting`, `Parked`, `Completed`, and `Abandoned`. Help status accepts `Needed`, `Tentative`, and `Found`. `summary`, `commitment`, `focus`, and `waiting_on` may be omitted or left blank. A help item's expandable description also uses `summary`; the older help-item field name `note` is still accepted.

`Committed` is intentionally not displayed for Active or Queued projects because it is normally implied there. Other commitment values still appear, and `Committed` still appears on Later projects where it conveys additional information.

## Publish at `username.github.io/projects/`

1. On GitHub, create a public repository named exactly `projects`. Do not initialize it with a README, because this folder already has one.
2. In this folder, run the commands below, replacing `YOUR_USERNAME`:

   ```bash
   git init
   git add .
   git commit -m "Create project status site"
   git branch -M master
   git remote add origin https://github.com/YOUR_USERNAME/projects.git
   git push -u origin master
   ```

3. On GitHub, open the repository, then go to **Settings → Pages**. Under **Build and deployment**, set **Source** to **GitHub Actions**.
4. Open the repository's **Actions** tab. The “Deploy project status site” workflow should run after the push. Its deploy job will show the published URL.

The repository name controls the default project-site path. A repository named `projects` publishes at `https://YOUR_USERNAME.github.io/projects/`. The special repository name `YOUR_USERNAME.github.io` instead publishes an account site at `https://YOUR_USERNAME.github.io/`.

Future status updates are normally just:

```bash
git add projects.yaml
git commit -m "Update project statuses"
git push
```

The workflow rebuilds and republishes the site after each push to `master`.

## What `git commit -am` does

`git commit -am "Message"` combines two options:

- `-a` automatically stages changes to files Git already tracks, including deletions.
- `-m` supplies the commit message on the command line.

It does **not** add new, previously untracked files. For a new file, use `git add FILE` first—or use `git add .` and then `git commit -m "Message"`.
