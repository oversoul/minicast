pub struct State {
    max: usize,
    pub value: tui::widgets::ListState,
}

impl State {
    pub fn new() -> Self {
        let mut value = tui::widgets::ListState::default();
        value.select(Some(0));
        State { value, max: 0 }
    }

    pub fn set_max(&mut self, max: usize) {
        self.max = max;
    }

    pub fn get_value(&self) -> usize {
        return self.value.selected().unwrap_or(0);
    }

    pub fn increment(&mut self) {
        let value = self.value.selected().unwrap_or(0);
        if value >= self.max - 1 {
            return;
        }
        self.value.select(Some(value + 1));
    }

    pub fn decrement(&mut self) {
        let value = self.value.selected().unwrap_or(0);
        if value <= 0 {
            return;
        }

        self.value.select(Some(value - 1));
    }
}

