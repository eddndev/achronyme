use std::fmt::Write as _;

use crate::symbol::Availability;
use crate::{CancellationPolicy, NativeBehavior, ResourceEffect, ResourceKind};

use super::BuiltinRegistry;

impl BuiltinRegistry {
    /// Render this exact audited registry as a stable Markdown reference.
    ///
    /// Hosts should construct the registry with their extra native metadata
    /// before calling this method so the generated table matches the compiler
    /// and VM surface they ship.
    pub fn markdown_reference(&self) -> String {
        self.audit()
            .expect("cannot render documentation for an invalid builtin registry");
        let mut output = String::from(
            "# Native callable registry\n\n\
             This table is generated from the canonical `BuiltinRegistry`. \
             Do not edit it by hand.\n\n\
             | Name | Arity | Context | Effects | Capabilities | Behavior | Cancellation | Resource |\n\
             | --- | ---: | --- | --- | --- | --- | --- | --- |\n",
        );
        for entry in self.entries() {
            writeln!(
                output,
                "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |",
                entry.name,
                entry.arity.describe(),
                availability_name(entry.availability),
                entry.effects,
                entry.capabilities,
                behavior_name(entry.behavior),
                cancellation_name(entry.cancellation),
                resource_effect_name(entry.resource),
            )
            .expect("writing Markdown to a String cannot fail");
        }
        output
    }
}

fn availability_name(availability: Availability) -> &'static str {
    match availability {
        Availability::Vm => "host",
        Availability::ProveIr => "prove/circuit",
        Availability::Both => "host+prove/circuit",
    }
}

fn behavior_name(behavior: NativeBehavior) -> &'static str {
    match behavior {
        NativeBehavior::Immediate => "immediate",
        NativeBehavior::Blocking => "blocking",
        NativeBehavior::Suspending => "suspending",
    }
}

fn cancellation_name(cancellation: CancellationPolicy) -> &'static str {
    match cancellation {
        CancellationPolicy::None => "none",
        CancellationPolicy::BeforeStart => "before-start",
        CancellationPolicy::Cooperative => "cooperative",
    }
}

fn resource_kind_name(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::File => "file",
        ResourceKind::Listener => "listener",
        ResourceKind::Connection => "connection",
        ResourceKind::Channel => "channel",
        ResourceKind::Task => "task",
    }
}

fn resource_effect_name(effect: ResourceEffect) -> String {
    match effect {
        ResourceEffect::None => "-".to_string(),
        ResourceEffect::Creates(kind) => format!("creates:{}", resource_kind_name(kind)),
        ResourceEffect::Consumes(kind) => format!("consumes:{}", resource_kind_name(kind)),
        ResourceEffect::Borrows(kind) => format!("borrows:{}", resource_kind_name(kind)),
        ResourceEffect::CreatesAndBorrows { created, borrowed } => format!(
            "creates:{}+borrows:{}",
            resource_kind_name(created),
            resource_kind_name(borrowed)
        ),
    }
}
