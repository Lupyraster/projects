use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};

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
    #[serde(default)]
    summary: Option<String>,
    activity: String,
    #[serde(default)]
    commitment: Option<String>,
    #[serde(default)]
    focus: Option<String>,
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
    summary: Option<String>,
    note: Option<String>,
}

struct Options {
    serve: bool,
    port: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(options) = parse_options()? else {
        return Ok(());
    };
    build_site()?;
    if options.serve {
        serve(Path::new("dist"), options.port)?;
    }
    Ok(())
}

fn parse_options() -> Result<Option<Options>, String> {
    let mut serve = false;
    let mut port = 8000;
    let mut arguments = std::env::args().skip(1);

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--serve" => serve = true,
            "--port" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--port requires a number".to_string())?;
                port = value
                    .parse::<u16>()
                    .map_err(|_| format!("invalid port: {value}"))?;
            }
            "--help" | "-h" => {
                print_help();
                return Ok(None);
            }
            _ => {
                return Err(format!(
                    "unknown option: {argument}\nRun with --help for usage."
                ));
            }
        }
    }

    if !serve && port != 8000 {
        return Err("--port can only be used with --serve".into());
    }
    Ok(Some(Options { serve, port }))
}

fn print_help() {
    println!(
        "Project Queue generator\n\n\
         Usage:\n  cargo run [-- OPTIONS]\n\n\
         Options:\n  --serve        Build the site, then serve it locally\n  --port PORT    Port for --serve (default: 8000)\n  -h, --help     Show this help\n\n\
         Examples:\n  cargo run\n  cargo run -- --serve\n  cargo run -- --serve --port 8080"
    );
}

fn build_site() -> Result<(), Box<dyn std::error::Error>> {
    let raw = fs::read_to_string("projects.yaml")?;
    let data: Data = serde_yaml::from_str(&raw)?;
    validate(&data)?;

    let output = Path::new("dist");
    if output.exists() {
        fs::remove_dir_all(output)?;
    }
    fs::create_dir_all(output.join("assets"))?;

    fs::write(output.join("index.html"), render_queue(&data))?;
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

fn serve(root: &Path, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let address = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&address)?;
    println!("Serving at http://{}/", listener.local_addr()?);
    println!("Press Ctrl+C to stop.");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(error) = serve_request(&mut stream, root) {
                    eprintln!("Request failed: {error}");
                }
            }
            Err(error) => eprintln!("Connection failed: {error}"),
        }
    }
    Ok(())
}

fn serve_request(stream: &mut TcpStream, root: &Path) -> std::io::Result<()> {
    let mut buffer = [0_u8; 8192];
    let bytes_read = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let mut parts = request
        .lines()
        .next()
        .unwrap_or_default()
        .split_whitespace();
    let method = parts.next().unwrap_or_default();
    let request_path = parts.next().unwrap_or("/").split('?').next().unwrap_or("/");

    if !matches!(method, "GET" | "HEAD") {
        return send_response(
            stream,
            method,
            405,
            "text/plain; charset=utf-8",
            b"Method not allowed",
        );
    }

    let relative = if request_path == "/" {
        PathBuf::from("index.html")
    } else {
        PathBuf::from(request_path.trim_start_matches('/'))
    };
    let safe = relative
        .components()
        .all(|component| matches!(component, Component::Normal(_)));
    if !safe {
        return send_response(
            stream,
            method,
            404,
            "text/plain; charset=utf-8",
            b"Not found",
        );
    }

    let file = root.join(&relative);
    match fs::read(&file) {
        Ok(body) => send_response(stream, method, 200, content_type(&file), &body),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => send_response(
            stream,
            method,
            404,
            "text/plain; charset=utf-8",
            b"Not found",
        ),
        Err(error) => Err(error),
    }
}

fn send_response(
    stream: &mut TcpStream,
    method: &str,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Error",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    if method != "HEAD" {
        stream.write_all(body)?;
    }
    stream.flush()
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
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
        if let Some(commitment) = project
            .commitment
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            if !commitments.contains(commitment) {
                return Err(format!(
                    "{}: unknown commitment '{}'",
                    project.title, commitment
                ));
            }
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

fn nav(data: &Data) -> String {
    format!(
        r#"
  <div class="site-nav">
    <a class="brand" href="./">{}</a>
    <span class="updated">Updated {}</span>
  </div>"#,
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

fn help_summary(item: &HelpItem) -> Option<&str> {
    item.summary
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            item.note
                .as_deref()
                .filter(|value| !value.trim().is_empty())
        })
}

fn help_detail(project: &Project, item: &HelpItem, show_focus: bool) -> String {
    let summary = help_summary(item)
        .map(|value| format!(r#"<p>{}</p>"#, escape(value)))
        .unwrap_or_default();
    let kind = item
        .kind
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!(r#"<div><dt>Type</dt><dd>{}</dd></div>"#, escape(value)))
        .unwrap_or_default();
    let focus = if show_focus {
        project
            .focus
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| format!(r#"<div><dt>Focus</dt><dd>{}</dd></div>"#, escape(value)))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let metadata = if kind.is_empty() && focus.is_empty() {
        String::new()
    } else {
        format!(r#"<dl>{kind}{focus}</dl>"#)
    };
    format!(r#"<div class="help-detail">{summary}{metadata}</div>"#)
}

fn project_help(project: &Project) -> String {
    let items = &project.help;
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
            format!(
                r#"
          <details class="project-need">
            <summary>
              <span class="help-chevron" aria-hidden="true"></span>
              <span class="project-need-role">{}</span>
              <span class="help-status help-status--{}">{}</span>
            </summary>
            {}
          </details>"#,
                escape(&item.role),
                slug(&item.status),
                escape(&item.status),
                help_detail(project, item, false)
            )
        })
        .collect::<String>();
    format!(
        r#"
      <div class="project-needs"><span class="needs-label">Needs</span><div class="project-need-list">{roles}</div></div>"#
    )
}

fn project_row(project: &Project, context: &str) -> String {
    let activity = match (context, project.activity.as_str()) {
        ("queue", "Queued") | ("current", "Active") => String::new(),
        _ => pill(&project.activity, &format!("activity-{}", project.activity)),
    };
    let commitment = project
        .commitment
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| match (context, value) {
            ("current" | "queue", "Committed") => String::new(),
            _ => pill(value, &format!("commitment-{value}")),
        })
        .unwrap_or_default();
    let tags = if commitment.is_empty() && activity.is_empty() {
        String::new()
    } else {
        format!(r#"<div class="project-tags">{commitment}{activity}</div>"#)
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
    let focus = project
        .focus
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            format!(
                r#"<div class="project-meta"><span class="focus"><strong>Focus</strong> {}</span></div>"#,
                escape(value)
            )
        })
        .unwrap_or_default();
    let summary = project
        .summary
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!(r#"<p class="project-summary">{}</p>"#, escape(value)))
        .unwrap_or_default();

    format!(
        r#"
  <article class="project-row project-row--{context}" id="{}">
    <div class="project-content">
      <div class="project-title-line">
        <h3>{}</h3>
        {tags}
      </div>
      {summary}
      {focus}
      {waiting}
      {}
      {}
    </div>
  </article>"#,
        slug(&project.title),
        escape(&project.title),
        project_help(project),
        progress(project)
    )
}

fn help_overview(data: &Data) -> String {
    let help = open_help(data);
    let items = help
        .iter()
        .map(|(project, item)| {
            format!(
                r#"
      <details class="help-item">
        <summary>
          <span class="help-chevron" aria-hidden="true"></span>
          <span class="help-role">{}</span>
          <span class="help-project">{}</span>
          <span class="help-status help-status--{}">{}</span>
        </summary>
        {}
      </details>"#,
                escape(&item.role),
                escape(&project.title),
                slug(&item.status),
                escape(&item.status),
                help_detail(project, item, true)
            )
        })
        .collect::<String>();
    format!(
        r#"
    <aside class="help-overview" aria-labelledby="help-overview-title">
      <div class="panel-heading"><h2 id="help-overview-title">Help Wanted</h2></div>
      <div class="help-items">{items}</div>
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
            .map(|project| project_row(project, "current"))
            .collect::<String>()
    };
    let queue_rows = if queue.is_empty() {
        r#"<p class="empty-state">The queue is empty.</p>"#.into()
    } else {
        queue
            .iter()
            .map(|project| project_row(project, "queue"))
            .collect::<String>()
    };
    let later_rows = if later.is_empty() {
        r#"<p class="empty-state">Nothing is waiting off-queue.</p>"#.into()
    } else {
        later
            .iter()
            .map(|project| project_row(project, "later"))
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
        <div class="panel-heading"><h2 id="current-heading">Current Focus</h2></div>
        {active_rows}
      </section>
      {}
    </div>

    <section class="project-section" aria-labelledby="queue-heading">
      <div class="section-heading"><h2 id="queue-heading">Up Next</h2></div>
      <div class="project-list">{queue_rows}</div>
    </section>

    <section class="project-section project-section--later" aria-labelledby="later-heading">
      <div class="section-heading"><h2 id="later-heading">Later</h2></div>
      <div class="project-list">{later_rows}</div>
    </section>
  </main>
  {}"#,
        head(
            &data.site.title,
            "Current project status, queue, progress, and open help requests.",
            ""
        ),
        nav(data),
        escape(&data.site.title),
        help_overview(data),
        foot()
    )
}
