// use std::{borrow::Cow, collections::HashMap, mem, rc::Rc};

// use crate::{
//     Subckt,
//     ast::{KeyValue, Value},
//     instance::{Capacitor, Current, Inductor, Instance, InstanceCtx, Resistor, Voltage},
// };
// use indexmap::IndexMap;
// use itertools::zip_eq;

// impl<'s> Subckt<'s> {
//     fn flatten_instances(
//         &self,
//         inst_name: &Cow<'s, str>,
//         inst_ports: &Vec<Cow<'s, str>>,
//         inst_params: &Vec<KeyValue<'s>>,
//     ) -> Vec<Instance<'s>> {
//         let node_name_mapping: HashMap<_, _> = zip_eq(&self.ports, inst_ports)
//             .map(|(port, inst_port)| (port.to_lowercase(), inst_port))
//             .collect();
//         let get_node = |n: &Cow<'s, str>| -> Cow<'s, str> {
//             if let Some(&inst_port) = node_name_mapping.get(&n.to_lowercase()) {
//                 inst_port.clone()
//             } else {
//                 n.clone()
//             }
//         };
//         let get_value = |v: &Value<'s>| -> Value<'s> { v.clone() };
//         self.ast
//             .instance
//             .iter()
//             .map(|inst| Instance {
//                 name: format!("{}.{}", inst.name, inst_name).into(),
//                 ctx: match &inst.ctx {
//                     InstanceCtx::Resistor(r) => InstanceCtx::Resistor(Resistor {
//                         n1: get_node(&r.n1),
//                         n2: get_node(&r.n2),
//                         value: get_value(&r.value),
//                     }),
//                     InstanceCtx::Capacitor(c) => InstanceCtx::Capacitor(Capacitor {
//                         n1: get_node(&c.n1),
//                         n2: get_node(&c.n2),
//                         value: get_value(&c.value),
//                     }),
//                     InstanceCtx::Inductor(i) => InstanceCtx::Inductor(Inductor {
//                         n1: get_node(&i.n1),
//                         n2: get_node(&i.n2),
//                         value: get_value(&i.value),
//                     }),
//                     InstanceCtx::Voltage(v) => InstanceCtx::Voltage(Voltage {
//                         n1: get_node(&v.n1),
//                         n2: get_node(&v.n2),
//                         source: v.source.clone(),
//                     }),
//                     InstanceCtx::Current(c) => InstanceCtx::Current(Current {
//                         n1: get_node(&c.n1),
//                         n2: get_node(&c.n2),
//                         source: c.source.clone(),
//                     }),
//                     InstanceCtx::MOSFET(m) => todo!(),
//                     InstanceCtx::BJT(b) => todo!(),
//                     InstanceCtx::Diode(d) => todo!(),
//                     InstanceCtx::Subckt(s) => todo!(),
//                     InstanceCtx::Unknown {
//                         r#type,
//                         nodes,
//                         params,
//                     } => todo!(),
//                 },
//             })
//             .collect()
//     }
//     pub fn flatten(&mut self, env_subckts: &mut Vec<Rc<IndexMap<String, Subckt<'s>>>>) {
//         let mut new_inst = Vec::new();
//         for subckt in self.ast.subckt.values_mut() {
//             subckt.flatten(env_subckts);
//         }
//         // self.params
//         env_subckts.push(self.ast.subckt);
//         for inst in mem::take(&mut self.ast.instance) {
//             if let InstanceCtx::Subckt(a) = &inst.ctx {
//             } else {
//                 new_inst.push(inst);
//             }
//         }
//     }
// }
