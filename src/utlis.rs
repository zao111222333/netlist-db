use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    iter::once,
    ops::Deref as _,
};

use crate::{Subckt, instance::InstanceCtx};
use indexmap::IndexMap;
use itertools::Itertools as _;

impl<'s> Subckt<'s> {
    pub fn append<'a>(
        cktname: &str,
        leaf_cells: &HashSet<String>,
        inst_path: &mut Vec<&'a Cow<'s, str>>,
        include_subckts: &mut Vec<&'a IndexMap<String, Self>>,
        internal_nodes: &mut Vec<String>,
        append_subckts: &mut Vec<&'a Self>,
    ) -> Result<(), String> {
        if leaf_cells.contains(cktname) {
            return Ok(());
        }
        for (idx, include) in include_subckts.iter().enumerate().rev() {
            if let Some(ckt) = include.get(&cktname.to_lowercase()) {
                let mut nodes_map =
                    ckt.ast
                        .instance
                        .iter()
                        .fold(HashMap::new(), |mut nodes_map, inst| {
                            inst.ctx.append_nodes(&mut nodes_map);
                            nodes_map
                        });
                for port in &ckt.ports {
                    _ = nodes_map.remove(&port.to_lowercase());
                }
                internal_nodes.extend(
                    nodes_map
                        .into_values()
                        .map(|n| inst_path.iter().copied().chain(once(n)).join(".")),
                );
                if idx == 0 {
                    // idx==0 means in top
                    append_subckts.push(ckt);
                }
                include_subckts.push(&ckt.ast.subckt);
                for inst in &ckt.ast.instance {
                    if let InstanceCtx::Subckt(ckt) = &inst.ctx {
                        inst_path.push(&inst.name);
                        Self::append(
                            &ckt.cktname,
                            leaf_cells,
                            inst_path,
                            include_subckts,
                            internal_nodes,
                            append_subckts,
                        )?;
                        inst_path.pop();
                    }
                }
                include_subckts.pop();
                return Ok(());
            }
        }
        Err(inst_path
            .iter()
            .map(|inst| (*inst).deref())
            .chain(once(cktname))
            .join("."))
    }
    // fn flatten_instances(
    //     &self,
    //     inst_name: &Cow<'s, str>,
    //     inst_ports: &Vec<Cow<'s, str>>,
    //     inst_params: &Vec<KeyValue<'s>>,
    // ) -> Vec<Instance<'s>> {
    //     let node_name_mapping: HashMap<_, _> = zip_eq(&self.ports, inst_ports)
    //         .map(|(port, inst_port)| (port.to_lowercase(), inst_port))
    //         .collect();
    //     let get_node = |n: &Cow<'s, str>| -> Cow<'s, str> {
    //         if let Some(&inst_port) = node_name_mapping.get(&n.to_lowercase()) {
    //             inst_port.clone()
    //         } else {
    //             n.clone()
    //         }
    //     };
    //     let get_value = |v: &Value<'s>| -> Value<'s> { v.clone() };
    //     self.ast
    //         .instance
    //         .iter()
    //         .map(|inst| Instance {
    //             name: format!("{}.{}", inst.name, inst_name).into(),
    //             ctx: match &inst.ctx {
    //                 InstanceCtx::Resistor(r) => InstanceCtx::Resistor(Resistor {
    //                     n1: get_node(&r.n1),
    //                     n2: get_node(&r.n2),
    //                     value: get_value(&r.value),
    //                 }),
    //                 InstanceCtx::Capacitor(c) => InstanceCtx::Capacitor(Capacitor {
    //                     n1: get_node(&c.n1),
    //                     n2: get_node(&c.n2),
    //                     value: get_value(&c.value),
    //                 }),
    //                 InstanceCtx::Inductor(i) => InstanceCtx::Inductor(Inductor {
    //                     n1: get_node(&i.n1),
    //                     n2: get_node(&i.n2),
    //                     value: get_value(&i.value),
    //                 }),
    //                 InstanceCtx::Voltage(v) => InstanceCtx::Voltage(Voltage {
    //                     n1: get_node(&v.n1),
    //                     n2: get_node(&v.n2),
    //                     source: v.source.clone(),
    //                 }),
    //                 InstanceCtx::Current(c) => InstanceCtx::Current(Current {
    //                     n1: get_node(&c.n1),
    //                     n2: get_node(&c.n2),
    //                     source: c.source.clone(),
    //                 }),
    //                 InstanceCtx::MOSFET(m) => todo!(),
    //                 InstanceCtx::BJT(b) => todo!(),
    //                 InstanceCtx::Diode(d) => todo!(),
    //                 InstanceCtx::Subckt(s) => todo!(),
    //                 InstanceCtx::Unknown {
    //                     r#type,
    //                     nodes,
    //                     params,
    //                 } => todo!(),
    //             },
    //         })
    //         .collect()
    // }
    // pub fn flatten(&mut self, env_subckts: &mut Vec<Rc<IndexMap<String, Subckt<'s>>>>) {
    //     let mut new_inst = Vec::new();
    //     for subckt in self.ast.subckt.values_mut() {
    //         subckt.flatten(env_subckts);
    //     }
    //     // self.params
    //     env_subckts.push(self.ast.subckt);
    //     for inst in mem::take(&mut self.ast.instance) {
    //         if let InstanceCtx::Subckt(a) = &inst.ctx {
    //         } else {
    //             new_inst.push(inst);
    //         }
    //     }
    // }
}

#[cfg(test)]
pub(crate) mod test {
    use std::{collections::HashSet, path::PathBuf};

    use crate::{FileId, Subckt, parser::parse_top_multi};

    pub(crate) fn init_logger() {
        #[cfg(not(feature = "tracing"))]
        {
            _ = simple_logger::SimpleLogger::new().init();
        }
        #[cfg(feature = "tracing")]
        {
            let subscriber = tracing_subscriber::FmtSubscriber::builder()
                // .with_ansi(colored::control::SHOULD_COLORIZE.should_colorize())
                .with_max_level(tracing::Level::DEBUG)
                .with_target(false)
                .with_file(true)
                .with_line_number(true)
                .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
                    "%FT%T".to_owned(),
                ))
                .finish();
            _ = tracing::subscriber::set_global_default(subscriber);
        }
    }

    #[tokio::test]
    async fn test_append() {
        init_logger();
        let (parsed, files) = parse_top_multi(
            [FileId::Include {
                path: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/async_sync_dff.cdl"),
            }]
            .into_iter(),
        )
        .await;
        let (ast, has_err) = files.build(parsed);
        assert!(!has_err);
        let mut internal_nodes = Vec::new();
        let mut append_subckts = Vec::new();
        Subckt::append(
            "ASYNC_SYNC_DFF",
            &HashSet::new(),
            &mut Vec::new(),
            &mut vec![&ast.subckt],
            &mut internal_nodes,
            &mut append_subckts,
        )
        .unwrap();
        dbg!(internal_nodes);
        for append_subckt in append_subckts {
            println!("{append_subckt}");
        }
    }
}
