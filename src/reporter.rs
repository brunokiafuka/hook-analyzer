use std::{
    collections::{HashMap, HashSet},
    fs,
};

use crate::analyzer;

pub fn run(results: &HashMap<String, analyzer::Report>, output_file: &str) -> std::io::Result<()> {
    // TODO: Add metrics measures %
    let mut html = format!(
    r#"<!DOCTYPE html>
    <html>
        <head>
            <title>React Hooks Manuel Report</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 20px; }}
                h1 {{ color: #333; }}
                table {{ width: 100%; border-collapse: collapse; margin-top: 20px; }}
                th, td {{ border: 1px solid #ccc; padding: 8px; text-align: left; }}
                th {{ background-color: #f4f4f4; }}
                tr:nth-child(even) {{ background-color: #f9f9f9; }}
            </style>
        </head>
    <body>
        <h1>Custom React Hooks Analyzer Report</h1>
        <p>Files visited: {}</p>
        <table>
            <thead>
                <tr>
                    <th>File Path</th>
                    <th>Hook Name</th>
                    <th>Is valid custom hook</th>
                    <th><code>use[HookName]</code> prefix</th>
                    <th>Used default hooks</th>
                    <th>Export Method</th>
                </tr>
            </thead>
            <tbody>"#,results.len()
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

        html.push_str(&format!("
                <tr>
                    <td>{}</td>
                    <td><code>{}</code></td>
                    <td>{}</td>
                    <td>{}</td>
                    <td><code>{}</code></td>
                    <td>{}</td>
                </tr>
        ",
            file,
            report.hook_name,
            has_valid_hooks,
            starts_with_use_prefix,
            report.hooks.join(", "),
            export_type
        ));
    }

    html.push_str("
            </tbody>
        </table>
    </body>
    </html>
    ");

    fs::write(output_file, html)
}
