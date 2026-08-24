use rust_i18n::t;

pub fn render_tutorial_prompt() -> (String, String) {
    let text = t!("tutorial.prompt_text").to_string();
    let spoken = t!("tutorial.prompt_spoken").to_string();
    (text, spoken)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_tutorial_prompt() {
        let (text, spoken) = render_tutorial_prompt();
        assert!(text.contains("Audio Tetris"));
        assert!(text.contains("Tutorial"));
        assert!(!spoken.is_empty());
    }
}
