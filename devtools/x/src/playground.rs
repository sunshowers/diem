// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Playground for arbitrary code.
//!
//! This lets users experiment with new lints and other throwaway code.
//! Add your code in the spots marked `// --- ADD PLAYGROUND CODE HERE ---`.
//!
//! This file should not have any production-related code checked into it.

#![allow(unused_variables)]

use crate::context::XContext;
use anyhow::anyhow;
use guppy::{
    graph::{
        cargo::{CargoOptions, CargoResolverVersion},
        feature::default_filter,
        DependencyDirection, PackageMetadata,
    },
    PackageId, Platform,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use structopt::StructOpt;
use x_core::WorkspaceStatus;
use x_lint::prelude::*;

#[derive(Copy, Clone, Debug)]
struct PlaygroundProject;

impl Linter for PlaygroundProject {
    fn name(&self) -> &'static str {
        "playground-project"
    }
}

impl ProjectLinter for PlaygroundProject {
    fn run<'l>(
        &self,
        ctx: &ProjectContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        // --- ADD PLAYGROUND CODE HERE ---

        let package_graph = ctx.package_graph()?;
        let workspace = package_graph.workspace();
        let default_members = ctx.default_members()?;

        let cargo_opts = CargoOptions::new()
            .with_version(CargoResolverVersion::V2)
            .with_dev_deps(false)
            .with_platform(Some(Platform::current().expect("wat")));

        let mut version_features: BTreeMap<
            (&PackageId, BuildPlatform),
            BTreeMap<BTreeSet<&str>, Vec<PackageMetadata<'_>>>,
        > = BTreeMap::new();

        let mut total_considered = 0;

        for package in workspace.iter() {
            match default_members.status_of(package.id()) {
                WorkspaceStatus::RootMember | WorkspaceStatus::Dependency => {
                    total_considered += 1;
                    let cargo_set = package
                        .to_package_query(DependencyDirection::Forward)
                        .to_feature_query(default_filter())
                        .resolve_cargo(&cargo_opts)
                        .expect("cargo resolution should succeed");

                    let build_features = &[
                        (BuildPlatform::Target, cargo_set.target_features()),
                        (BuildPlatform::Host, cargo_set.host_features()),
                    ];

                    for (build, features) in build_features {
                        for feature_list in
                            features.packages_with_features(DependencyDirection::Forward)
                        {
                            let id = feature_list.package().id();
                            let features = feature_list.features().iter().copied().collect();

                            version_features
                                .entry((id, *build))
                                .or_default()
                                .entry(features)
                                .or_default()
                                .push(package);
                        }
                    }
                }
                _ => {}
            }
        }

        let mut total_non_workspace = 0;
        let mut total = 0;
        for ((built_id, build), map) in version_features {
            if map.len() <= 1 {
                continue;
            }

            println!("* {} ({:?}): count: {}", built_id, build, map.len());
            for (features, packages) in map {
                let feature_str = features.iter().copied().collect::<Vec<_>>().join(", ");
                println!("  for features {} ({} packages):", feature_str, packages.len());
                for package in packages {
                    println!("    * {}", package.name());
                }
            }

            total += 1;
            if !package_graph.metadata(built_id).expect("valid package").in_workspace() {
                total_non_workspace += 1;
            }
        }

        println!("\ntotal workspace packages considered: {}", total_considered);
        println!("\ntotal multi-features: {} (non-workspace: {})", total, total_non_workspace);

        Ok(RunStatus::Executed)
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum BuildPlatform {
    Target,
    Host,
}

#[derive(Copy, Clone, Debug)]
struct PlaygroundPackage;

impl Linter for PlaygroundPackage {
    fn name(&self) -> &'static str {
        "playground-package"
    }
}

impl PackageLinter for PlaygroundPackage {
    fn run<'l>(
        &self,
        ctx: &PackageContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        // --- ADD PLAYGROUND CODE HERE ---

        Ok(RunStatus::Executed)
    }
}

#[derive(Copy, Clone, Debug)]
struct PlaygroundFilePath;

impl Linter for PlaygroundFilePath {
    fn name(&self) -> &'static str {
        "playground-file-path"
    }
}

impl FilePathLinter for PlaygroundFilePath {
    fn run<'l>(
        &self,
        ctx: &FilePathContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        // --- ADD PLAYGROUND CODE HERE ---

        Ok(RunStatus::Executed)
    }
}

#[derive(Copy, Clone, Debug)]
struct PlaygroundContent;

impl Linter for PlaygroundContent {
    fn name(&self) -> &'static str {
        "playground-content"
    }
}

impl ContentLinter for PlaygroundContent {
    fn run<'l>(
        &self,
        ctx: &ContentContext<'l>,
        out: &mut LintFormatter<'l, '_>,
    ) -> Result<RunStatus<'l>> {
        // --- ADD PLAYGROUND CODE HERE ---

        Ok(RunStatus::Executed)
    }
}

// ---

#[derive(Debug, StructOpt)]
pub struct Args {
    /// Dummy arg that doesn't do anything
    #[structopt(long)]
    dummy: bool,
}

pub fn run(args: Args, xctx: XContext) -> crate::Result<()> {
    let engine = LintEngineConfig::new(xctx.core())
        .with_project_linters(&[&PlaygroundProject])
        .with_package_linters(&[&PlaygroundPackage])
        .with_file_path_linters(&[&PlaygroundFilePath])
        .with_content_linters(&[&PlaygroundContent])
        .build();

    let results = engine.run()?;

    for (source, message) in &results.messages {
        println!(
            "[{}] [{}] [{}]: {}\n",
            message.level(),
            source.name(),
            source.kind(),
            message.message()
        );
    }

    if !results.messages.is_empty() {
        Err(anyhow!("there were lint errors"))
    } else {
        Ok(())
    }
}
