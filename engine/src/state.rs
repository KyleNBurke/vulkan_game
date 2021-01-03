use glfw::Window;
use crate::{vulkan::Renderer, Scene};

pub struct EngineResources<'a, T> {
	pub window: Window,
	pub renderer: Renderer<'a>,
	pub game_resources: T,
	pub scene: Scene
}

pub trait State<T> {
	fn enter(&mut self, resources: &mut EngineResources<T>);
	fn leave(&mut self, resources: &mut EngineResources<T>);
	fn handle_event(&mut self, event: &glfw::WindowEvent, resources: &mut EngineResources<T>);
	fn update(&mut self, resources: &mut EngineResources<T>) -> StateAction<T>;
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
	pub fn new(resources: &mut EngineResources<T>, mut initial_state: Box<dyn State<T>>) -> Self {
		initial_state.enter(resources);

		Self { states: vec![initial_state] }
	}

	pub fn handle_event(&mut self, event: &glfw::WindowEvent, resources: &mut EngineResources<T>) {
		for state in &mut self.states {
			state.handle_event(event, resources);
		}
	}

	pub fn update(&mut self, resources: &mut EngineResources<T>) {
		let mut actions = Vec::with_capacity(self.states.len());

		for state in &mut self.states {
			actions.push(state.update(resources));
		}

		for action in actions {
			match action {
				StateAction::Push(mut state) => {
					state.enter(resources);
					self.states.push(state);
				},
				StateAction::Pop => {
					self.states.last_mut().unwrap().leave(resources);
					self.states.pop();
				},
				_ => ()
			}
		}
	}
}