use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier, Program, Statement};

pub(super) enum ImportKind<'source> {
    Alien,
    Tokens,
    Css,
    Custom,
    Value(&'source str),
}

#[derive(Default)]
pub(super) struct Imports<'source> {
    pub mode: alien_ir::Mode,
    alien: Vec<&'source str>,
    tokens: Vec<&'source str>,
    css: Vec<&'source str>,
    custom: Vec<&'source str>,
    values: Vec<(&'source str, &'source str)>,
}

impl<'source> Imports<'source> {
    pub(super) fn from_program(program: &Program<'source>) -> Self {
        let mut imports = Self::default();
        for_each_import(program, |declaration| {
            for_each_alien_binding(declaration, |kind, name| match kind {
                ImportKind::Alien => imports.alien.push(name),
                ImportKind::Tokens => imports.tokens.push(name),
                ImportKind::Css => imports.css.push(name),
                ImportKind::Custom => imports.custom.push(name),
                ImportKind::Value(export) => imports.values.push((name, export)),
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
        self.is_token_root(name) || self.is_alien(name)
    }

    pub(super) fn is_token_root(&self, name: &str) -> bool {
        self.tokens.contains(&name)
    }

    pub(super) fn is_css(&self, name: &str) -> bool {
        self.css.contains(&name)
    }

    pub(super) fn is_custom(&self, name: &str) -> bool {
        self.custom.contains(&name)
    }

    pub(super) fn value_export(&self, name: &str) -> Option<&'source str> {
        self.values
            .iter()
            .find_map(|&(binding, export)| (binding == name).then_some(export))
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
    mut visit: impl FnMut(ImportKind<'source>, &'source str),
) {
    let module = declaration.source.value.as_str();
    if !matches!(module, Some("alien" | "alien/values")) || declaration.import_kind.is_type() {
        return;
    }

    for specifier in declaration.specifiers.iter().flatten() {
        if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier
            && !specifier.import_kind.is_type()
        {
            let kind = match (module, specifier.imported.name().as_str()) {
                (Some("alien"), "alien") => ImportKind::Alien,
                (Some("alien"), "tokens") => ImportKind::Tokens,
                (Some("alien"), "css") => ImportKind::Css,
                (Some("alien"), "custom") => ImportKind::Custom,
                (Some("alien/values"), export) => ImportKind::Value(export),
                _ => continue,
            };
            visit(kind, specifier.local.name.as_str());
        }
    }
}
