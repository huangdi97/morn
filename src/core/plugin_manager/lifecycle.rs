use std::collections::{HashMap, VecDeque};

use super::context::PluginContext;
use super::error::{PluginError, PluginOrderError};
use super::traits::MornPlugin;

pub fn topological_sort(plugins: &[Box<dyn MornPlugin>]) -> Result<Vec<usize>, PluginOrderError> {
    let n = plugins.len();
    let mut in_degree = vec![0; n];
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();

    let id_to_idx: HashMap<&str, usize> = plugins
        .iter()
        .enumerate()
        .map(|(i, p)| (p.id(), i))
        .collect();

    for (i, plugin) in plugins.iter().enumerate() {
        for dep in plugin.deps() {
            match id_to_idx.get(dep) {
                Some(&j) => {
                    adj.entry(j).or_default().push(i);
                    in_degree[i] += 1;
                }
                None => {
                    tracing::error!(
                        "[plugin_manager] topological_sort missing dependency: {} -> {}",
                        plugin.id(),
                        dep
                    );
                    return Err(PluginOrderError::MissingDependency(
                        plugin.id().to_string(),
                        dep.to_string(),
                    ));
                }
            }
        }
    }

    let mut queue: VecDeque<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    let mut result = Vec::with_capacity(n);

    while let Some(node) = queue.pop_front() {
        result.push(node);
        if let Some(neighbors) = adj.get(&node) {
            for &next in neighbors {
                in_degree[next] -= 1;
                if in_degree[next] == 0 {
                    queue.push_back(next);
                }
            }
        }
    }

    if result.len() != n {
        let cycle: Vec<String> = (0..n)
            .filter(|&i| in_degree[i] > 0)
            .map(|i| plugins[i].id().to_string())
            .collect();
        tracing::error!(
            "[plugin_manager] topological_sort cycle detected: {:?}",
            cycle
        );
        return Err(PluginOrderError::CycleDetected(cycle));
    }

    Ok(result)
}

pub fn load_plugins(
    plugins: &mut [Box<dyn MornPlugin>],
    ctx: &PluginContext,
) -> Result<(), PluginError> {
    let order = topological_sort(plugins).map_err(|e| PluginError::OrderError(e.to_string()))?;

    for &i in &order {
        safe_init(plugins[i].as_mut(), ctx)?;
    }

    for &i in &order {
        safe_activate(plugins[i].as_mut(), ctx)?;
    }

    Ok(())
}

pub fn safe_init(plugin: &mut dyn MornPlugin, ctx: &PluginContext) -> Result<(), PluginError> {
    let id = plugin.id().to_string();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| plugin.init(ctx))).map_err(
        |panic| {
            let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic".to_string()
            };
            tracing::error!("[plugin_manager] safe_init {} panicked: {}", id, msg);
            PluginError::LoadFailed(id, msg)
        },
    )?
}

pub fn safe_activate(plugin: &mut dyn MornPlugin, ctx: &PluginContext) -> Result<(), PluginError> {
    let id = plugin.id().to_string();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| plugin.activate(ctx))).map_err(
        |panic| {
            let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic".to_string()
            };
            tracing::error!("[plugin_manager] safe_activate {} panicked: {}", id, msg);
            PluginError::ActivateFailed(id, msg)
        },
    )?
}

pub fn safe_deactivate(
    plugin: &mut dyn MornPlugin,
    ctx: &PluginContext,
) -> Result<(), PluginError> {
    let id = plugin.id().to_string();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| plugin.deactivate(ctx))).map_err(
        |panic| {
            let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown panic".to_string()
            };
            tracing::error!("[plugin_manager] safe_deactivate {} panicked: {}", id, msg);
            PluginError::Other(format!("{} deactivate panicked: {}", id, msg))
        },
    )?
}

pub fn enable_plugin(
    plugin: &mut Box<dyn MornPlugin>,
    ctx: &PluginContext,
) -> Result<(), PluginError> {
    safe_init(plugin.as_mut(), ctx)?;
    safe_activate(plugin.as_mut(), ctx)?;
    Ok(())
}

pub fn disable_plugin(
    plugin: &mut Box<dyn MornPlugin>,
    ctx: &PluginContext,
) -> Result<(), PluginError> {
    safe_deactivate(plugin.as_mut(), ctx)
}
