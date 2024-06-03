use std::path::Path;
use std::{collections::HashMap, fs};
use swc_common::{sync::Lrc, SourceFile, SourceMap};
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};
use swc_ecma_visit::{Visit, VisitWith};

struct Analyzer {
    filename: String,
}

struct TreeVisitor {
    filename: String,
    pub report: HashMap<String, Vec<String>>,
}

impl Analyzer {
    fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
        }
    }

    fn run(&self, module: &Module) -> HashMap<String, Vec<String>> {
        let mut visitor = TreeVisitor::new(&self.filename);

        module.visit_with(&mut visitor);

        // this returns the report after running the `visit_call_expr`
        visitor.report
    }
}

impl TreeVisitor {
    fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
            report: HashMap::new(),
        }
    }

    fn arrange_report(&mut self, value: String) {
        let key = self.filename.to_string();

        let empty_vec = vec![];

        let previous_result = match self.report.get(&key) {
            Some(x) => x,
            None => &empty_vec,
        };

        let mut copy_previous_result = previous_result.clone();
        copy_previous_result.push(value);

        self.report.insert(key, copy_previous_result);
    }
}

impl Visit for TreeVisitor {
    // TODO: check non caller cases and handle empty states
    fn visit_call_expr(&mut self, call_expr: &CallExpr) {
        let default_hooks = [
            "useState",
            "useEffect",
            "useContext",
            "useReducer",
            "useCallback",
            "useMemo",
            "useRef",
            "useImperativeHandle",
            "useLayoutEffect",
            "useDebugValue",
            "useTransition",
            "useDeferredValue",
            "useId",
        ];

        if let Callee::Expr(expr) = &call_expr.callee {
            // check directly for use* without React.* object reference
            if let Expr::Ident(obj_id) = &**expr {
                if default_hooks.contains(&obj_id.sym.as_ref()) {
                    self.arrange_report(obj_id.sym.as_ref().to_string());
                } else {
                    println!("No hook in file {}", self.filename);
                }
            } else if let Expr::Member(member_expr) = &**expr {
                // check for React.use*
                if let Expr::Ident(_obj_id) = &*member_expr.obj {
                    if let MemberProp::Ident(prop_id) = &member_expr.prop {
                        if default_hooks.contains(&prop_id.sym.as_ref()) {
                            self.arrange_report(prop_id.sym.as_ref().to_string());
                        } else {
                            println!("No hook in file {}", self.filename);
                        }
                    }
                }
            }
        }

        call_expr.visit_children_with(self);
    }
}

fn parse_file(filename: &str) -> Result<Module, swc_ecma_parser::error::Error> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm
        .load_file(Path::new(filename))
        .expect("failed to load file");

    let source_file: &SourceFile = &*fm;

    let lexer = Lexer::new(
        Syntax::Typescript(TsConfig {
            tsx: true,
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(source_file),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser.parse_module();

    match module {
        Ok(m) => Ok(m),
        Err(e) => {
            eprintln!("Error parsing file {}: {:?}", filename, e);
            Err(e)
        }
    }
}

/**
 * This is the entry point function that is responsible for checking the folders and call the analyzer functions
 */
pub fn read_directory(directory_file: &Path, results: &mut HashMap<String, Vec<String>>) {
    for entry in fs::read_dir(directory_file).expect("") {
        let entry = entry.expect("");
        let path = entry.path();

        if path.is_dir() {
            if path.ends_with("hooks") {
                analyze_hook_dir(&path, results)
            } else {
                read_directory(&path, results);
            }
        }
    }
}

/**
 * This function runs the analyzer in the hooks folder
 */
fn analyze_hook_dir(directory_path: &Path, results: &mut HashMap<String, Vec<String>>) {
    for entry in fs::read_dir(directory_path).expect("Failed to read hooks directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_file() {
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if (filename.starts_with("use"))
                && (ext == "js" || ext == "jsx" || ext == "ts" || ext == "tsx")
            {
                match parse_file(path.to_str().unwrap()) {
                    Ok(module) => {
                        let analyzer = Analyzer::new(path.to_str().unwrap());

                        let analysis_results = analyzer.run(&module);

                        // push to results
                        results.extend(analysis_results);
                    }
                    Err(err) => {
                        eprintln!("Error parsing file {}: {:?}", path.display(), err);
                    }
                }
            }
        }
    }
}
