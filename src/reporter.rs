use std::{
    collections::{HashMap, HashSet},
    fs,
};

use crate::analyzer;

pub fn run(results: &HashMap<String, analyzer::Report>, output_file: &str) -> std::io::Result<()> {
    // TODO: Add metrics measures %
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
    html.push_str(
        "<tr>
                    <th>File Path</th>
                    <th>Hook Name</th>
                    <th>Is valid custom hook</th>
                    <th><code>use[HookName]</code> prefix</th>
                    <th>Used default hooks</th>
                    <th>Export Method</th>
                </tr>",
    );

    for (file, report) in results {
        let hooks_set: HashSet<_> = report.hooks.iter().cloned().collect();

        let has_valid_hooks = match analyzer::DEFAULT_HOOKS
            .iter()
            .any(|&item| hooks_set.contains(item) && report.export_use_prefix)
        {
            true => "✅",
            false => "❌",
        };

        let starts_with_use_prefix = match report.export_use_prefix {
            true => "✅",
            false => "❌",
        };

        let export_type = match report.export_type {
            analyzer::FileExportType::Function => {
                "Function Declaration <code>function f(…) {…}</code>"
            }
            analyzer::FileExportType::Arrow => {
                "Function Expression <code>const f = () =>{...}</code>"
            }
        };

        html.push_str(&format!(
            "<tr>
            <td>{}</td>
            <td><code>{}</code></td>
            <td>{}</td>
            <td>{}</td>
            <td><code>{}</code></td>
            <td>{}</td>
            </tr>",
            file,
            report.hook_name,
            has_valid_hooks,
            starts_with_use_prefix,
            report.hooks.join(", "),
            export_type
        ));
    }

    html.push_str("</table>");
    html.push_str("</body></html>");

    fs::write(output_file, html)
}
