use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::{Action, App, View};

pub fn dispatch(app: &App, key: KeyEvent) -> Option<Action> {
    if app.help_visible {
        return match key.code {
            KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') | KeyCode::F(1) => Some(Action::ToggleHelp),
            _ => None,
        };
    }

    if matches!(app.current_view(), View::AccountPicker) {
        return match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Action::Back),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::Up),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::Down),
            KeyCode::Enter => Some(Action::SelectAccount),
            KeyCode::Char('?') | KeyCode::F(1) => Some(Action::ToggleHelp),
            _ => None,
        };
    }

    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(Action::Quit),
        (_, KeyCode::Char('q')) => Some(Action::Quit),
        (_, KeyCode::Up) | (_, KeyCode::Char('k')) => Some(Action::Up),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => Some(Action::Down),
        (KeyModifiers::CONTROL, KeyCode::Char('u')) => Some(Action::PageUp),
        (KeyModifiers::CONTROL, KeyCode::Char('d')) => Some(Action::PageDown),
        (_, KeyCode::PageUp) => Some(Action::PageUp),
        (_, KeyCode::PageDown) => Some(Action::PageDown),
        (_, KeyCode::Home) | (_, KeyCode::Char('g')) => Some(Action::Home),
        (_, KeyCode::End) | (_, KeyCode::Char('G')) => Some(Action::End),
        (_, KeyCode::Enter) | (_, KeyCode::Char('l')) | (_, KeyCode::Right) => Some(Action::Enter),
        (_, KeyCode::Esc) | (_, KeyCode::Backspace) | (_, KeyCode::Char('h')) | (_, KeyCode::Left) => Some(Action::Back),
        (_, KeyCode::Char('r')) => Some(Action::Refresh),
        (_, KeyCode::Char('s')) => Some(Action::OpenAccountPicker),
        (_, KeyCode::Char('?')) | (_, KeyCode::F(1)) => Some(Action::ToggleHelp),
        _ => None,
    }
}
