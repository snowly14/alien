use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier, Program, Statement};

pub(super) enum ImportKind {
    Alien,
    Tokens,
}

#[derive(Default)]
pub(super) struct Imports<'source> {
    alien: Vec<&'source str>,
    tokens: Vec<&'source str>,
}

impl<'source> Imports<'source> {
    pub(super) fn from_program(program: &Program<'source>) -> Self {
        let mut imports = Self::default();
        for_each_import(program, |declaration| {
            for_each_alien_binding(declaration, |kind, name| match kind {
                ImportKind::Alien => imports.alien.push(name),
                ImportKind::Tokens => imports.tokens.push(name),
            });
        });
        imports
    }

    pub(super) fn has_alien(&self) -> bool {
        !self.alien.is_empty()
    }

    pub(super) fn is_alien(&self, name: &str) -> bool {
        self.alien.contains(&name)
    }

    pub(super) fn is_color_root(&self, name: &str) -> bool {
        self.tokens.contains(&name) || self.is_alien(name)
    }
}

pub(super) fn for_each_import<'source>(
    program: &Program<'source>,
    mut visit: impl FnMut(&ImportDeclaration<'source>),
) {
    for statement in &program.body {
        if let Statement::ImportDeclaration(declaration) = statement {
            visit(declaration);
        }
    }
}

pub(super) fn for_each_alien_binding<'source>(
    declaration: &ImportDeclaration<'source>,
    mut visit: impl FnMut(ImportKind, &'source str),
) {
    if declaration.source.value.as_str() != Some("alien") || declaration.import_kind.is_type() {
        return;
    }

    for specifier in declaration.specifiers.iter().flatten() {
        if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier
            && !specifier.import_kind.is_type()
        {
            let kind = match specifier.imported.name().as_str() {
                "alien" => ImportKind::Alien,
                "tokens" => ImportKind::Tokens,
                _ => continue,
            };
            visit(kind, specifier.local.name.as_str());
        }
    }
}
