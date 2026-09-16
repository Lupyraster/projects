use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct Data {
    site: Site,
    projects: Vec<Project>,
}

#[derive(Deserialize)]
struct Site {
    title: String,
    updated: String,
}

#[derive(Deserialize)]
struct Project {
    title: String,
    activity: String,
    commitment: String,
    focus: String,
    waiting_on: Option<String>,
    #[serde(default)]
    steps: Vec<Step>,
    #[serde(default)]
    help: Vec<HelpItem>,
}

#[derive(Deserialize)]
struct Step {
    name: String,
    status: String,
}

#[derive(Deserialize)]
struct HelpItem {
    role: String,
    status: String,
    kind: Option<String>,
    note: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw = fs::read_to_string("projects.yaml")?;
    let data: Data = serde_yaml::from_str(&raw)?;
    validate(&data)?;

    let output = Path::new("dist");
    if output.exists() {
        fs::remove_dir_all(output)?;
    }
    fs::create_dir_all(output.join("help"))?;
    fs::create_dir_all(output.join("assets"))?;

    fs::write(output.join("index.html"), render_queue(&data))?;
    fs::write(output.join("help/index.html"), render_help(&data))?;
    fs::copy("static/style.css", output.join("assets/style.css"))?;
    fs::copy("static/favicon.svg", output.join("assets/favicon.svg"))?;
    fs::write(output.join(".nojekyll"), "")?;

    let public_count = data
        .projects
        .iter()
        .filter(|project| !matches!(project.activity.as_str(), "Completed" | "Abandoned"))
        .count();
    let help_count = open_help(&data).len();
    println!("Built {public_count} projects and {help_count} open help items in dist/");
    Ok(())
}

fn validate(data: &Data) -> Result<(), String> {
    let commitments = HashSet::from([
        "Committed",
        "Likely",
        "Tentative",
        "Very Tentative",
        "Unlikely",
    ]);
    let activities = HashSet::from([
        "Active",
        "Queued",
        "Incubating",
        "Waiting",
        "Parked",
        "Completed",
        "Abandoned",
    ]);
    let help_statuses = HashSet::from(["Needed", "Tentative", "Found"]);

    if data.site.title.trim().is_empty() {
        return Err("site.title cannot be empty".into());
    }
    for project in &data.projects {
        if project.title.trim().is_empty() {
            return Err("Every project needs a title".into());
        }
        if !commitments.contains(project.commitment.as_str()) {
            return Err(format!(
                "{}: unknown commitment '{}'",
                project.title, project.commitment
            ));
        }
        if !activities.contains(project.activity.as_str()) {
            return Err(format!(
                "{}: unknown activity '{}'",
                project.title, project.activity
            ));
        }
        for item in &project.help {
            if !help_statuses.contains(item.status.as_str()) {
                return Err(format!(
                    "{}: unknown help status '{}'",
                    project.title, item.status
                ));
            }
        }
    }
    Ok(())
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#039;")
}

fn slug(value: &str) -> String {
    let mut result = String::new();
    let mut last_was_dash = false;
    for character in value.to_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            result.push(character);
            last_was_dash = false;
        } else if !last_was_dash && !result.is_empty() {
            result.push('-');
            last_was_dash = true;
        }
    }
    result.trim_end_matches('-').to_string()
}

fn pill(text: &str, kind: &str) -> String {
    format!(
        r#"<span class="pill pill--{}">{}</span>"#,
        slug(kind),
        escape(text)
    )
}

fn nav(data: &Data, active: &str, prefix: &str) -> String {
    let queue_current = if active == "queue" {
        r#" aria-current="page""#
    } else {
        ""
    };
    let help_current = if active == "help" {
        r#" aria-current="page""#
    } else {
        ""
    };
    let home = if prefix.is_empty() { "./" } else { prefix };
    format!(
        r#"
  <nav class="site-nav" aria-label="Primary navigation">
    <a class="brand" href="{home}">{}</a>
    <div class="nav-right">
      <span class="updated">Updated {}</span>
      <div class="nav-links">
        <a href="{home}"{queue_current}>Queue</a>
        <a href="{prefix}help/"{help_current}>Help wanted</a>
      </div>
    </div>
  </nav>"#,
        escape(&data.site.title),
        escape(&data.site.updated)
    )
}

fn head(title: &str, description: &str, asset_prefix: &str) -> String {
    format!(
        r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="description" content="{}">
  <meta name="color-scheme" content="light">
  <meta name="theme-color" content="#75433f">
  <title>{}</title>
  <link rel="icon" href="{asset_prefix}assets/favicon.svg" type="image/svg+xml">
  <link rel="stylesheet" href="{asset_prefix}assets/style.css">
</head>"##,
        escape(description),
        escape(title)
    )
}

fn foot() -> &'static str {
    "\n</body>\n</html>"
}

fn open_help(data: &Data) -> Vec<(&Project, &HelpItem)> {
    data.projects
        .iter()
        .flat_map(|project| {
            project
                .help
                .iter()
                .filter(|item| item.status != "Found")
                .map(move |item| (project, item))
        })
        .collect()
}

fn progress(project: &Project) -> String {
    if project.steps.is_empty() {
        return String::new();
    }
    let done = project
        .steps
        .iter()
        .filter(|step| step.status == "Done")
        .count();
    let percentage = done * 100 / project.steps.len();
    let items = project
        .steps
        .iter()
        .map(|step| {
            let marker = if step.status == "Done" { "✓" } else { "" };
            format!(
                r#"<li class="step step--{}"><span class="step-check">{marker}</span><span>{}</span><small>{}</small></li>"#,
                slug(&step.status),
                escape(&step.name),
                escape(&step.status)
            )
        })
        .collect::<String>();
    format!(
        r#"
      <details class="progress">
        <summary>
          <span class="progress-label">Progress</span>
          <span class="progress-track" aria-hidden="true"><i style="width: {percentage}%"></i></span>
          <span>{done} of {} steps</span>
        </summary>
        <ul class="steps">{items}</ul>
      </details>"#,
        project.steps.len()
    )
}

fn project_help(items: &[HelpItem]) -> String {
    let open = items
        .iter()
        .filter(|item| item.status != "Found")
        .collect::<Vec<_>>();
    if open.is_empty() {
        return String::new();
    }
    let roles = open
        .iter()
        .map(|item| {
            let status = if item.status == "Tentative" {
                r#" <span class="need-status">tentative</span>"#.to_string()
            } else {
                String::new()
            };
            format!(
                r#"<span class="need-item">{}{status}</span>"#,
                escape(&item.role)
            )
        })
        .collect::<String>();
    format!(
        r#"
      <div class="project-needs"><span class="needs-label">Needs</span><div>{roles}</div></div>"#
    )
}

fn project_row(project: &Project, position: Option<usize>, context: &str) -> String {
    let rank = position
        .map(|number| format!(r#"<span class="rank">{number}</span>"#))
        .unwrap_or_default();
    let activity = match (context, project.activity.as_str()) {
        ("queue", "Queued") | ("current", "Active") => String::new(),
        _ => pill(&project.activity, &format!("activity-{}", project.activity)),
    };
    let waiting = project
        .waiting_on
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            format!(
                r#"<p class="waiting"><strong>Waiting on:</strong> {}</p>"#,
                escape(value)
            )
        })
        .unwrap_or_default();

    format!(
        r#"
  <article class="project-row project-row--{context}" id="{}">
    {rank}
    <div class="project-content">
      <div class="project-title-line">
        <h3>{}</h3>
        <div class="project-tags">{}{activity}</div>
      </div>
      <div class="project-meta"><span class="focus"><strong>Focus</strong> {}</span></div>
      {waiting}
      {}
      {}
    </div>
  </article>"#,
        slug(&project.title),
        escape(&project.title),
        pill(
            &project.commitment,
            &format!("commitment-{}", project.commitment)
        ),
        escape(&project.focus),
        project_help(&project.help),
        progress(project)
    )
}

fn help_overview(data: &Data) -> String {
    let help = open_help(data);
    let items = help
        .iter()
        .map(|(project, item)| {
            let tentative = if item.status == "Tentative" {
                r#"<span class="overview-status">tentative</span>"#
            } else {
                ""
            };
            format!(
                r##"<li><a href="#{}"><span>{}</span><small>{}</small></a>{tentative}</li>"##,
                slug(&project.title),
                escape(&item.role),
                escape(&project.title)
            )
        })
        .collect::<String>();
    format!(
        r#"
    <aside class="help-overview" aria-labelledby="help-overview-title">
      <div class="panel-heading"><h2 id="help-overview-title">Help wanted</h2></div>
      <ul>{items}</ul>
    </aside>"#
    )
}

fn render_queue(data: &Data) -> String {
    let active = data
        .projects
        .iter()
        .filter(|project| project.activity == "Active")
        .collect::<Vec<_>>();
    let queue = data
        .projects
        .iter()
        .filter(|project| matches!(project.activity.as_str(), "Queued" | "Waiting"))
        .collect::<Vec<_>>();
    let later = data
        .projects
        .iter()
        .filter(|project| matches!(project.activity.as_str(), "Incubating" | "Parked"))
        .collect::<Vec<_>>();

    let active_rows = if active.is_empty() {
        r#"<p class="empty-state">No current project.</p>"#.into()
    } else {
        active
            .iter()
            .map(|project| project_row(project, None, "current"))
            .collect::<String>()
    };
    let queue_rows = if queue.is_empty() {
        r#"<p class="empty-state">The queue is empty.</p>"#.into()
    } else {
        queue
            .iter()
            .enumerate()
            .map(|(index, project)| project_row(project, Some(index + 1), "queue"))
            .collect::<String>()
    };
    let later_rows = if later.is_empty() {
        r#"<p class="empty-state">Nothing is waiting off-queue.</p>"#.into()
    } else {
        later
            .iter()
            .map(|project| project_row(project, None, "later"))
            .collect::<String>()
    };

    format!(
        r#"{}
<body>
  <header class="shell">{}</header>
  <main class="shell main-content">
    <h1 class="sr-only">{}</h1>
    <div class="overview">
      <section class="current-panel" aria-labelledby="current-heading">
        <div class="panel-heading"><h2 id="current-heading">Current focus</h2></div>
        {active_rows}
      </section>
      {}
    </div>

    <section class="project-section" aria-labelledby="queue-heading">
      <div class="section-heading"><h2 id="queue-heading">Up next</h2><span>{} projects</span></div>
      <div class="project-list">{queue_rows}</div>
    </section>

    <section class="project-section project-section--later" aria-labelledby="later-heading">
      <div class="section-heading"><h2 id="later-heading">Later</h2><span>Not in the active queue</span></div>
      <div class="project-list">{later_rows}</div>
    </section>
  </main>
  {}"#,
        head(
            &data.site.title,
            "Current project status, queue, progress, and open help requests.",
            ""
        ),
        nav(data, "queue", ""),
        escape(&data.site.title),
        help_overview(data),
        queue.len(),
        foot()
    )
}

fn render_help(data: &Data) -> String {
    let help = open_help(data);
    let rows = help
        .iter()
        .map(|(project, item)| {
            let kind = item.kind.as_deref().unwrap_or("Help");
            let note = item
                .note
                .as_ref()
                .map(|note| format!(r#"<p class="help-note">{}</p>"#, escape(note)))
                .unwrap_or_default();
            format!(
                r#"
      <article class="help-row">
        <div class="help-row-main">
          <div class="help-title-line"><h2>{}</h2>{}</div>
          <p><a href="../#{}">{}</a><span aria-hidden="true"> · </span>{}<span aria-hidden="true"> · </span>{}</p>
          {note}
        </div>
      </article>"#,
                escape(&item.role),
                pill(&item.status, &format!("help-{}", item.status)),
                slug(&project.title),
                escape(&project.title),
                escape(kind),
                escape(&project.focus)
            )
        })
        .collect::<String>();
    let rows = if rows.is_empty() {
        r#"<p class="empty-state">Nothing is currently open.</p>"#.into()
    } else {
        rows
    };

    format!(
        r#"{}
<body>
  <header class="shell">{}</header>
  <main class="shell main-content help-page">
    <div class="page-heading"><h1>Help wanted</h1><span>{} open items</span></div>
    <div class="help-list">{rows}</div>
  </main>
  {}"#,
        head(
            &format!("Help wanted · {}", data.site.title),
            "Current volunteer roles and project input requests.",
            "../"
        ),
        nav(data, "help", "../"),
        help.len(),
        foot()
    )
}
