//! Modelling the grid by bringing other parts together happens in this module.

use std::sync::Arc;

use log::{debug, info};
use parking_lot::RwLock;

use crate::agent::{AgentKind, AgentList, Area, AreaList, Household, HouseholdList, Netstation, NetstationList, Root};
use crate::grid::{PowerGeneration, ReservePower};
use crate::util::{random_percentage, uni_dist, BaseUint, ModelError, Watt};

mod modelparameters;
mod mygraph;
mod step;

pub use modelparameters::*;
pub use mygraph::*;
pub use step::*;

/// Struct for creating and running a model based on the respective
/// [`ModelParameters`].
pub struct Model {
    /// The parameters of the model. See [ModelParameters]
    pub param:         ModelParameters,
    /// The graph of the model
    pub graph:         MyGraph,
    /// The list of agents in the model.
    pub agents:        AgentList,
    /// ReservePower
    pub reserve_power: ReservePower,
    /// The [Root] agent of the model.
    pub root:          Arc<RwLock<Root>>,
    /// A list of [Area] agents that are part of the model
    pub areas:         Vec<Arc<RwLock<Area>>>,
    /// A list of [Netstation] agents that are part of the model.
    pub netstations:   Vec<Arc<RwLock<Netstation>>>,
    /// A list of [Household] agents that are part of the model.
    pub households:    Vec<Arc<RwLock<Household>>>,
}
impl Model {
    #[allow(clippy::type_complexity)]
    fn generate_graph_and_agents(
        param: &mut ModelParameters,
    ) -> Result<
        (
            MyGraph,
            AgentList,
            Arc<RwLock<Root>>,
            AreaList,
            NetstationList,
            HouseholdList,
        ),
        ModelError,
    > {
        let mut agents: AgentList = vec![];
        let mut mygraph: MyGraph = MyGraph::default();

        let mut areas: Vec<Arc<RwLock<Area>>> = vec![];
        let mut netstations: Vec<Arc<RwLock<Netstation>>> = vec![];
        let mut households: Vec<Arc<RwLock<Household>>> = vec![];

        // Root
        let root_index = agents.len() as BaseUint;
        let root = Root::new(root_index, &param.grid);
        mygraph.add_node(AgentKind::Root, root_index)?;
        let root = Arc::new(RwLock::new(root));
        agents.push(root.clone());

        // Areas
        for _ in 0..param.grid.n_areas {
            let area_index = agents.len() as BaseUint;
            let area = Area::new(area_index);
            let a = Arc::new(RwLock::new(area));
            agents.push(a.clone());
            areas.push(a);

            mygraph.add_node(AgentKind::Area, area_index)?;
            mygraph.add_edge(root_index, area_index)?;

            // Netstations
            for _ in 0..uni_dist(param.grid.ns_per_a, &mut param.seed) {
                let netstation_index = agents.len() as BaseUint;
                let netstation = Netstation::new(netstation_index, &param.grid);
                let ns = Arc::new(RwLock::new(netstation));
                agents.push(ns.clone());
                netstations.push(ns);

                mygraph.add_node(AgentKind::Netstation, netstation_index)?;
                mygraph.add_edge(area_index, netstation_index)?;

                // Households
                for _ in 0..uni_dist(param.grid.hs_per_ns, &mut param.seed) {
                    // PowerGeneration
                    let random_perc = random_percentage(&mut param.seed);
                    let power_generation = if param.grid.pv_adoption > random_perc {
                        PowerGeneration::new_pv(None, param)?
                    } else {
                        PowerGeneration::new_no_pv(None, param)?
                    };

                    // Household
                    let household_index = agents.len() as BaseUint;
                    let household = Household::new(household_index, power_generation);
                    let h = Arc::new(RwLock::new(household));
                    agents.push(h.clone());
                    households.push(h);

                    mygraph.add_node(AgentKind::Household, household_index)?;
                    mygraph.add_edge(netstation_index, household_index)?;
                }
            }
        }
        Ok((mygraph, agents, root, areas, netstations, households))
    }

    /// Creating the graph and agents and adding agents to the graph. Graph is
    /// tree. Ordinality: Area -> Netstation -> Households -> PV.
    pub fn new(mut model_param: ModelParameters) -> Result<Self, ModelError> {
        let model_name: &str = &model_param.name.clone();
        info!("{model_name} - Agent and Graph generation");
        let (mygraph, agents, root, areas, netstations, households) =
            Self::generate_graph_and_agents(&mut model_param)?;

        info!("{model_name} - Populating agents with their children");
        // Populate the agents with references to their children
        for rw_agent in &agents {
            let mut agent = rw_agent.write();
            for child in mygraph.get_children(&(*agent.index() as i32))? {
                agent.children_mut().push(agents[child as usize].clone());
            }
        }

        let reserve_power = ReservePower {
            lower_limit:   model_param.grid.energy_storage * -1,
            upper_limit:   model_param.grid.energy_storage,
            current_usage: Watt(0),
            watt_per_step: model_param.grid.max_gen_inc_tick,
        };

        debug!("{model_name} - finished building the model");
        Ok(Self {
            param: model_param,
            graph: mygraph,
            agents,
            reserve_power,
            root,
            areas,
            netstations,
            households,
        })
    }

    /// Run the model. Generates the outcome by calling the step function.
    pub fn run(&mut self) -> Result<(), ModelError> {
        let name = self.param.name.clone();
        info!("{} - Running", name);
        self.step(self.param.steps)?;
        info!("{} - Finished running", name);
        Ok(())
    }
}

#[cfg(test)]
mod model_tests {

    use super::*;
    use crate::grid::PowerState;
    use crate::util::{output_graph_to_png, Steps};

    #[test]
    #[ignore]
    fn print_test_graph() {
        let model = Model::new(ModelParameters::test()).unwrap();
        output_graph_to_png(&"testgraph.dot".to_string(), &model.graph).expect("could not print");
    }

    #[test]
    fn getting_children() {
        let model = Model::new(ModelParameters::test()).unwrap();
        assert_eq!(model.graph.get_children(&0).expect("Couldnt get children"), vec![1]);
        assert_eq!(model.graph.get_children(&1).expect("Couldnt get children"), vec![2, 5]);
        assert_eq!(model.graph.get_children(&2).expect("Couldnt get children"), vec![3, 4]);
        assert_eq!(model.graph.get_children(&5).expect("Couldnt get children"), vec![6, 7]);
    }

    #[test]
    fn grid_power_sum_clean() {
        let param = ModelParameters::test();
        let mut model = Model::new(param).unwrap();
        model.step(Steps(96)).expect("Error in taking steps");

        let agent0 = &model.agents.get(0).expect("No Agent 0");

        let agent1 = &model.agents.get(1).expect("No Agent 1");

        let agent2 = &model.agents.get(2).expect("No Agent 2");

        let agent3 = &model.agents.get(3).expect("No Agent 3");

        let agent4 = &model.agents.get(4).expect("No Agent 4");

        let agent5 = &model.agents.get(5).expect("No Agent 5");

        let agent6 = &model.agents.get(6).expect("No Agent 6");

        let agent7 = &model.agents.get(7).expect("No Agent 7");

        let mut comp1 = PowerState::new();
        let mut comp2 = PowerState::new();
        let mut comp3 = PowerState::new();
        comp1.manual_add(agent3.read_arc_recursive().powerstate());
        comp1.manual_add(agent4.read_arc_recursive().powerstate());
        comp2.manual_add(agent6.read_arc_recursive().powerstate());
        comp2.manual_add(agent7.read_arc_recursive().powerstate());
        comp3.manual_add(agent2.read_arc_recursive().powerstate());
        comp3.manual_add(agent5.read_arc_recursive().powerstate());

        // println!("{:?}", agent0);
        // println!("{:?}", agent1);
        // println!("{:?}",agent2);
        // println!("{:?}",agent3);
        // println!("{:?}",agent4);
        // println!("{:?}",agent5);
        // println!("{:?}",agent6);
        // println!("{:?}",agent7);
        // println!("{:?}",comp1);
        // println!("{:?}",comp2);
        // println!("{:?}",comp3);
        // println!("agent0: {:}\n\n", agent0.read_arc_recursive());

        assert_eq!(*agent2.read_arc_recursive().powerstate(), comp1);
        assert_eq!(*agent5.read_arc_recursive().powerstate(), comp2);
        assert_eq!(*agent1.read_arc_recursive().powerstate(), comp3);
        assert_eq!(
            *agent0.read_arc_recursive().powerstate(),
            *agent1.read_arc_recursive().powerstate()
        );
    }
}
