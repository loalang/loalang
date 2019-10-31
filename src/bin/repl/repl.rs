use crate::*;
use loa::server::Server;
use loa::*;
use rustyline::completion::{Candidate, Completer};
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::{CompletionType, Context, EditMode, Editor, Helper};

struct EditorHelper {
    pub server: Arc<Mutex<Server>>,
    pub uri: URI,
}

impl Helper for EditorHelper {}

impl Hinter for EditorHelper {}

#[derive(Debug)]
struct CompletionCandidate {
    pub display: String,
    pub replacement: String,
}

impl Candidate for CompletionCandidate {
    fn display(&self) -> &str {
        self.display.as_ref()
    }

    fn replacement(&self) -> &str {
        self.replacement.as_ref()
    }
}

impl Completer for EditorHelper {
    type Candidate = CompletionCandidate;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        let mut server = self.server.lock().unwrap();

        server.set(self.uri.clone(), line.into(), SourceKind::REPLLine);

        if let Some(location) = server.location(&self.uri, (1, pos + 1)) {
            if let Some(completion) = server.completion(location, String::new()) {
                let candidates = match completion {
                    server::Completion::VariablesInScope(prefix, vars) => vars
                        .into_iter()
                        .map(|v| CompletionCandidate {
                            display: format!("{} ({})", v.name, v.type_),
                            replacement: format!("{} ", &v.name[prefix.len()..]),
                        })
                        .collect(),

                    server::Completion::Behaviours(prefix, behaviours) => behaviours
                        .into_iter()
                        .map(|behaviour| CompletionCandidate {
                            display: behaviour.to_string(),
                            replacement: format!("{} ", &behaviour.selector()[prefix.len()..]),
                        })
                        .collect(),
                };
                return Ok((pos, candidates));
            }
        }
        Ok((0, vec![]))
    }
}

impl Highlighter for EditorHelper {}

pub struct REPL {
    editor: Editor<EditorHelper>,
    server: Arc<Mutex<Server>>,
}

impl REPL {
    pub fn new() -> REPL {
        let server = Arc::new(Mutex::new(Server::new()));

        let mut editor = Editor::new();
        editor.set_edit_mode(EditMode::Vi);
        editor.set_completion_type(CompletionType::List);
        editor.set_helper(Some(EditorHelper {
            server: server.clone(),
            uri: URI::REPLLine(0),
        }));

        REPL { editor, server }
    }

    pub fn start(&mut self) {
        let mut n = 1;
        let mut line = String::new();
        loop {
            let uri = loa::URI::REPLLine(n);
            self.editor.helper_mut().unwrap().uri = uri.clone();
            let addition = match self
                .editor
                .readline(if line.len() == 0 { ">>> " } else { "... " })
            {
                Ok(ref s) if s == "" => continue,
                Ok(line) => line,
                Err(_) => break,
            };
            line.push_str(addition.as_str());

            let mut server = self.server.lock().unwrap();

            server.set(uri.clone(), line.clone(), loa::SourceKind::REPLLine);

            if server.ends_with_syntax_error(&uri) {
                line.push('\n');
                server.remove(uri);
                continue;
            }
            let line = std::mem::replace(&mut line, String::new());
            info!("{}", line);

            self.editor.add_history_entry(&line);
            n += 1;

            let mut failure = false;
            for (d_uri, diagnostics) in server.diagnostics() {
                if d_uri == uri {
                    for d in diagnostics {
                        if let DiagnosticLevel::Error = d.level() {
                            failure = true;
                        }
                        println!("{:?}", d);
                    }
                }
            }
            if failure {
                server.remove(uri);
            }
        }
    }
}
