use std::path::Path;

use research_domain::session::Sessioned;
use research_domain::task::ResearchRun;

use super::citations;
use super::data;
use super::sources;
use super::text;

/// Render task section HTML.
pub fn taskhtml(root: &Path, session: &dyn Sessioned, task: &ResearchRun) -> (String, Vec<String>) {
    let (content, task_sources) = data::resultmap(root, session, task);
    let content = text::clean(&content);
    let content = text::underscorify(&content);
    let content = citations::stars(&content);
    let content = sources::emojify(&content);
    let cite = citations::citations(&content, &task_sources);
    let content = cite.text;
    let urls = cite.urls;
    let mark = cite.marks;
    let content = text::tablecite(&content);
    let content = text::tablelead(&content);
    let content = text::tablepipe(&content);
    let content = text::tablerows(&content);
    let content = citations::strip(&content);
    let content = text::nested(&content);
    let content = text::normalize(&content);
    let content = text::rule(&content);
    let html = if content.trim().is_empty() {
        String::new()
    } else {
        text::markdown(&content)
    };
    let html = citations::tables(&html);
    let html = citations::codeindent(&html);
    let html = text::paragraphs(&html);
    let html = citations::backslash(&html);
    let html = mark
        .iter()
        .fold(html, |note, (token, link)| note.replace(token, link));
    let body = if html.trim().is_empty() {
        String::new()
    } else {
        format!("<div class=\"synthesis\">{}</div>", html)
    };
    (
        format!("<section>{}<div class=\"divider\"></div></section>", body),
        urls,
    )
}

/// Render all tasks into HTML sections.
pub fn tasks(root: &Path, session: &dyn Sessioned) -> (String, Vec<String>) {
    let list = session.tasks();
    let mut content = String::new();
    let mut all_urls: Vec<String> = Vec::new();
    for task in list {
        let (html, urls) = taskhtml(root, session, task);
        content.push_str(&html);
        for url in urls {
            if !all_urls.contains(&url) {
                all_urls.push(url);
            }
        }
    }
    (content, all_urls)
}
