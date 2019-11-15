use crate::*;
use colored::{Color, Colorize};
use loa::generation::{Generator, REPLDirectives};
use loa::semantics::Type;
use loa::server::Server;
use loa::syntax::{characters_to_string, string_to_characters, tokenize, TokenKind};
use loa::vm::VM;
use loa::*;
use rustyline::completion::{Candidate, Completer};
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::{CompletionType, Context, EditMode, Editor, Helper};
use std::borrow::Cow::{Borrowed, Owned};

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
        let chars_before = string_to_characters(line[..pos].into());
        let realpos = chars_before.len();

        let mut server = self.server.lock().unwrap();

        server.set(self.uri.clone(), line.into(), SourceKind::REPLLine);

        if let Some(location) = server.location(&self.uri, (1, realpos + 1)) {
            if let Some(completion) =
                server.completion(location, characters_to_string(chars_before.into_iter()))
            {
                let candidates = match completion {
                    server::Completion::VariablesInScope(prefix, vars) => vars
                        .into_iter()
                        .map(|v| CompletionCandidate {
                            display: format!("{} ({})", v.name, v.type_),
                            replacement: format!("{}", &v.name[prefix.len()..]),
                        })
                        .collect(),

                    server::Completion::Behaviours(prefix, behaviours) => behaviours
                        .into_iter()
                        .map(|behaviour| CompletionCandidate {
                            display: behaviour.to_string(),
                            replacement: format!("{}", &behaviour.selector()[prefix.len()..]),
                        })
                        .collect(),
                };
                return Ok((pos, candidates));
            }
        }
        Ok((0, vec![]))
    }
}

pub fn highlight(source: Arc<Source>, markers: Vec<(Color, Span)>) -> String {
    let tokens = tokenize(source);
    tokens
        .into_iter()
        .map(|token| {
            let lexeme = token.lexeme();

            for (color, span) in markers.iter() {
                if span.contains_location(&token.span.start)
                    && span.contains_location(&token.span.end)
                {
                    return lexeme.color(color.clone()).underline().to_string();
                }
            }

            match token.kind {
                TokenKind::EOF => lexeme,
                TokenKind::Unknown(_) => lexeme.red().underline().to_string(),
                TokenKind::Whitespace(_) => lexeme,

                TokenKind::Arrow
                | TokenKind::FatArrow
                | TokenKind::Period
                | TokenKind::Comma
                | TokenKind::OpenCurly
                | TokenKind::CloseCurly
                | TokenKind::LineComment(_)
                | TokenKind::Underscore => lexeme.bright_black().to_string(),

                TokenKind::SimpleString(_) | TokenKind::SimpleCharacter(_) => {
                    lexeme.green().to_string()
                }

                TokenKind::SymbolLiteral(_) => lexeme.cyan().to_string(),

                TokenKind::AsKeyword
                | TokenKind::InKeyword
                | TokenKind::IsKeyword
                | TokenKind::OutKeyword
                | TokenKind::InoutKeyword
                | TokenKind::ClassKeyword
                | TokenKind::PrivateKeyword
                | TokenKind::PublicKeyword
                | TokenKind::NamespaceKeyword
                | TokenKind::SelfKeyword
                | TokenKind::ImportKeyword
                | TokenKind::ExportKeyword
                | TokenKind::PartialKeyword
                | TokenKind::LetKeyword
                | TokenKind::NativeKeyword => lexeme.blue().to_string(),
                TokenKind::Plus => lexeme,
                TokenKind::Colon => lexeme,
                TokenKind::Slash => lexeme,
                TokenKind::EqualSign => lexeme,
                TokenKind::OpenAngle => lexeme,
                TokenKind::CloseAngle => lexeme,
                TokenKind::SimpleInteger(_) | TokenKind::SimpleFloat(_) => {
                    lexeme.magenta().to_string()
                }
                TokenKind::SimpleSymbol(_) => lexeme,
            }
        })
        .collect::<String>()
}

impl Highlighter for EditorHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        let mut server = self.server.lock().unwrap();
        server.set(self.uri.clone(), line.into(), SourceKind::REPLLine);
        let source = server.get(&self.uri).unwrap().source;
        Owned(highlight(source, vec![]))
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Borrowed(hint)
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        _completion: CompletionType,
    ) -> Cow<'c, str> {
        Borrowed(candidate)
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        true
    }
}

pub struct REPL {
    editor: Editor<EditorHelper>,
    server: Arc<Mutex<Server>>,
    vm: VM,
}

impl REPL {
    pub fn new<R: Reporter>() -> REPL {
        let mut server = Server::new();

        let mut sources = Source::files("**/*.loa").unwrap_or(vec![]);
        sources.extend(Source::stdlib().expect("failed to load stdlib"));
        server.add_all(sources.clone());

        let mut failure = false;
        for (_, diagnostics) in server.diagnostics() {
            if Diagnostic::failed(&diagnostics) {
                failure = true;
            }
            R::report(diagnostics, &server.analysis.navigator);
        }

        let mut vm = VM::new();

        if failure {
            server = Server::new();
        } else {
            match server.generator().generate_all() {
                Err(err) => eprintln!("{:?}", err),
                Ok(i) => {
                    vm.eval::<loa::vm::ServerNative>(i);
                }
            };
        }

        let server = Arc::new(Mutex::new(server));

        let mut editor = Editor::new();
        editor.set_edit_mode(EditMode::Vi);
        editor.set_completion_type(CompletionType::List);
        editor.set_helper(Some(EditorHelper {
            server: server.clone(),
            uri: URI::REPLLine(0),
        }));

        REPL { editor, server, vm }
    }

    pub fn start<R: Reporter>(&mut self) {
        let mut n = 1;
        let mut line = String::new();
        loop {
            let uri = loa::URI::REPLLine(n);
            self.editor.helper_mut().unwrap().uri = uri.clone();
            let addition = match self.editor.readline(
                if line.len() == 0 { ">>> " } else { "... " }
                    .bright_black()
                    .to_string()
                    .as_ref(),
            ) {
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
            self.editor.add_history_entry(&line);
            n += 1;

            let mut failure = false;
            for (d_uri, diagnostics) in server.diagnostics() {
                if d_uri == uri {
                    if Diagnostic::failed(&diagnostics) {
                        failure = true;
                    }
                    R::report(diagnostics, &server.analysis.navigator);
                }
            }
            if failure {
                server.remove(uri);
                continue;
            }

            let is_expression = server
                .analysis
                .navigator
                .root_of(&uri)
                .and_then(|repl_line| {
                    if let syntax::REPLLine { mut statements } = repl_line.kind {
                        statements.pop()
                    } else {
                        None
                    }
                })
                .and_then(|statement| server.analysis.navigator.find_node_in(&uri, statement))
                .map(|statement| match statement.kind {
                    syntax::REPLExpression { .. } => true,
                    _ => false,
                })
                .unwrap_or(false);

            match Generator::new(&mut server.analysis).generate::<REPLDirectivesImpl>(&uri) {
                Err(err) => {
                    server.remove(uri);
                    println!("{:?}", err)
                }
                Ok(instructions) => {
                    if is_expression {
                        if let Some(o) = self.vm.eval_pop::<loa::vm::ServerNative>(instructions) {
                            println!("{}", o);
                        }
                    } else {
                        self.vm.eval::<loa::vm::ServerNative>(instructions);
                    }
                }
            }
        }
    }
}

struct REPLDirectivesImpl;

impl REPLDirectives for REPLDirectivesImpl {
    fn show_type(type_: Type) {
        println!("{}", type_.to_string().blue());
    }

    fn show_behaviours(type_: Type, types: &semantics::Types) {
        for b in types.get_behaviours(&type_) {
            println!("{}", b.to_string().magenta());
        }
    }
}
