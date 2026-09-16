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
    introduction: String,
}

#[derive(Deserialize)]
struct Project {
    title: String,
    activity: String,
    commitment: String,
    focus: String,
    #[serde(default)]
    summary: String,
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
    let help_count = data
        .projects
        .iter()
        .flat_map(|project| &project.help)
        .filter(|item| item.status != "Found")
        .count();
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

fn icon(name: &str) -> String {
    let paths = match name {
        "arrow" => r#"<path d="M5 12h14M13 6l6 6-6 6"/>"#,
        "check" => r#"<path d="m5 12 4 4L19 6"/>"#,
        "clock" => r#"<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>"#,
        "people" => {
            r#"<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"/>"#
        }
        _ => "",
    };
    format!(r#"<svg class="icon" viewBox="0 0 24 24" aria-hidden="true">{paths}</svg>"#)
}

fn pill(text: &str, kind: &str) -> String {
    format!(
        r#"<span class="pill pill--{}">{}</span>"#,
        slug(kind),
        escape(text)
    )
}

fn nav(active: &str, prefix: &str) -> String {
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
    <a class="brand" href="{home}" aria-label="Project Queue home">
      <span class="brand-mark" aria-hidden="true"><i></i><i></i><i></i></span>
      <span>Project Queue</span>
    </a>
    <div class="nav-links">
      <a href="{home}"{queue_current}>Queue</a>
      <a href="{prefix}help/"{help_current}>Help needed</a>
    </div>
  </nav>"#
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
  <meta name="theme-color" content="#783f3b">
  <title>{}</title>
  <link rel="icon" href="{asset_prefix}assets/favicon.svg" type="image/svg+xml">
  <link rel="stylesheet" href="{asset_prefix}assets/style.css">
</head>"##,
        escape(description),
        escape(title)
    )
}

fn foot(data: &Data, asset_prefix: &str) -> String {
    let home = if asset_prefix.is_empty() {
        "./"
    } else {
        asset_prefix
    };
    format!(
        r#"
  <footer class="site-footer">
    <p>Last updated {}</p>
    <a href="{home}">Return to the queue {}</a>
  </footer>
</body>
</html>"#,
        escape(&data.site.updated),
        icon("arrow")
    )
}

fn step_list(steps: &[Step]) -> String {
    if steps.is_empty() {
        return String::new();
    }
    let done = steps.iter().filter(|step| step.status == "Done").count();
    let rows = steps
        .iter()
        .map(|step| {
            let mark = if step.status == "Done" {
                icon("check")
            } else {
                String::new()
            };
            format!(
                r#"<li class="step step--{}">
              <span class="step-mark">{mark}</span>
              <span>{}</span>
              <span class="step-status">{}</span>
            </li>"#,
                slug(&step.status),
                escape(&step.name),
                escape(&step.status)
            )
        })
        .collect::<String>();
    format!(
        r#"
    <details class="project-details">
      <summary>
        <span>Progress</span>
        <span class="summary-count">{done} of {} complete</span>
      </summary>
      <ul class="steps">{rows}</ul>
    </details>"#,
        steps.len()
    )
}

fn compact_help(items: &[HelpItem]) -> String {
    let rows = items
        .iter()
        .filter(|item| item.status != "Found")
        .map(|item| {
            format!(
                "<span>{} {}</span>",
                escape(&item.role),
                pill(&item.status, &format!("help-{}", item.status))
            )
        })
        .collect::<String>();
    if rows.is_empty() {
        String::new()
    } else {
        format!(
            r#"
    <div class="card-help">
      <div class="card-help__label">{} Help</div>
      <div class="card-help__items">{rows}</div>
    </div>"#,
            icon("people")
        )
    }
}

fn project_card(project: &Project, number: &str, featured: bool) -> String {
    let featured_class = if featured {
        " project-card--featured"
    } else {
        ""
    };
    let number_text = if featured { "NOW" } else { number };
    let number_label = if featured {
        "Current focus".to_string()
    } else {
        format!("Queue position {number}")
    };
    let summary = if project.summary.is_empty() {
        String::new()
    } else {
        format!(
            r#"<p class="project-summary">{}</p>"#,
            escape(&project.summary)
        )
    };
    let waiting = project
        .waiting_on
        .as_ref()
        .map(|value| {
            format!(
                r#"<p class="waiting"><strong>Waiting on:</strong> {}</p>"#,
                escape(value)
            )
        })
        .unwrap_or_default();

    format!(
        r#"
  <article class="project-card{featured_class}">
    <div class="queue-number" aria-label="{}">{number_text}</div>
    <div class="project-main">
      <div class="project-heading">
        <div>
          <div class="project-pills">
            {}
            {}
          </div>
          <h2>{}</h2>
        </div>
        <div class="focus-time">{}<span><small>Expected focus</small>{}</span></div>
      </div>
      {summary}
      {waiting}
      {}
      {}
    </div>
  </article>"#,
        escape(&number_label),
        pill(&project.activity, &format!("activity-{}", project.activity)),
        pill(
            &project.commitment,
            &format!("commitment-{}", project.commitment)
        ),
        escape(&project.title),
        icon("clock"),
        escape(&project.focus),
        compact_help(&project.help),
        step_list(&project.steps)
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

    let active_cards = if active.is_empty() {
        r#"<p class="empty-state">No project is marked active right now.</p>"#.into()
    } else {
        active
            .iter()
            .map(|project| project_card(project, "0", true))
            .collect::<String>()
    };
    let queue_cards = if queue.is_empty() {
        r#"<p class="empty-state">The queue is empty.</p>"#.into()
    } else {
        queue
            .iter()
            .enumerate()
            .map(|(index, project)| project_card(project, &format!("{:02}", index + 1), false))
            .collect::<String>()
    };
    let later_cards = if later.is_empty() {
        r#"<p class="empty-state">Nothing is waiting off-queue.</p>"#.into()
    } else {
        later
            .iter()
            .map(|project| project_card(project, "—", false))
            .collect::<String>()
    };

    format!(
        r#"{}
<body>
  <header class="shell">{}</header>
  <main class="shell">
    <section class="page-intro" aria-labelledby="queue-title">
      <p class="eyebrow">Public project status</p>
      <h1 id="queue-title">What I’m working on,<br><em>and what comes next.</em></h1>
      <p>{}</p>
      <a class="help-callout" href="help/">
        {}
        <span><strong>Want to help?</strong> See the roles and project input I currently need.</span>
        {}
      </a>
    </section>

    <section class="queue-section" aria-labelledby="current-heading">
      <div class="section-heading"><span>01</span><h2 id="current-heading">Current focus</h2></div>
      <div class="project-list">{active_cards}</div>
    </section>

    <section class="queue-section" aria-labelledby="next-heading">
      <div class="section-heading"><span>02</span><h2 id="next-heading">Up next</h2><p>Priority order when runnable</p></div>
      <div class="project-list project-list--numbered">{queue_cards}</div>
    </section>

    <section class="queue-section queue-section--later" aria-labelledby="later-heading">
      <div class="section-heading"><span>03</span><h2 id="later-heading">Incubating &amp; later</h2><p>Useful work may happen, but these have no firm queue position</p></div>
      <div class="project-list project-list--later">{later_cards}</div>
    </section>
  </main>
  {}"#,
        head(&data.site.title, &data.site.introduction, ""),
        nav("queue", ""),
        escape(&data.site.introduction),
        icon("people"),
        icon("arrow"),
        foot(data, "")
    )
}

fn render_help(data: &Data) -> String {
    let cards = data
        .projects
        .iter()
        .flat_map(|project| {
            project
                .help
                .iter()
                .filter(|item| item.status != "Found")
                .map(move |item| (project, item))
        })
        .map(|(project, item)| {
            let kind = item.kind.as_deref().unwrap_or("Help");
            let note = item
                .note
                .as_ref()
                .map(|note| format!("<p>{}</p>", escape(note)))
                .unwrap_or_default();
            format!(
                r#"
          <article class="help-card">
            <div class="help-card__top">
              {}
              {}
            </div>
            <h2>{}</h2>
            <p class="help-project">For <strong>{}</strong></p>
            {note}
            <div class="help-card__meta">
              {}
              <span>{}</span>
            </div>
          </article>"#,
                pill(kind, "help-kind"),
                pill(&item.status, &format!("help-{}", item.status)),
                escape(&item.role),
                escape(&project.title),
                pill(&project.activity, &format!("activity-{}", project.activity)),
                escape(&project.focus)
            )
        })
        .collect::<String>();
    let cards = if cards.is_empty() {
        r#"<p class="empty-state">Nothing is currently open.</p>"#.into()
    } else {
        cards
    };

    format!(
        r#"{}
<body>
  <header class="shell">{}</header>
  <main class="shell">
    <section class="page-intro page-intro--help" aria-labelledby="help-title">
      <p class="eyebrow">Open roles &amp; requests</p>
      <h1 id="help-title">Ways to help.</h1>
      <p>These projects need a person, a test, or information. <strong>Needed</strong> items are open; <strong>Tentative</strong> items have a possible helper or may still need confirmation.</p>
    </section>

    <section class="help-grid" aria-label="Open help requests">{cards}</section>

    <aside class="legend">
      <h2>What the labels mean</h2>
      <dl>
        <div><dt>{}</dt><dd>The role or input is currently open.</dd></div>
        <div><dt>{}</dt><dd>Someone may be available, or I still need to confirm the details.</dd></div>
      </dl>
    </aside>
  </main>
  {}"#,
        head(
            &format!("Help needed · {}", data.site.title),
            "Open volunteer roles and requested project input.",
            "../"
        ),
        nav("help", "../"),
        pill("Needed", "help-Needed"),
        pill("Tentative", "help-Tentative"),
        foot(data, "../")
    )
}
