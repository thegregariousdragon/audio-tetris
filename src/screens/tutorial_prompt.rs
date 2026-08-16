pub fn render_tutorial_prompt() -> (String, String) {
    let text = "Welcome to Audio Tetris!\n\nWould you like to play the interactive tutorial?\n\n-> Press Enter to Start Tutorial\n   Press Escape to Go to Main Menu".to_string();
    let spoken = "Welcome to Audio Tetris! Would you like to play the interactive tutorial? Press Enter for Yes, or Escape for No.".to_string();
    (text, spoken)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_tutorial_prompt() {
        let (text, spoken) = render_tutorial_prompt();
        assert!(text.contains("Welcome to Audio Tetris"));
        assert!(text.contains("Press Enter to Start Tutorial"));
        assert!(spoken.contains("Press Enter for Yes, or Escape for No."));
    }
}
