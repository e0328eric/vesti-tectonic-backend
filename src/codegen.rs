// Implementing ToString for Statement enum so that making full latex text easily.

use std::collections::HashMap;

use crate::commands::LatexEngineType;
use crate::error::{self, VestiErr, VestiParseErrKind};
use crate::location::Span;
use crate::parser::ast::*;
use crate::parser::Parser;
use crate::python::PythonVm;

#[repr(transparent)]
#[derive(Clone)]
struct PythonCode(String);

pub struct Codegen<'p> {
    parser: Box<Parser<'p>>,
    latex_type: LatexEngineType,
    pycode: HashMap<String, PythonCode>,
}

impl<'p> Codegen<'p> {
    pub fn new(parser: Box<Parser<'p>>, latex_type: LatexEngineType) -> Self {
        Self {
            parser,
            latex_type,
            pycode: HashMap::with_capacity(10),
        }
    }

    #[inline]
    pub fn is_main_vesti(&self) -> bool {
        self.parser.is_main_vesti()
    }

    pub fn make_latex_format<const IS_TEST: bool>(&mut self) -> error::Result<String> {
        let latex = self.parser.parse_latex()?;
        let mut output = String::new();

        if !IS_TEST {
            output += &format!(
                "%\n%  This file was generated by vesti {}\n",
                env!("CARGO_PKG_VERSION")
            );
            output += &format!("%  Compile this file using {} engine\n%\n", self.latex_type)
        }

        for stmt in latex {
            if stmt == Statement::NopStmt {
                continue;
            }
            output += &self.eval_vesti(&stmt)?;
        }

        Ok(output)
    }

    fn eval_vesti(&mut self, stmt: &Statement) -> error::Result<String> {
        match stmt {
            // an empty statement
            Statement::NopStmt => Ok(String::new()),
            Statement::NonStopMode => Ok(String::from("\\nonstopmode\n")),
            Statement::ImportExpl3Pkg => Ok(String::from("\\usepackage{expl3, xparse}\n")),
            Statement::MakeAtLetter => Ok(String::from("\\makeatletter\n")),
            Statement::MakeAtOther => Ok(String::from("\\makeatother\n")),
            Statement::Latex3On => Ok(String::from("\\ExplSyntaxOn\n")),
            Statement::Latex3Off => Ok(String::from("\\ExplSyntaxOff\n")),
            Statement::DocumentClass { name, options } => self.docclass_to_string(name, options),
            Statement::Usepackage { name, options } => self.usepackage_to_string(name, options),
            Statement::MultiUsepackages { pkgs } => self.multiusepacakge_to_string(pkgs),
            Statement::ImportVesti { filename } => Ok(format!("\\input{{{}}}", filename.display())),
            Statement::FilePath { filename } => Ok(format!("{}", filename.display())),
            Statement::DocumentStart => Ok(String::from("\\begin{document}\n")),
            Statement::DocumentEnd => Ok(String::from("\n\\end{document}\n")),
            Statement::MainText(s) => Ok(s.clone()),
            Statement::BracedStmt(latex) => Ok(format!("{{{}}}", self.latex_to_string(latex)?)),
            Statement::MathDelimiter { delimiter, kind } => {
                self.math_delimiter_to_string(delimiter, kind)
            }
            Statement::Fraction {
                numerator,
                denominator,
            } => self.fraction_to_string(numerator, denominator),
            Statement::PlainTextInMath {
                remove_front_space,
                remove_back_space,
                text,
            } => self.plaintext_in_math_to_string(*remove_front_space, *remove_back_space, text),
            Statement::Integer(i) => Ok(i.to_string()),
            Statement::Float(f) => Ok(f.to_string()),
            Statement::RawLatex(s) => Ok(s.clone()),
            Statement::MathText { state, text } => self.math_text_to_string(*state, text),
            Statement::LatexFunction { name, args } => self.latex_function_to_string(name, args),
            Statement::Environment { name, args, text } => {
                self.environment_to_string(name, args, text)
            }
            Statement::BeginPhantomEnvironment {
                name,
                args,
                add_newline,
            } => self.begin_phantom_environment_to_string(name, args, *add_newline),
            Statement::EndPhantomEnvironment { name } => Ok(format!("\\end{{{name}}}")),
            Statement::FunctionDefine {
                kind,
                name,
                args,
                trim,
                body,
            } => self.function_def_to_string(kind, name, args, trim, body),
            Statement::EnvironmentDefine {
                is_redefine,
                name,
                args_num,
                optional_arg,
                trim,
                begin_part,
                end_part,
            } => self.environment_def_to_string(
                *is_redefine,
                name,
                *args_num,
                optional_arg.as_ref(),
                trim,
                begin_part,
                end_part,
            ),
            Statement::PythonCode {
                pycode_span,
                pycode_import,
                pycode_export,
                code,
            } => self.run_pycode(*pycode_span, pycode_import, pycode_export, code),
        }
    }

    fn docclass_to_string(
        &mut self,
        name: &str,
        options: &Option<Vec<Latex>>,
    ) -> error::Result<String> {
        Ok(if let Some(opts) = options {
            let mut options_str = String::new();
            for o in opts {
                options_str = options_str + &self.latex_to_string(o)? + ",";
            }
            options_str.pop();

            format!("\\documentclass[{options_str}]{{{name}}}\n")
        } else {
            format!("\\documentclass{{{name}}}\n")
        })
    }

    fn usepackage_to_string(
        &mut self,
        name: &str,
        options: &Option<Vec<Latex>>,
    ) -> error::Result<String> {
        Ok(if let Some(opts) = options {
            let mut options_str = String::new();
            for o in opts {
                options_str = options_str + &self.latex_to_string(o)? + ",";
            }
            options_str.pop();

            format!("\\usepackage[{options_str}]{{{name}}}\n")
        } else {
            format!("\\usepackage{{{name}}}\n")
        })
    }

    fn multiusepacakge_to_string(&mut self, pkgs: &[Statement]) -> error::Result<String> {
        let mut output = String::new();
        for pkg in pkgs {
            if let Statement::Usepackage { name, options } = pkg {
                output += &self.usepackage_to_string(name, options)?;
            }
        }
        Ok(output)
    }

    fn math_text_to_string(
        &mut self,
        state: MathState,
        text: &[Statement],
    ) -> error::Result<String> {
        let mut output = String::new();
        match state {
            MathState::Text => {
                output += "$";
                for t in text {
                    output += &self.eval_vesti(t)?;
                }
                output += "$";
            }
            MathState::Inline => {
                output += "\\[";
                for t in text {
                    output += &self.eval_vesti(t)?;
                }
                output += "\\]";
            }
        }
        Ok(output)
    }

    fn math_delimiter_to_string(
        &mut self,
        delimiter: &str,
        kind: &DelimiterKind,
    ) -> error::Result<String> {
        Ok(match kind {
            DelimiterKind::Default => String::from(delimiter),
            DelimiterKind::LeftBig => format!("\\left{delimiter}"),
            DelimiterKind::RightBig => format!("\\right{delimiter}"),
        })
    }

    fn fraction_to_string(
        &mut self,
        numerator: &Latex,
        denominator: &Latex,
    ) -> error::Result<String> {
        Ok(format!(
            "\\frac{{{}}}{{{}}}",
            self.latex_to_string(numerator)?,
            self.latex_to_string(denominator)?
        ))
    }

    fn plaintext_in_math_to_string(
        &mut self,
        remove_front_space: bool,
        remove_back_space: bool,
        text: &Latex,
    ) -> error::Result<String> {
        let output = self.latex_to_string(text)?;
        Ok(match (remove_front_space, remove_back_space) {
            (false, false) => format!("\\text{{{}}}", output),
            (true, false) => format!("\\text{{ {}}}", output),
            (false, true) => format!("\\text{{{} }}", output),
            (true, true) => format!("\\text{{ {} }}", output),
        })
    }

    fn latex_function_to_string(
        &mut self,
        name: &str,
        args: &Vec<(ArgNeed, Vec<Statement>)>,
    ) -> error::Result<String> {
        let mut output = name.to_string();
        for arg in args {
            let mut tmp = String::new();
            for t in &arg.1 {
                tmp += &self.eval_vesti(t)?;
            }
            match arg.0 {
                ArgNeed::MainArg => output += &format!("{{{tmp}}}"),
                ArgNeed::Optional => output += &format!("[{tmp}]"),
                ArgNeed::StarArg => output.push('*'),
            }
        }
        Ok(output)
    }

    fn begin_phantom_environment_to_string(
        &mut self,
        name: &str,
        args: &Vec<(ArgNeed, Vec<Statement>)>,
        add_newline: bool,
    ) -> error::Result<String> {
        let mut output = format!("\\begin{{{name}}}");
        if add_newline {
            output.push('\n');
        }
        for arg in args {
            let mut tmp = String::new();
            for t in &arg.1 {
                tmp += &self.eval_vesti(t)?;
            }
            match arg.0 {
                ArgNeed::MainArg => output += &format!("{{{tmp}}}"),
                ArgNeed::Optional => output += &format!("[{tmp}]"),
                ArgNeed::StarArg => output.push('*'),
            }
        }
        Ok(output)
    }

    fn environment_to_string(
        &mut self,
        name: &str,
        args: &Vec<(ArgNeed, Vec<Statement>)>,
        text: &Latex,
    ) -> error::Result<String> {
        let mut output = format!("\\begin{{{name}}}");
        for arg in args {
            let mut tmp = String::new();
            for t in &arg.1 {
                tmp += &self.eval_vesti(t)?;
            }
            match arg.0 {
                ArgNeed::MainArg => output += &format!("{{{tmp}}}"),
                ArgNeed::Optional => output += &format!("[{tmp}]"),
                ArgNeed::StarArg => output.push('*'),
            }
        }
        for t in text {
            output += &self.eval_vesti(t)?;
        }
        output += &format!("\\end{{{name}}}\n");
        Ok(output)
    }

    fn latex_to_string(&mut self, latex: &Latex) -> error::Result<String> {
        let mut output = String::new();
        for l in latex {
            output += &self.eval_vesti(l)?;
        }
        Ok(output)
    }

    fn function_def_to_string(
        &mut self,
        kind: &FunctionDefKind,
        name: &str,
        args: &str,
        trim: &TrimWhitespace,
        body: &Latex,
    ) -> error::Result<String> {
        use FunctionDefKind as FDK;

        let mut output = String::with_capacity(30);

        if kind.has_property(FDK::LONG) {
            output.push_str("\\long");
        }

        if kind.has_property(FDK::OUTER) {
            output.push_str("\\outer");
        }

        if kind.has_property(FDK::EXPAND | FDK::GLOBAL) {
            output.push_str("\\xdef")
        } else if kind.has_property(FDK::GLOBAL) {
            output.push_str("\\gdef")
        } else if kind.has_property(FDK::EXPAND) {
            output.push_str("\\edef")
        } else {
            output.push_str("\\def")
        }

        output += &format!("\\{name}{args}{{");
        if trim.start {
            output += "%\n";
        }

        let mut tmp = String::new();
        for b in body {
            tmp += &self.eval_vesti(b)?;
        }

        output += match (trim.start, trim.end) {
            (false, false) => tmp.as_str(),
            (false, true) => tmp.trim_end(),
            (true, false) => tmp.trim_start(),
            (true, true) => tmp.trim(),
        };
        output.push_str("%\n}\n");

        Ok(output)
    }

    #[allow(clippy::too_many_arguments)]
    fn environment_def_to_string(
        &mut self,
        is_redefine: bool,
        name: &str,
        args_num: u8,
        optional_arg: Option<&Latex>,
        trim: &TrimWhitespace,
        begin_part: &Latex,
        end_part: &Latex,
    ) -> error::Result<String> {
        let mut output = if is_redefine {
            format!("\\renewenvironment{{{name}}}")
        } else {
            format!("\\newenvironment{{{name}}}")
        };

        if args_num > 0 {
            output += &format!("[{args_num}]");
            if let Some(inner) = optional_arg {
                output.push('[');
                for stmt in inner {
                    output += &self.eval_vesti(stmt)?;
                }
                output.push_str("]{");
            } else {
                output.push('{');
            }
        } else {
            output.push('{');
        }

        let mut tmp = String::new();
        for b in begin_part {
            tmp += &self.eval_vesti(b)?;
        }
        output += match (trim.start, trim.mid) {
            (false, Some(false)) => tmp.as_str(),
            (true, Some(false)) => tmp.trim_start(),
            (false, Some(true)) => tmp.trim_end(),
            (true, Some(true)) => tmp.trim(),
            _ => unreachable!("VESTI BUG!!!!: codegen::environment_def_to_string"),
        };
        output.push_str("}{");

        tmp.clear();
        for b in end_part {
            tmp += &self.eval_vesti(b)?;
        }
        output += match (trim.mid, trim.end) {
            (Some(false), false) => tmp.as_str(),
            (Some(true), false) => tmp.trim_start(),
            (Some(false), true) => tmp.trim_end(),
            (Some(true), true) => tmp.trim(),
            _ => unreachable!("VESTI BUG!!!!: codegen::environment_def_to_string"),
        };
        output.push_str("}\n");

        Ok(output)
    }

    fn run_pycode(
        &mut self,
        pycode_span: Span,
        pycode_import: &Option<Vec<String>>,
        pycode_export: &Option<String>,
        code: &str,
    ) -> error::Result<String> {
        let mut pycode = String::with_capacity(code.len());

        match pycode_import {
            None => {}
            Some(labels) => {
                for label in labels {
                    let Some(import_pycode) = self.pycode.get(label) else {
                        return Err(VestiErr::ParseErr {
                            err_kind: VestiParseErrKind::PythonEvalErr {
                                msg: format!("the label <{label}> cannot be found. Notice that the label should be forward declared"),
                            },
                            location: pycode_span,
                        });
                    };
                    pycode.push_str(&import_pycode.0);
                    pycode.push_str("\n\n");
                }
            }
        }
        match pycode_export {
            None => {}
            Some(label) => {
                self.pycode
                    .insert(label.clone(), PythonCode(code.to_string()));
            }
        }

        pycode.push_str(code);

        let python = PythonVm::new(pycode, pycode_span)?;
        python.run()
    }
}
