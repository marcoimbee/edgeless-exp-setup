use edgeless_orc::proxy::Proxy;
use redis::Commands;
use std::process::Command;
use uuid::Uuid;

/*
    TODO: (maybe?) rebalance only if the latency changed of some quantity from the last time
*/

#[derive(Debug)]
struct NodeDesc {
    function_instances: Vec<edgeless_api::function_instance::ComponentId>,
    capabilities: edgeless_api::node_registration::NodeCapabilities,    
    resource_providers: std::collections::HashSet<String>,
    node_to_node_latency: f64,      // Latency from this node to the other one
    node_to_orc_latency: f64,       // Latency from this node to the orchestrator
}

#[derive(Debug)]
struct InstanceDesc {
    runtime: String,            // Python or Rust

    // Deployment requirements for functions, as specified by annotations
    // Among others:
    //      -> node_id_match_any: Vec<uuid::Uuid>
    deployment_requirements: edgeless_orc::deployment_requirements::DeploymentRequirements,
    relocatable: bool,          // Will be true if node_id_match_any is empty
}

pub struct NetworkAwareOrchestrator {
    proxy: edgeless_orc::proxy_redis::ProxyRedis,
    redis_url: String,
    nodes: std::collections::HashMap<edgeless_api::function_instance::NodeId, NodeDesc>,
    instances:std::collections::HashMap<edgeless_api::function_instance::ComponentId, InstanceDesc>,
    latency_threshold: f64,
    num_relocations: u64,       // How many functions to relocate, if possible
    relocated: bool,            // Tells if relocation(s) have already been done
}

impl NetworkAwareOrchestrator {
    pub fn new(redis_url: &str, latency_threshold: f64, num_relocations: u64) -> anyhow::Result<Self> {
        let proxy = match edgeless_orc::proxy_redis::ProxyRedis::new(redis_url, false, None) {
            Ok(proxy) => proxy,
            Err(err) => anyhow::bail!("could not connect to Redis at {}: {}.", redis_url, err),
        };

        Ok(Self {
            proxy,
            redis_url: redis_url.to_string(),
            nodes: std::collections::HashMap::new(),
            instances: std::collections::HashMap::new(),
            latency_threshold,
            num_relocations,
            relocated: false,          
        })
    }

    pub fn move_to_rpi(&mut self, rpi_node_id: &str) -> (bool, u32) {
        self.get_function_instances();
        self.update_node_desc();

        let mut counter = 0;
        for (_node_id, node_desc) in &self.nodes {
            for lid in &node_desc.function_instances {
                let status = Command::new("../edgeless/target/debug/proxy_cli")
                    .arg("intent")
                    .arg("migrate")
                    .arg(lid.to_string())
                    .arg(rpi_node_id)
                    .status()
                    .expect("Failed to execute proxy_cli command.");

                counter += 1;

                if status.success() {
                    println!("[INFO] Successfully moved.");
                } else {
                    eprintln!("[ERROR] Command failed with status: {}.", status);
                    return (false, counter);
                }
            }
        }

        (true, counter)
    }

    pub fn monitor_cluster(&mut self) {
        // Init NAO structure
        self.get_function_instances();
        println!("----------------------------------------------------------------------------------");

        // Fetch the status of nodes
        self.update_node_desc();
        println!("----------------------------------------------------------------------------------");

        for (node_id, node_desc) in &self.nodes {
            for lid in &node_desc.function_instances {
                let instance_desc = self
                    .instances
                    .get(lid)
                    .expect("Function instance disappeared.");

                if instance_desc.relocatable {
                    println!("[INFO] {} (spawned on: {}) is relocatable.", lid, node_id);
                }
            }
        }
    }

    pub fn rebalance(&mut self) -> (bool, usize) {
        // if !self.proxy.updated(edgeless_orc::proxy::Category::NodeCapabilities)
        //     && !self.proxy.updated(edgeless_orc::proxy::Category::ResourceProviders)
        //     && !self.proxy.updated(edgeless_orc::proxy::Category::ActiveInstances)
        // {
        //     println!("[INFO] Nothing has changed since the last iteration. Nothing to do.");
        //     return (false, 0);
        // }

        // Init NAO structure
        self.get_function_instances();
        println!("----------------------------------------------------------------------------------");

        // Fetch the status of nodes
        self.update_node_desc();
        println!("----------------------------------------------------------------------------------");

        // Try to rebalance things
        (true, self.migrate())
    }

    fn get_function_instances(&mut self) {
        self.instances.clear();

        let mut instances = self.proxy.fetch_function_instance_requests();
        for (lid, req) in &mut instances {
            let runtime = req.code.function_class_type.clone();
            let deployment_requirements = edgeless_orc::deployment_requirements::DeploymentRequirements::from_annotations(
                &req.annotations,
            );

            let relocatable = deployment_requirements.node_id_match_any.is_empty();

            let instance_desc = InstanceDesc {
                runtime,
                deployment_requirements,
                relocatable,
            };

            self.instances.insert(
                *lid,
                instance_desc,
            );
        }
    }

    fn update_node_desc(&mut self) {
        // Create node descriptors, with capabilities
        self.nodes.clear();

        for (node_id, capabilities) in self.proxy.fetch_node_capabilities() {
            self.nodes.insert(
                node_id,
                NodeDesc {
                    function_instances: vec![],
                    capabilities,
                    resource_providers: std::collections::HashSet::new(),
                    node_to_node_latency: 0.0,
                    node_to_orc_latency: 0.0,
                },
            );

            // println!("[INFO] Fetched capabilities for node {}.", node_id);
        }

        // Add function instances spawned on the node
        let mut instances = self.proxy.fetch_nodes_to_instances();
        if instances.is_empty() {
            println!("[INFO] No function instances found.");
        } else {
            for (node_id, instances) in &mut instances {
                let node_function_instances = &mut self
                    .nodes
                    .get_mut(node_id)
                    .expect("Cannot find node.")
                    .function_instances;
                
                let func_instances = instances.len();
                println!("[INFO] Fetched {} function instances for node {}.", func_instances, node_id);
                for instance in instances {
                    if let edgeless_orc::proxy::Instance::Function(lid) = instance {
                        println!("\t-> LID: {}", lid.to_string());
                        node_function_instances.push(*lid);
                    }
                }
            }
        }

        // Add resource providers
        let providers = self.proxy.fetch_resource_providers();
        for (provider_id, resource_provider) in providers {
            let node_resource_providers = &mut self
                .nodes
                .get_mut(&resource_provider.node_id)
                .expect("Cannot find node.")
                .resource_providers;
            node_resource_providers.insert(provider_id);
        }

        // Get latencies
        let client = redis::Client::open(self.redis_url.clone()).expect("[ERROR] Failed to create Redis client.");
        let mut conn = client.get_connection().expect("[ERROR] Failed to get Redis connection.");

        // Obtaining node-to-node latencies
        let node_ids: Vec<_> = self.nodes.keys().cloned().collect();        // Needed to avoid double borrowing problems
        for node1_id in &node_ids {
            if let Some(node1_desc) = self.nodes.get_mut(node1_id) {
                for node2_id in &node_ids {
                    if node1_id != node2_id {
                        let redis_key = format!("latency:{}:{}", node1_id, node2_id);
                        match conn.get::<_, Option<f64>>(&redis_key) {
                            Ok(Some(latency_value)) => {
                                node1_desc.node_to_node_latency = latency_value;
                                println!(
                                    "[INFO] {} ---> {} ms ---> {}",
                                    node1_id, latency_value, node2_id
                                );
                            }
                            Ok(None) => {
                                println!("[WARN] Not measuring latency from VM-node to RPI (key {} not set)", redis_key);
                            }
                            Err(err) => {
                                println!("[ERROR] Failed to fetch key {}: {}.", redis_key, err);
                            }
                        }
                    }
                }
            }
        }

        // // Obtaining node-to-orc latencies
        // for (node_id, node_desc) in &mut self.nodes {
        //     let redis_key = format!("latency:{}:orc", node_id);
        //     match conn.get::<_, Option<f64>>(&redis_key) {
        //         Ok(Some(latency_value)) => {
        //             node_desc.node_to_orc_latency = latency_value;
        //             println!("[INFO] {} ---> {} ms ---> E-ORC", node_id, latency_value);
        //         }
        //         Ok(None) => {
        //             println!("[WARN] Key {} not found", redis_key);
        //         }
        //         Err(err) => {
        //             println!("[ERROR] Failed to fetch key {}: {}", redis_key, err);
        //         }
        //     }
        // }
    }

    fn string_to_uuid(&mut self, input: &str) -> Uuid {
        Uuid::parse_str(input).unwrap_or_else(|_| Uuid::nil())
    }

    fn get_relocatable_functions_count(&self, node_uuid: &Uuid) -> usize {
        if let Some(node_desc) = self.nodes.get(node_uuid) {
            node_desc
                .function_instances
                .iter()
                .filter(|&lid| {
                    if let Some(instance_desc) = self.instances.get(lid) {
                        instance_desc.relocatable
                    } else {
                        false
                    }
                })
                .count()
        } else {
            0
        }
    }
    

    fn migrate(&mut self) -> usize {
        
        let rpi_node_uuid = self.string_to_uuid("c7126760-223a-44a4-9a61-4ce1eaca8141");            // TODO: better way to handle these
        let vm_node_uuid = self.string_to_uuid("41cea154-cc7a-4eb9-8337-c0947ccba13e");

        let rpi_to_vm_latency = self
            .nodes
            .get(&rpi_node_uuid)
            .map_or(f64::INFINITY, |desc| desc.node_to_node_latency);

        println!("[INFO] RPI to VM node latency: {} ms. Threshold set to {} ms.", rpi_to_vm_latency, self.latency_threshold);
        println!("----------------------------------------------------------------------------------");

        // Check if we have already relocated and if so return right away
        if self.relocated {
            println!("[INFO] Relocation round already performed. Exiting...");
            return 0;
        }

        // Check if no relocatable functions exist on the RPI, if so we can return right away
        if self.get_relocatable_functions_count(&rpi_node_uuid) == 0 {
            println!("[INFO] No relocatable functions found on node {} (RPI node).", rpi_node_uuid);
            return 0;
        }

        let mut migrations = Vec::new();        // Migrations vector

        // Determine migrations to be performed
        if rpi_to_vm_latency < self.latency_threshold {
            if let Some(node_desc) = self.nodes.get(&rpi_node_uuid) {
                let mut relocations_count = 0;          // Tracks the number of performed relocations

                for lid in &node_desc.function_instances {
                    if relocations_count >= self.num_relocations {
                        break;      // Desired # relocations reached
                    }

                    let instance_desc = self.instances.get(lid).expect("Function instance disappeared.");

                    if instance_desc.relocatable {
                        println!("[INFO] Relocating {} from {} to {}.", lid, rpi_node_uuid, vm_node_uuid);

                        migrations.push(edgeless_orc::deploy_intent::DeployIntent::Migrate(
                            *lid,
                            vec![vm_node_uuid],
                        ));

                        relocations_count += 1;
                        self.relocated = true;
                    }
                }
            }
        } else {
            println!("[INFO] RPI-to-VM latency exceeds threshold, no migration performed.");
        }

        if !migrations.is_empty() {
            self.proxy.add_deploy_intents(migrations.clone());
        }

        migrations.len()
    }
}
