//! User-provided program settings, taking into account pyproject.toml and
//! command-line options. Structure mirrors the user-facing representation of
//! the various parameters.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use path_absolutize::path_dedot;
use regex::Regex;

use crate::checks_gen::{CheckCodePrefix, CATEGORIES};
use crate::settings::pyproject::load_options;
use crate::settings::types::{FilePattern, PerFileIgnore, PythonVersion};
use crate::{
    flake8_annotations, flake8_bugbear, flake8_quotes, flake8_tidy_imports, fs, isort, mccabe,
    pep8_naming,
};

#[derive(Debug)]
pub struct Configuration {
    pub dummy_variable_rgx: Regex,
    pub exclude: Vec<FilePattern>,
    pub extend_exclude: Vec<FilePattern>,
    pub extend_ignore: Vec<CheckCodePrefix>,
    pub extend_select: Vec<CheckCodePrefix>,
    pub external: Vec<String>,
    pub fix: bool,
    pub fixable: Vec<CheckCodePrefix>,
    pub ignore: Vec<CheckCodePrefix>,
    pub line_length: usize,
    pub per_file_ignores: Vec<PerFileIgnore>,
    pub select: Vec<CheckCodePrefix>,
    pub show_source: bool,
    pub src: Vec<PathBuf>,
    pub target_version: PythonVersion,
    pub unfixable: Vec<CheckCodePrefix>,
    // Plugins
    pub flake8_annotations: flake8_annotations::settings::Settings,
    pub flake8_bugbear: flake8_bugbear::settings::Settings,
    pub flake8_quotes: flake8_quotes::settings::Settings,
    pub flake8_tidy_imports: flake8_tidy_imports::settings::Settings,
    pub isort: isort::settings::Settings,
    pub mccabe: mccabe::settings::Settings,
    pub pep8_naming: pep8_naming::settings::Settings,
}

static DEFAULT_EXCLUDE: Lazy<Vec<FilePattern>> = Lazy::new(|| {
    vec![
        FilePattern::Builtin(".bzr"),
        FilePattern::Builtin(".direnv"),
        FilePattern::Builtin(".eggs"),
        FilePattern::Builtin(".git"),
        FilePattern::Builtin(".hg"),
        FilePattern::Builtin(".mypy_cache"),
        FilePattern::Builtin(".nox"),
        FilePattern::Builtin(".pants.d"),
        FilePattern::Builtin(".ruff_cache"),
        FilePattern::Builtin(".svn"),
        FilePattern::Builtin(".tox"),
        FilePattern::Builtin(".venv"),
        FilePattern::Builtin("__pypackages__"),
        FilePattern::Builtin("_build"),
        FilePattern::Builtin("buck-out"),
        FilePattern::Builtin("build"),
        FilePattern::Builtin("dist"),
        FilePattern::Builtin("node_modules"),
        FilePattern::Builtin("venv"),
    ]
});

static DEFAULT_DUMMY_VARIABLE_RGX: Lazy<Regex> =
    Lazy::new(|| Regex::new("^(_+|(_+[a-zA-Z0-9_]*[a-zA-Z0-9]+?))$").unwrap());

impl Configuration {
    pub fn from_pyproject(
        pyproject: Option<&PathBuf>,
        project_root: Option<&PathBuf>,
    ) -> Result<Self> {
        let options = load_options(pyproject)?;
        Ok(Configuration {
            dummy_variable_rgx: match options.dummy_variable_rgx {
                Some(pattern) => Regex::new(&pattern)
                    .map_err(|e| anyhow!("Invalid dummy-variable-rgx value: {e}"))?,
                None => DEFAULT_DUMMY_VARIABLE_RGX.clone(),
            },
            src: options.src.map_or_else(
                || {
                    vec![match project_root {
                        Some(project_root) => project_root.clone(),
                        None => path_dedot::CWD.clone(),
                    }]
                },
                |src| {
                    src.iter()
                        .map(|path| {
                            let path = Path::new(path);
                            match project_root {
                                Some(project_root) => fs::normalize_path_to(path, project_root),
                                None => fs::normalize_path(path),
                            }
                        })
                        .collect()
                },
            ),
            target_version: options.target_version.unwrap_or(PythonVersion::Py310),
            exclude: options.exclude.map_or_else(
                || DEFAULT_EXCLUDE.clone(),
                |paths| paths.into_iter().map(FilePattern::User).collect(),
            ),
            extend_exclude: options
                .extend_exclude
                .map(|paths| paths.into_iter().map(FilePattern::User).collect())
                .unwrap_or_default(),
            extend_ignore: options.extend_ignore.unwrap_or_default(),
            select: options
                .select
                .unwrap_or_else(|| vec![CheckCodePrefix::E, CheckCodePrefix::F]),
            extend_select: options.extend_select.unwrap_or_default(),
            external: options.external.unwrap_or_default(),
            fix: options.fix.unwrap_or_default(),
            fixable: options.fixable.unwrap_or_else(|| CATEGORIES.to_vec()),
            unfixable: options.unfixable.unwrap_or_default(),
            ignore: options.ignore.unwrap_or_default(),
            line_length: options.line_length.unwrap_or(88),
            per_file_ignores: options
                .per_file_ignores
                .map(|per_file_ignores| {
                    per_file_ignores
                        .into_iter()
                        .map(|(pattern, prefixes)| PerFileIgnore::new(pattern, &prefixes))
                        .collect()
                })
                .unwrap_or_default(),
            show_source: options.show_source.unwrap_or_default(),
            // Plugins
            flake8_annotations: options
                .flake8_annotations
                .map(flake8_annotations::settings::Settings::from_options)
                .unwrap_or_default(),
            flake8_bugbear: options
                .flake8_bugbear
                .map(flake8_bugbear::settings::Settings::from_options)
                .unwrap_or_default(),
            flake8_quotes: options
                .flake8_quotes
                .map(flake8_quotes::settings::Settings::from_options)
                .unwrap_or_default(),
            flake8_tidy_imports: options
                .flake8_tidy_imports
                .map(flake8_tidy_imports::settings::Settings::from_options)
                .unwrap_or_default(),
            isort: options
                .isort
                .map(isort::settings::Settings::from_options)
                .unwrap_or_default(),
            mccabe: options
                .mccabe
                .map(mccabe::settings::Settings::from_options)
                .unwrap_or_default(),
            pep8_naming: options
                .pep8_naming
                .map(pep8_naming::settings::Settings::from_options)
                .unwrap_or_default(),
        })
    }
}
