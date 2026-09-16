# Project Queue

A small public project-status site generated from `projects.yaml`. It creates:

- `/` — the current focus, ordered queue, and later projects
- `/help/` — open volunteer roles and information requests collected from every project

## Edit the page

Edit `projects.yaml`, then preview the generated files locally:

```bash
cargo run
python3 -m http.server 8000 --directory dist
```

Open <http://localhost:8000/>. Do not edit files in `dist/`; the next build replaces them.

The accepted commitment values are `Committed`, `Likely`, `Tentative`, `Very Tentative`, and `Unlikely`. Activity accepts `Active`, `Queued`, `Incubating`, `Waiting`, `Parked`, `Completed`, and `Abandoned`. Help status accepts `Needed`, `Tentative`, and `Found`.

## Publish at `username.github.io/projects/`

1. On GitHub, create a public repository named exactly `projects`. Do not initialize it with a README, because this folder already has one.
2. In this folder, run the commands below, replacing `YOUR_USERNAME`:

   ```bash
   git init
   git add .
   git commit -m "Create project status site"
   git branch -M main
   git remote add origin https://github.com/YOUR_USERNAME/projects.git
   git push -u origin main
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

The workflow rebuilds and republishes the site after each push to `main`.

## What `git commit -am` does

`git commit -am "Message"` combines two options:

- `-a` automatically stages changes to files Git already tracks, including deletions.
- `-m` supplies the commit message on the command line.

It does **not** add new, previously untracked files. For a new file, use `git add FILE` first—or use `git add .` and then `git commit -m "Message"`.
