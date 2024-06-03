use std::path::Path;
use std::{env, fs};
use swc_common::{sync::Lrc, SourceFile, SourceMap};
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};
use swc_ecma_visit::{Visit, VisitWith};

struct Analyzer {
    filename: String,
}

struct TreeVisitor {
    filename: String,
}

impl Analyzer {
    fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
        }
    }

    fn run(&self, module: &Module) {
        let mut visitor = TreeVisitor::new(&self.filename);

        module.visit_with(&mut visitor);
    }
}

impl TreeVisitor {
    fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
        }
    }
}

impl Visit for TreeVisitor {
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
                    println!(
                        "Default hook \"{:?}\" found in file \"{}\" \n",
                        obj_id.sym, self.filename
                    );
                } else {
                    println!("No hook in file {}", self.filename);
                }
            } else if let Expr::Member(member_expr) = &**expr {
                // check for React.use*
                if let Expr::Ident(_obj_id) = &*member_expr.obj {
                    if let MemberProp::Ident(prop_id) = &member_expr.prop {
                        if default_hooks.contains(&prop_id.sym.as_ref()) {
                            println!(
                                "Default hook \"{:?}\" found in file \"{}\"",
                                prop_id.sym, self.filename
                            );
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

fn analyze_directory(directory_file: &Path) {
    for entry in fs::read_dir(directory_file).expect("") {
        let entry = entry.expect("");
        let path = entry.path();

        if path.is_dir() {
            if path.ends_with("hooks") {
                analyze_hook_dir(&path)
            } else {
                analyze_directory(&path);
            }
        }
    }
}

fn analyze_hook_dir(directory_path: &Path) {
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

                        analyzer.run(&module);
                    }
                    Err(err) => {
                        eprintln!("Error parsing file {}: {:?}", path.display(), err);
                    }
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let dir = &args[1];
    println!("{:?}", dir);

    let src_directory = Path::new(dir);
    analyze_directory(src_directory);
}
