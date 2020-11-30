use glfw::Window;
use crate::Scene;

pub trait State<T> {
	fn enter(&mut self, window: &mut Window, scene: &mut Scene);
	fn leave(&mut self, window: &mut Window, scene: &mut Scene);
	fn handle_event(&mut self, event: &glfw::WindowEvent, window: &mut Window, scene: &mut Scene);
	fn update(&mut self, window: &mut Window, scene: &mut Scene, data: &mut T) -> StateAction<T>;
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
	pub fn new(window: &mut Window, scene: &mut Scene, mut initial_state: Box<dyn State<T>>) -> Self {
		initial_state.enter(window, scene);

		Self { states: vec![initial_state] }
	}

	pub fn handle_event(&mut self, event: &glfw::WindowEvent, window: &mut Window, scene: &mut Scene) {
		for state in &mut self.states {
			state.handle_event(event, window, scene);
		}
	}

	pub fn update(&mut self, window: &mut Window, scene: &mut Scene, data: &mut T) {
		let mut actions = Vec::with_capacity(self.states.len());

		for state in &mut self.states {
			actions.push(state.update(window, scene, data));
		}

		for action in actions {
			match action {
				StateAction::Push(mut state) => {
					state.enter(window, scene);
					self.states.push(state);
				},
				StateAction::Pop => {
					self.states.last_mut().unwrap().leave(window, scene);
					self.states.pop();
				},
				_ => ()
			}
		}
	}
}