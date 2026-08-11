//! Quoted, syntax-quoted, collection, and primitive emission.

use super::*;

impl Compiler {
    pub(super) fn compile_quote(
        &mut self,
        children: &[Child<'_>],
        span: &Span,
    ) -> Result<(), CompileError> {
        if children.len() != 2 {
            return Err(CompileError::new(
                CompileErrorKind::Arity,
                "quote expects one argument",
                Some(span.start),
            ));
        }
        let value = crate::core::form_to_value(children[1].form).map_err(|message| {
            CompileError::new(CompileErrorKind::UnsupportedForm, message, Some(span.start))
        })?;
        self.constant(value, span)
    }

    pub(super) fn compile_syntax_quote(
        &mut self,
        children: &[Child<'_>],
        span: &Span,
    ) -> Result<(), CompileError> {
        if children.len() != 2 {
            return Err(CompileError::new(
                CompileErrorKind::Arity,
                "syntax-quote expects one argument",
                Some(span.start),
            ));
        }
        self.compile_syntax_value(children[1].form, span, false)
    }

    pub(super) fn compile_syntax_value(
        &mut self,
        form: &Form,
        span: &Span,
        nested: bool,
    ) -> Result<(), CompileError> {
        if let Some(argument) = unquote_argument(form, "unquote") {
            let argument = argument.map_err(|message| {
                CompileError::new(CompileErrorKind::Arity, message, Some(span.start))
            })?;
            return self.compile_form(&argument, span, None, false);
        }
        if unquote_argument(form, "unquote-splicing").is_some() {
            return Err(CompileError::new(
                CompileErrorKind::UnsupportedForm,
                if nested {
                    "unquote-splicing is only valid as a collection element"
                } else {
                    "unquote-splicing is not valid at the root of syntax-quote"
                },
                Some(span.start),
            ));
        }
        match crate::core::form_without_metadata(form) {
            Form::List(values) | Form::Vector(values) => {
                let vector = matches!(crate::core::form_without_metadata(form), Form::Vector(_));
                let spliced = values
                    .iter()
                    .any(|value| unquote_argument(value, "unquote-splicing").is_some());
                for value in values {
                    if let Some(argument) = unquote_argument(value, "unquote-splicing") {
                        let argument = argument.map_err(|message| {
                            CompileError::new(CompileErrorKind::Arity, message, Some(span.start))
                        })?;
                        self.compile_form(&argument, span, None, false)?;
                    } else {
                        self.compile_syntax_value(value, span, true)?;
                        if spliced {
                            self.emit(Instruction::BuildList(1), Some(span.start));
                        }
                    }
                }
                let count = self.collection_count(values.len(), span)?;
                if spliced {
                    self.emit(Instruction::ConcatList(count), Some(span.start));
                    if vector {
                        self.emit(Instruction::ToVector, Some(span.start));
                    }
                } else if vector {
                    self.emit(Instruction::BuildVector(count), Some(span.start));
                } else {
                    self.emit(Instruction::BuildList(count), Some(span.start));
                }
                Ok(())
            }
            Form::Map(entries) => {
                for (key, value) in entries {
                    self.compile_syntax_value(key, span, true)?;
                    self.compile_syntax_value(value, span, true)?;
                }
                self.emit(
                    Instruction::BuildMap(entries.len() as u16),
                    Some(span.start),
                );
                Ok(())
            }
            Form::Set(values) => {
                for value in values {
                    self.compile_syntax_value(value, span, true)?;
                }
                self.emit(Instruction::BuildSet(values.len() as u16), Some(span.start));
                Ok(())
            }
            _ => {
                let value = crate::core::form_to_value(form).map_err(|message| {
                    CompileError::new(CompileErrorKind::UnsupportedForm, message, Some(span.start))
                })?;
                self.constant(value, span)
            }
        }
    }

    pub(super) fn compile_primitive(
        &mut self,
        children: &[Child<'_>],
        span: &Span,
        op: Primitive,
    ) -> Result<(), CompileError> {
        let argc = children.len() - 1;
        if argc > MAX_PRIMITIVE_ARGUMENTS {
            return Err(CompileError::new(
                CompileErrorKind::Limit,
                format!("primitive calls support at most {MAX_PRIMITIVE_ARGUMENTS} arguments"),
                Some(span.start),
            ));
        }
        // Mutable conversion creates/consumes runtime identity and must run on
        // every execution. Folding it would place a one-shot transient in the
        // constant pool, so the second execution would observe a frozen value.
        if !matches!(
            op,
            Primitive::ToMutable
                | Primitive::ToPersistent
                | Primitive::ArrayNew
                | Primitive::ArraySet
                | Primitive::ObjectNew
                | Primitive::ObjectSet
        ) && children[1..]
            .iter()
            .all(|argument| constant_form(argument.form))
        {
            let arguments = children[1..]
                .iter()
                .map(|argument| crate::core::form_to_value(argument.form))
                .collect::<Result<Vec<_>, _>>();
            if let Ok(arguments) = arguments {
                if let Ok(value) = crate::core::apply_primitive(op, &arguments) {
                    return self.constant(value, span);
                }
            }
        }
        if op == Primitive::First && argc == 1 {
            if let Form::List(elements) = children[1].form {
                if matches!(elements.as_slice(), [Form::Symbol(name), _] if name == "rest") {
                    let nested =
                        self.list_children(elements, children[1].span, children[1].children);
                    if constant_form(nested[1].form) {
                        if let Ok(argument) = crate::core::form_to_value(nested[1].form) {
                            if let Ok(value) =
                                crate::core::apply_primitive(Primitive::Second, &[argument])
                            {
                                return self.constant(value, span);
                            }
                        }
                    }
                    self.compile_form(nested[1].form, nested[1].span, nested[1].children, false)?;
                    if self.ctx().fallthrough {
                        self.emit(
                            Instruction::Primitive {
                                op: Primitive::Second,
                                argc: 1,
                            },
                            Some(span.start),
                        );
                    }
                    return Ok(());
                }
            }
        }
        if argc == 2 {
            if let (Form::Symbol(name), Form::Number(value)) = (children[1].form, children[2].form)
            {
                if let Some(local) = self.ctx().scopes.resolve(name) {
                    let constant =
                        self.constant_index_of(Value::Number(*value), children[2].span)?;
                    self.emit(
                        Instruction::PrimitiveLocalConst {
                            op,
                            local,
                            constant,
                        },
                        Some(span.start),
                    );
                    return Ok(());
                }
            }
        }
        for argument in &children[1..] {
            self.compile_form(argument.form, argument.span, argument.children, false)?;
        }
        if !self.ctx().fallthrough {
            return Ok(());
        }
        self.emit(
            Instruction::Primitive {
                op,
                argc: argc as u8,
            },
            Some(span.start),
        );
        Ok(())
    }
}
