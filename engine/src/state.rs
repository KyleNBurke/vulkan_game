use glfw::Window;
use crate::SceneGraph;

pub trait State<T> {
	fn enter(&mut self, window: &mut Window, scene_graph: &mut SceneGraph);
	fn leave(&mut self, window: &mut Window, scene_graph: &mut SceneGraph);
	fn handle_event(&mut self, event: &glfw::WindowEvent, window: &mut Window, scene_graph: &mut SceneGraph);
	fn update(&mut self, window: &mut Window, scene_graph: &mut SceneGraph, data: &mut T) -> StateAction<T>;
}

pub enum StateAction<T> {
	None,
	Push(Box<dyn State<T>>),
	Pop
}

pub struct StateManager<T> {
	states: Vec<Box<dyn State<T>>>
}

impl<T> StateManager<T> {
	pub fn new(window: &mut Window, scene_graph: &mut SceneGraph, mut initial_state: Box<dyn State<T>>) -> Self {
		initial_state.enter(window, scene_graph);

		Self { states: vec![initial_state] }
	}

	pub fn handle_event(&mut self, event: &glfw::WindowEvent, window: &mut Window, scene_graph: &mut SceneGraph) {
		for state in &mut self.states {
			state.handle_event(event, window, scene_graph);
		}
	}

	pub fn update(&mut self, window: &mut Window, scene_graph: &mut SceneGraph, data: &mut T) {
		let mut actions = Vec::with_capacity(self.states.len());

		for state in &mut self.states {
			actions.push(state.update(window, scene_graph, data));
		}

		for action in actions {
			match action {
				StateAction::Push(mut state) => {
					state.enter(window, scene_graph);
					self.states.push(state);
				},
				StateAction::Pop => {
					self.states.last_mut().unwrap().leave(window, scene_graph);
					self.states.pop();
				},
				_ => ()
			}
		}
	}
}