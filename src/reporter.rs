use std::{
    collections::{HashMap, HashSet},
    fs,
};

use crate::analyzer;

pub fn run(results: &HashMap<String, Vec<String>>, output_file: &str) -> std::io::Result<()> {
    let mut html = String::new();
    html.push_str("<html><head><title>React Hooks Analyzer Report</title>");
    html.push_str("<style>");
    html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }");
    html.push_str("h1 { color: #333; }");
    html.push_str("table { width: 100%; border-collapse: collapse; margin-top: 20px; }");
    html.push_str("th, td { border: 1px solid #ccc; padding: 8px; text-align: left; }");
    html.push_str("th { background-color: #f4f4f4; }");
    html.push_str("tr:nth-child(even) { background-color: #f9f9f9; }");
    html.push_str("</style>");
    html.push_str("</head><body>");
    html.push_str("<h1>Custom React Hooks Analyzer Report</h1>");
    html.push_str(&format!("<p>Files visited: {}</p>", results.len()));

    html.push_str("<table>");
    html.push_str("<tr><th>File</th><th>Is valid custom hook</th><th>Used default hooks</th></tr>");

    for (file, hooks) in results {
        let hooks_set: HashSet<_> = hooks.iter().cloned().collect();

        let has_valid_hooks = match analyzer::DEFAULT_HOOKS
            .iter()
            .any(|&item| hooks_set.contains(item))
        {
            true => "✅",
            false => "❌",
        };

        html.push_str(&format!(
            "<tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            </tr>",
            file,
            has_valid_hooks,
            hooks.join(", ")
        ));
    }

    html.push_str("</table>");
    html.push_str("</body></html>");

    fs::write(output_file, html)
}
