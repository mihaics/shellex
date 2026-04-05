use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{stdout, Write};

pub enum UserAction {
    Run(String),
    Cancel,
}

/// Display the generated command and wait for user action.
pub fn prompt_command(command: &str) -> Result<UserAction> {
    let mut out = stdout();

    // Show the command in green
    execute!(
        out,
        SetForegroundColor(Color::Green),
        Print(format!("> {}", command)),
        ResetColor,
        Print("\n"),
        Print("  [Enter] Run  [Tab] Edit  [Esc] Cancel\n"),
    )?;

    terminal::enable_raw_mode()?;
    let result = wait_for_action(command);
    terminal::disable_raw_mode()?;

    // Clear the hint line
    execute!(out, cursor::MoveUp(1), terminal::Clear(ClearType::CurrentLine))?;

    result
}

fn wait_for_action(command: &str) -> Result<UserAction> {
    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => return Ok(UserAction::Run(command.to_string())),
                KeyCode::Tab => return edit_command(command),
                KeyCode::Esc => return Ok(UserAction::Cancel),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(UserAction::Cancel);
                }
                _ => {}
            }
        }
    }
}

fn edit_command(initial: &str) -> Result<UserAction> {
    let mut out = stdout();
    let mut buffer: Vec<char> = initial.chars().collect();
    let mut cursor_pos: usize = buffer.len();

    // Clear and redraw with editable prompt
    execute!(
        out,
        cursor::MoveUp(2),
        terminal::Clear(ClearType::FromCursorDown),
    )?;
    redraw_edit_line(&buffer, cursor_pos)?;

    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => {
                    execute!(out, Print("\n"))?;
                    let cmd: String = buffer.into_iter().collect();
                    return Ok(UserAction::Run(cmd));
                }
                KeyCode::Esc => {
                    execute!(out, Print("\n"))?;
                    return Ok(UserAction::Cancel);
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    execute!(out, Print("\n"))?;
                    return Ok(UserAction::Cancel);
                }
                KeyCode::Left if cursor_pos > 0 => {
                    cursor_pos -= 1;
                    redraw_edit_line(&buffer, cursor_pos)?;
                }
                KeyCode::Right if cursor_pos < buffer.len() => {
                    cursor_pos += 1;
                    redraw_edit_line(&buffer, cursor_pos)?;
                }
                KeyCode::Home => {
                    cursor_pos = 0;
                    redraw_edit_line(&buffer, cursor_pos)?;
                }
                KeyCode::End => {
                    cursor_pos = buffer.len();
                    redraw_edit_line(&buffer, cursor_pos)?;
                }
                KeyCode::Backspace if cursor_pos > 0 => {
                    cursor_pos -= 1;
                    buffer.remove(cursor_pos);
                    redraw_edit_line(&buffer, cursor_pos)?;
                }
                KeyCode::Delete if cursor_pos < buffer.len() => {
                    buffer.remove(cursor_pos);
                    redraw_edit_line(&buffer, cursor_pos)?;
                }
                KeyCode::Char(c) => {
                    buffer.insert(cursor_pos, c);
                    cursor_pos += 1;
                    redraw_edit_line(&buffer, cursor_pos)?;
                }
                _ => {}
            }
        }
    }
}

fn redraw_edit_line(buffer: &[char], cursor_pos: usize) -> Result<()> {
    let mut out = stdout();
    let text: String = buffer.iter().collect();

    execute!(
        out,
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::CurrentLine),
        SetForegroundColor(Color::Yellow),
        Print(format!("> {}", text)),
        ResetColor,
        cursor::MoveToColumn((cursor_pos + 2) as u16), // +2 for "> " prefix
    )?;
    out.flush()?;
    Ok(())
}

/// Display a dangerous command warning and require explicit "yes" to proceed.
pub fn prompt_dangerous(command: &str, matched_patterns: &[String]) -> Result<UserAction> {
    let mut out = stdout();
    terminal::disable_raw_mode().ok(); // Ensure we're not in raw mode

    execute!(
        out,
        SetForegroundColor(Color::Red),
        Print("\nWarning: This command matches a dangerous pattern:\n"),
    )?;
    for pattern in matched_patterns {
        execute!(out, Print(format!("  - {}\n", pattern)))?;
    }
    execute!(
        out,
        ResetColor,
        Print(format!("shellex generated: {}\n\n", command)),
        Print("Type 'yes' to proceed, anything else cancels: "),
    )?;
    out.flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim() == "yes" {
        Ok(UserAction::Run(command.to_string()))
    } else {
        Ok(UserAction::Cancel)
    }
}

/// Show the command in --yes mode (prints to stderr so stdout stays clean).
pub fn print_yes_mode(command: &str) {
    eprintln!("> {}", command);
}
