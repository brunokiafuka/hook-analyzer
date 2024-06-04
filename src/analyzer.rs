use std::path::Path;
use std::{collections::HashMap, fs};
use swc_common::{sync::Lrc, SourceFile, SourceMap};
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};
use swc_ecma_visit::{Visit, VisitWith};

pub const DEFAULT_HOOKS: [&str; 14] = [
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
    "useSyncExternalStore",
];

struct Analyzer {
    filename: String,
}

#[derive(Debug)]
pub struct Report {
    pub hooks: Vec<String>,
    pub export_use_prefix: bool,
}

struct TreeVisitor {
    filename: String,
    pub report: HashMap<String, Report>,
}

impl Analyzer {
    fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
        }
    }

    fn run(&self, module: &Module) -> HashMap<String, Report> {
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

        let empty_vec = Report {
            hooks: vec![],
            export_use_prefix: false,
        };

        let previous_result = match self.report.get(&key) {
            Some(x) => x,
            None => &empty_vec,
        };

        let copy_previous_result = Report {
            hooks: {
                let mut hooks = previous_result.hooks.clone();
                hooks.push(value);
                hooks
            },
            export_use_prefix: { previous_result.export_use_prefix },
        };

        self.report.insert(key, copy_previous_result);
    }
}

impl Visit for TreeVisitor {
    fn visit_call_expr(&mut self, call_expr: &CallExpr) {
        if let Callee::Expr(expr) = &call_expr.callee {
            // check directly for use* without React.* object reference
            if let Expr::Ident(obj_id) = &**expr {
                if DEFAULT_HOOKS.contains(&obj_id.sym.as_ref()) {
                    self.arrange_report(obj_id.sym.as_ref().to_string());
                } else {
                    println!("No hook in file {}", self.filename);
                }
            } else if let Expr::Member(member_expr) = &**expr {
                // check for React.use*
                if let Expr::Ident(_obj_id) = &*member_expr.obj {
                    if let MemberProp::Ident(prop_id) = &member_expr.prop {
                        if DEFAULT_HOOKS.contains(&prop_id.sym.as_ref()) {
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

    fn visit_export_decl(&mut self, exp_decl: &ExportDecl) {
        if let Decl::Fn(fn_dec) = &exp_decl.decl {
            let key = self.filename.to_string();

            let empty_vec = Report {
                hooks: vec![],
                export_use_prefix: false,
            };

            let previous_result = match self.report.get(&key) {
                Some(x) => x,
                None => &empty_vec,
            };
            let copy_previous_result = Report {
                hooks: previous_result.hooks.clone(),
                export_use_prefix: {
                    match fn_dec.ident.sym.to_string().starts_with("use") {
                        true => true,
                        false => false,
                    }
                },
            };

            self.report.insert(key, copy_previous_result);
        }

        exp_decl.visit_children_with(self);
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
pub fn read_directory(directory_file: &Path, results: &mut HashMap<String, Report>) {
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
fn analyze_hook_dir(directory_path: &Path, results: &mut HashMap<String, Report>) {
    for entry in fs::read_dir(directory_path).expect("Failed to read hooks directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_file() {
            //  let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            // previous code was checking for (filename.starts_with("use"))
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if ext == "js" || ext == "jsx" || ext == "ts" || ext == "tsx" {
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
